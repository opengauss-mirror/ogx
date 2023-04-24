/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use crate::command::stop::stop_opengauss;
use crate::command::version::ogx_default;
use crate::CommandExecute;
use eyre::{eyre, WrapErr};
use owo_colors::OwoColorize;
use ogx_pg_config::{prefix_path, PgConfig, PgConfigSelector, Ogx, SUPPORTED_MAJOR_VERSIONS};
use rayon::prelude::*;
use sysinfo::{System, SystemExt};

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Stdio;

use std::sync::{Arc, Mutex};

static PROCESS_ENV_DENYLIST: &'static [&'static str] = &[
    "DEBUG",
    "MAKEFLAGS",
    "MAKELEVEL",
    "MFLAGS",
    "DYLD_FALLBACK_LIBRARY_PATH",
    "OPT_LEVEL",
    "TARGET",
    "PROFILE",
    "OUT_DIR",
    "HOST",
    "NUM_JOBS",
    "LIBRARY_PATH", 
];

/// Initialize ogx development environment for the first time
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct Init {
    /// If installed locally, the path to OG3's `pgconfig` tool, or `download` to have ogx download/compile/install it
    #[clap(env = "OG3_PG_CONFIG", long, help = "")]
    og3: Option<String>,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
    #[clap(long, help = "Base port number")]
    base_port: Option<u16>,
    #[clap(long, help = "Base testing port number")]
    base_testing_port: Option<u16>,
}

impl CommandExecute for Init {
    #[tracing::instrument(level = "error", skip(self))]
    fn execute(self) -> eyre::Result<()> {
        let mut versions = HashMap::new();

        if let Some(ref version) = self.og3 {
            versions.insert("og3", version.clone());
        }

        if versions.is_empty() {
            // no arguments specified, so we'll just install our defaults
            init_ogx(&ogx_default(SUPPORTED_MAJOR_VERSIONS)?, &self)
        } else {
            // user specified arguments, so we'll only install those versions of openGauss
            let mut default_ogx = None;
            let mut ogx = Ogx::default();

            for (ogver, pg_config_path) in versions {
                let config = if pg_config_path == "download" {
                    if default_ogx.is_none() {
                        default_ogx = Some(ogx_default(SUPPORTED_MAJOR_VERSIONS)?);
                    }
                    default_ogx
                        .as_ref()
                        .unwrap() // We just set this
                        .get(&ogver)
                        .wrap_err_with(|| format!("{} is not a known openGauss version", ogver))?
                        .clone()
                } else {
                    PgConfig::new_with_defaults(pg_config_path.into())
                };
                ogx.push(config);
            }

            init_ogx(&ogx, &self)
        }
    }
}

#[tracing::instrument(skip_all, fields(ogx_home = %Ogx::home()?.display()))]
pub(crate) fn init_ogx(ogx: &Ogx, init: &Init) -> eyre::Result<()> {
    let dir = Ogx::home()?;

    let output_configs = Arc::new(Mutex::new(Vec::new()));

    let mut pg_configs = Vec::new();
    for pg_config in ogx.iter(PgConfigSelector::All) {
        pg_configs.push(pg_config?);
    }

    let span = tracing::Span::current();
    pg_configs
        .into_par_iter()
        .map(|pg_config| {
            let _span = span.clone().entered();
            let mut pg_config = pg_config.clone();
            stop_opengauss(&pg_config).ok(); // no need to fail on errors trying to stop openGauss while initializing
            if !pg_config.is_real() {
                pg_config = match download_install_opengauss(&pg_config, &dir) {
                    Ok(pg_config) => pg_config,
                    Err(e) => return Err(eyre!(e)),
                }
            }

            let mut mutex = output_configs.lock();
            // PoisonError doesn't implement std::error::Error, can't `?` it.
            let output_configs = mutex.as_mut().expect("failed to get output_configs lock");

            output_configs.push(pg_config);
            Ok(())
        })
        .collect::<eyre::Result<()>>()?;

    let mut mutex = output_configs.lock();
    // PoisonError doesn't implement std::error::Error, can't `?` it.
    let output_configs = mutex.as_mut().unwrap();

    output_configs.sort_by(|a, b| {
        a.major_version()
            .ok()
            .expect("could not determine major version")
            .cmp(&b.major_version().ok().expect("could not determine major version"))
    });
    for pg_config in output_configs.iter() {
        validate_pg_config(pg_config)?;

        if is_root_user() {
            println!("{} initdb as current user is root user", "   Skipping".bold().green(),);
        } else {
            let datadir = pg_config.data_dir()?;
            let bindir = pg_config.bin_dir()?;
            if !datadir.exists() {
                initdb(&bindir, &datadir)?;
            }
        }
    }

    write_config(output_configs, init)?;
    Ok(())
}

#[tracing::instrument(level = "error", skip_all, fields(og_version = %pg_config.version()?, ogx_home))]
fn download_install_opengauss(pg_config: &PgConfig, ogx_home: &PathBuf) -> eyre::Result<PgConfig> {
    let mut ogvdir = ogx_home.clone(); 
    ogvdir.push(format!("{}.{}.{}", pg_config.major_version()?, pg_config.middle_version()?, pg_config.minor_version()?));
    if ogvdir.exists() {
        // delete everything at this path if it already exists
        println!("{} {}", "  Removing".bold().green(), ogvdir.display());
        std::fs::remove_dir_all(&ogvdir)?;
    }
    
    download_opengauss_third(pg_config, &ogvdir)?;
    let ogdir = download_opengauss_server(pg_config, &ogvdir)?;
    adapt_openeuler_2203(&ogdir, pg_config)?;
    configure_opengauss(pg_config, &ogdir)?;
    make_opengauss(pg_config, &ogdir)?;
    make_install_opengauss(pg_config, &ogdir) // returns a new PgConfig object
}

#[tracing::instrument(level = "error", skip_all, fields(og_version = %pg_config.version()?, ogx_home))]
fn download_opengauss_third(pg_config: &PgConfig, ogdir: &PathBuf) -> eyre::Result<PathBuf> {
    use env_proxy::for_url_str;
    use ureq::{Agent, AgentBuilder, Proxy};

    println!(
        "{} openGauss v{}.{}.{} third party from {}",
        "  Downloading".bold().green(),
        pg_config.major_version()?,
        pg_config.middle_version()?,
        pg_config.minor_version()?,
        pg_config.third().expect("no url"),
    );
    let url = pg_config.third().expect("no third url for pg_config").as_str();
    tracing::debug!(url = %url, "Fetching");
    let http_client = if let Some((host, port)) =
        for_url_str(pg_config.url().expect("no url for pg_config")).host_port()
    {
        AgentBuilder::new().proxy(Proxy::new(format!("https://{host}:{port}"))?).build()
    } else {
        Agent::new()
    };
    let http_response = http_client.get(url).call()?;
    let status = http_response.status();
    tracing::trace!(status_code = %status, url = %url, "Fetched");
    if status != 200 {
        return Err(eyre!(
            "Problem downloading {}:\ncode={status}\n{}",
            pg_config.third().unwrap().to_string().yellow().bold(),
            http_response.into_string()?
        ));
    }
    let mut buf = Vec::new();
    let _count = http_response.into_reader().read_to_end(&mut buf)?;
    untar(&buf, &ogdir, pg_config)
}

#[tracing::instrument(level = "error", skip_all, fields(og_version = %pg_config.version()?, ogx_home))]
fn download_opengauss_server(pg_config: &PgConfig, ogx_home: &PathBuf) -> eyre::Result<PathBuf> {
    println!(
        "{} openGauss v{}.{}.{} from {}",
        "  Downloading".bold().green(),
        pg_config.major_version()?,
        pg_config.middle_version()?,
        pg_config.minor_version()?,
        pg_config.url().expect("no url"),
    );
    let url = pg_config.url().expect("no url for pg_config").as_str();
    tracing::debug!(url = %url, "Fetching");

    let mut zipfile = ogx_home.clone();
    zipfile.push("server.zip");
    if zipfile.exists() {
        // delete everything at this path if it already exists
        println!("{} {}", "     Removing".bold().green(), zipfile.display());
        std::fs::remove_file(&zipfile)?;
    }

    let child = std::process::Command::new("wget")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .arg(&url)
    .arg("-O")
    .arg(&zipfile)
    .spawn()
    .wrap_err("failed to spawn `unzip`")?;
    
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?));
    }

    unzip(&zipfile, &ogx_home, pg_config)
}

fn adapt_openeuler_2203(ogxdir: &PathBuf, _pg_config: &PgConfig) -> eyre::Result<()>  {
    if is_openeuler_2203() {
        // println!(
        //     "{} openGauss v{}.{}.{} to openEuler 22.03",
        //     "  Adapting".bold().green(),
        //     pg_config.major_version()?,
        //     pg_config.middle_version()?,
        //     pg_config.minor_version()?,
        // );
    
        let mut interface_file = ogxdir.clone();
        interface_file.push("src/include/communication/commproxy_interface.h");
        let child = std::process::Command::new("sed")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("-i")
        .arg("/extern int gettimeofday/d")
        .arg(&interface_file)
        .spawn()
        .wrap_err("failed to spawn `sed`")?;
    
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?));
        }
    
        let mut syscall_support = ogxdir.clone();
        syscall_support.push("src/gausskernel/cbb/bbox/bbox_syscall_support.h");
        let child = std::process::Command::new("sed")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-i")
            .arg(r##"s/sys\/sysctl.h/linux\/sysctl.h/g"##)
            .arg(&syscall_support)
            .spawn()
            .wrap_err("failed to spawn `sed`")?;
    
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?));
        }
    
        let mut elf_dump = ogxdir.clone();
        elf_dump.push("src/gausskernel/cbb/bbox/bbox_elf_dump.h");
        let child = std::process::Command::new("sed")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-i")
            .arg(r##"s/sys\/sysctl.h/linux\/sysctl.h/g"##)
            .arg(&elf_dump)
            .spawn()
            .wrap_err("failed to spawn `sed`")?;
    
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?));
        }
    
        let mut bbox_threads = ogxdir.clone();
        bbox_threads.push("src/gausskernel/cbb/bbox/bbox_threads.h");
        let child = std::process::Command::new("sed")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-i")
            .arg("/#define BBOX_PROC_PATH_LEN 128/a#define MINSIGSTKSZ 512")
            .arg(&bbox_threads)
            .spawn()
            .wrap_err("failed to spawn `sed`")?;
    
        let output = child.wait_with_output()?;
        if output.status.success() {
            Ok(())
        } else {
            Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?))
        }
    } else {
        Ok(())
    }
}

fn untar(bytes: &[u8], ogxdir: &PathBuf, pg_config: &PgConfig) -> eyre::Result<PathBuf> {
    let mut ogdir = ogxdir.clone();
    ogdir.push("tools");
    if ogdir.exists() {
        // delete everything at this path if it already exists
        println!("{} {}", "     Removing".bold().green(), ogdir.display());
        std::fs::remove_dir_all(&ogdir)?;
    }
    std::fs::create_dir_all(&ogdir)?;
    println!(
        "{} openGauss third party v{}.{}.{} to {}",
        "    Untarring".bold().green(),
        pg_config.major_version()?,
        pg_config.middle_version()?,
        pg_config.minor_version()?,
        ogdir.display()
    );
    let mut child = std::process::Command::new("tar")
        .arg("-C")
        .arg(&ogdir)
        .arg("--strip-components=1")
        .arg("-zxf")
        .arg("-")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .wrap_err("failed to spawn `tar`")?;

    let stdin = child.stdin.as_mut().expect("failed to get `tar`'s stdin");
    stdin.write_all(bytes)?;
    stdin.flush()?;
    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(ogdir)
    } else {
        Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?))
    }
}

fn unzip(zipfile: &PathBuf, ogxdir: &PathBuf, pg_config: &PgConfig) -> eyre::Result<PathBuf> {
    let mut ogdir = ogxdir.clone();
    ogdir.push(format!("openGauss-server-v{}.{}.{}", pg_config.major_version()?, pg_config.middle_version()?, pg_config.minor_version()?));
    if ogdir.exists() {
        // delete everything at this path if it already exists
        println!("{} {}", "     Removing".bold().green(), ogdir.display());
        std::fs::remove_dir_all(&ogdir)?;
    }
    std::fs::create_dir_all(&ogdir)?;

    println!(
        "{} openGauss server.zip v{}.{}.{} to {}",
        "    Unzipping".bold().green(),
        pg_config.major_version()?,
        pg_config.middle_version()?,
        pg_config.minor_version()?,
        ogdir.display()
    );

    let child = std::process::Command::new("unzip")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&zipfile)
        .arg("-d")
        .arg(&ogxdir)
        .spawn()
        .wrap_err("failed to spawn `unzip`")?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        std::fs::remove_file(&zipfile)?;
        Ok(ogdir)
    } else {
        Err(eyre!("Command error: {}", String::from_utf8(output.stderr)?))
    }
}

fn configure_opengauss(pg_config: &PgConfig, ogdir: &PathBuf) -> eyre::Result<()> {
    println!(
        "{} openGauss v{}.{}.{}",
        "  Configuring".bold().green(),
        pg_config.major_version()?,
        pg_config.middle_version()?,
        pg_config.minor_version()?
    );
    let mut configure_path = ogdir.clone();
    configure_path.push("configure");
    let mut gcc_path = get_opengauss_toolsdir(&ogdir);
    gcc_path.push("buildtools");
    gcc_path.push("gcc7.3");

    let mut command = std::process::Command::new(configure_path);
    command
        .arg("--gcc-version=7.3.0")
        .arg("CC=g++")
        .arg(format!("CFLAGS={}", "-O0 -g"))
        .arg(format!("--prefix={}", get_opengauss_installdir(&ogdir).display()))
        .arg(format!("--with-pgport={}", pg_config.port()?))
        .arg(format!("--3rd={}", get_opengauss_toolsdir(&ogdir).display()))
        .arg("--enable-debug")
        .arg("--enable-cassert")
        .arg("--enable-thread-safety")
        .arg("--without-readline")
        .arg("--without-zlib")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .env("GCC_PATH", &gcc_path)
        .env("CC", format!("{}/gcc/bin/gcc", &gcc_path.display()))
        .env("CXX", format!("{}/gcc/bin/g++", &gcc_path.display()))
        .env("LD_LIBRARY_PATH", format!("{}/gcc/lib64:{}/isl/lib:{}/mpc/lib/:{}/mpfr/lib/:{}/gmp/lib/", 
             &gcc_path.display(), &gcc_path.display(), &gcc_path.display(), &gcc_path.display(), &gcc_path.display()))
        .env("PATH", prefix_path(format!("{}/gcc/bin", &gcc_path.display())))
        .current_dir(&ogdir);
    for var in PROCESS_ENV_DENYLIST {
        command.env_remove(var);
    }

    let command_str = format!("{:?}", command);
    tracing::debug!(command = %command_str, "Running");
    let child = command.spawn()?;
    let output = child.wait_with_output()?;
    tracing::trace!(status_code = %output.status, command = %command_str, "Finished");

    if output.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "{}\n{}{}",
                command_str,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            ),
        ))?
    }
}

fn make_opengauss(pg_config: &PgConfig, ogdir: &PathBuf) -> eyre::Result<()> {
    let num_cpus = 1.max(num_cpus::get() / 3);
    println!(
        "{} openGauss v{}.{}.{}",
        "  Compiling".bold().green(),
        pg_config.major_version()?,
        pg_config.middle_version()?,
        pg_config.minor_version()?
    );

    let mut gcc_path = get_opengauss_toolsdir(&ogdir);
    gcc_path.push("buildtools");
    gcc_path.push("gcc7.3");

    let mut command = std::process::Command::new("make");
    command
        .arg("-s")
        .arg("-j")
        .arg(num_cpus.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .env("GCC_PATH", &gcc_path)
        .env("CC", format!("{}/gcc/bin/gcc", &gcc_path.display()))
        .env("CXX", format!("{}/gcc/bin/g++", &gcc_path.display()))
        .env("LD_LIBRARY_PATH", format!("{}/gcc/lib64:{}/isl/lib:{}/mpc/lib/:{}/mpfr/lib/:{}/gmp/lib/", 
             &gcc_path.display(), &gcc_path.display(), &gcc_path.display(), &gcc_path.display(), &gcc_path.display()))
        .env("PATH", prefix_path(format!("{}/gcc/bin", &gcc_path.display())))
        .current_dir(&ogdir);

    for var in PROCESS_ENV_DENYLIST {
        command.env_remove(var);
    }

    let command_str = format!("{:?}", command);
    tracing::debug!(command = %command_str, "Running");
    let child = command.spawn()?;
    let output = child.wait_with_output()?;
    tracing::trace!(status_code = %output.status, command = %command_str, "Finished");

    if output.status.success() {
        Ok(())
    } else {
        Err(eyre!(
            "{}\n{}{}",
            command_str,
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        ))
    }
}

fn make_install_opengauss(version: &PgConfig, ogdir: &PathBuf) -> eyre::Result<PgConfig> {
    let num_cpus = 1.max(num_cpus::get() / 3);
    println!(
        "{} openGauss v{}.{}.{} to {}",
        "  Installing".bold().green(),
        version.major_version()?,
        version.middle_version()?,
        version.minor_version()?,
        get_opengauss_installdir(&ogdir).display()
    );
    let mut command = std::process::Command::new("make");

    command
        .arg("install")
        .arg("-s")
        .arg("-j")
        .arg(num_cpus.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .current_dir(&ogdir);
    for var in PROCESS_ENV_DENYLIST {
        command.env_remove(var);
    }

    let command_str = format!("{:?}", command);
    tracing::debug!(command = %command_str, "Running");
    let child = command.spawn()?;
    let output = child.wait_with_output()?;
    tracing::trace!(status_code = %output.status, command = %command_str, "Finished");

    if output.status.success() {
        let mut pg_config = get_opengauss_installdir(ogdir);
        pg_config.push("bin");
        pg_config.push("pg_config");
        Ok(PgConfig::new_with_defaults(pg_config))
    } else {
        Err(eyre!(
            "{}\n{}{}",
            command_str,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        ))
    }
}

fn validate_pg_config(pg_config: &PgConfig) -> eyre::Result<()> {
    println!(
        "{} {}",
        "   Validating".bold().green(),
        pg_config.path().expect("no path for pg_config").display()
    );

    pg_config.includedir_server()?;
    pg_config.pkglibdir()?;
    Ok(())
}

fn write_config(pg_configs: &Vec<PgConfig>, init: &Init) -> eyre::Result<()> {
    let config_path = Ogx::config_toml()?;
    let mut file = File::create(&config_path)?;

    if let Some(port) = init.base_port {
        file.write_all(format!("base_port = {}\n", port).as_bytes())?;
    }
    if let Some(port) = init.base_testing_port {
        file.write_all(format!("base_testing_port = {}\n", port).as_bytes())?;
    }

    file.write_all(b"[configs]\n")?;
    for pg_config in pg_configs {
        file.write_all(
            format!(
                "{}=\"{}\"\n",
                pg_config.label()?,
                pg_config.path().ok_or(eyre!("no path for pg_config"))?.display()
            )
            .as_bytes(),
        )?;
    }

    Ok(())
}

fn get_opengauss_installdir(ogdir: &PathBuf) -> PathBuf {
    let mut dir = PathBuf::from(ogdir);
    dir.pop();
    dir.push("ogx-install");
    dir
}

fn get_opengauss_toolsdir(ogdir: &PathBuf) -> PathBuf {
    let mut dir = PathBuf::from(ogdir);
    dir.pop();
    dir.push("tools");
    dir
}

fn is_openeuler_2203() -> bool {
    let s = System::new();
    //println!("  {} info: os_name={:?}, os_version={:?}", 
    //    "Current OS".bold().green(), s.distribution_id(), s.os_version().unwrap());
    match s.os_version() {
        Some(val) => val == "22.03",
        _ => false,
    }
}

fn is_root_user() -> bool {
    match env::var("USER") {
        Ok(val) => val == "root",
        Err(_) => false,
    }
}

pub(crate) fn initdb(bindir: &PathBuf, datadir: &PathBuf) -> eyre::Result<()> {
    println!(" {} data directory at {}", "Initializing".bold().green(), datadir.display());
    let mut libpath = bindir.clone();
    libpath.pop();
    libpath.push("lib");

    let mut command = std::process::Command::new(format!("{}/gs_initdb", bindir.display()));
    command.stdout(Stdio::piped()).stderr(Stdio::piped())
        .arg("-D").arg(&datadir)
        .arg("-w").arg("openGauss2022")
        .arg("-E").arg("utf8")
        .arg(format!("--nodename={}", "datanode"))
        .env("PATH", prefix_path(bindir))
        .env("LD_LIBRARY_PATH", &libpath);

    let command_str = format!("{:?}", command);
    tracing::debug!(command = %command_str, "Running");

    let output = command.output().wrap_err_with(|| eyre!("unable to execute: {}", command_str))?;
    tracing::trace!(command = %command_str, status_code = %output.status, "Finished");

    if !output.status.success() {
        return Err(eyre!(
            "problem running gs_initdb: {}\n{}",
            command_str,
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    Ok(())
}
