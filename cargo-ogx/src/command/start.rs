/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use crate::command::init::initdb;
use crate::command::status::status_opengauss;
use crate::CommandExecute;
use cargo_toml::Manifest;
use eyre::{eyre, WrapErr};
use owo_colors::OwoColorize;
use ogx_pg_config::{PgConfig, PgConfigSelector, Ogx};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Stdio;

/// Start a ogx-managed openGauss instance
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct Start {
    /// The openGauss version to start (`og3`)
    #[clap(env = "OG_VERSION")]
    pg_version: Option<String>,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
    /// Package to determine default `pg_version` with (see `cargo help pkgid`)
    #[clap(long, short)]
    package: Option<String>,
    /// Path to Cargo.toml
    #[clap(long, parse(from_os_str))]
    manifest_path: Option<PathBuf>,
}

impl CommandExecute for Start {
    #[tracing::instrument(level = "error", skip(self))]
    fn execute(self) -> eyre::Result<()> {
        let ogx = Ogx::from_config()?;

        let pg_version = match self.pg_version {
            Some(s) => s,
            None => {
                let metadata =
                    crate::metadata::metadata(&Default::default(), self.manifest_path.as_ref())
                        .wrap_err("couldn't get cargo metadata")?;
                crate::metadata::validate(&metadata)?;
                let package_manifest_path =
                    crate::manifest::manifest_path(&metadata, self.package.as_ref())
                        .wrap_err("Couldn't get manifest path")?;
                let package_manifest = Manifest::from_path(&package_manifest_path)
                    .wrap_err("Couldn't parse manifest")?;

                crate::manifest::default_og_version(&package_manifest)
                    .ok_or(eyre!("no provided `og$VERSION` flag."))?
            }
        };

        for pg_config in ogx.iter(PgConfigSelector::new(&pg_version)) {
            let pg_config = pg_config?;
            start_opengauss(pg_config)?
        }

        Ok(())
    }
}

#[tracing::instrument(level = "error", skip_all, fields(pg_version = %pg_config.version()?))]
pub(crate) fn start_opengauss(pg_config: &PgConfig) -> eyre::Result<()> {
    let datadir = pg_config.data_dir()?;
    let logfile = pg_config.log_file()?;
    let bindir = pg_config.bin_dir()?;
    let port = pg_config.port()?;

    if !datadir.exists() {
        initdb(&bindir, &datadir)?;
    }

    if status_opengauss(pg_config)? {
        tracing::debug!("Already started");
        return Ok(());
    }

    println!(
        "{} openGauss v{} on port {}",
        "    Starting".bold().green(),
        pg_config.major_version()?,
        port.to_string().bold().cyan()
    );
    let mut command = std::process::Command::new(format!("{}/gs_ctl", bindir.display()));
    let mut libpath = bindir.clone();
    libpath.pop();
    libpath.push("lib");

    let mut gauss_home = bindir.clone();
    gauss_home.pop();
    // Unsafe block is for the pre_exec setsid call below
    //
    // This means that when cargo ogx run dumps a user into psql, pushing ctrl-c will abort
    // the postgres server started by ogx
    unsafe {
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("start")
            .arg(format!("-o -i -p {} -c unix_socket_directory={}", port, Ogx::home()?.display()))
            .arg("-D")
            .arg(&datadir)
            .arg("-Z")
            .arg("single_node")
            .arg("-l")
            .arg(&logfile)
            .env("LD_LIBRARY_PATH", &libpath)
            .env("GAUSSHOME", &gauss_home)
            .pre_exec(|| {
                fork::setsid().expect("setsid call failed for gs_ctl");
                Ok(())
            });
    }

    let command_str = format!("{:?}", command);
    let output = command.output()?;

    if !output.status.success() {
        return Err(eyre!(
            "problem running gs_ctl: {}\n\n{}",
            command_str,
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    Ok(())
}
