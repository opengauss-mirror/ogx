/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use crate::command::status::status_opengauss;
use crate::CommandExecute;
use cargo_toml::Manifest;
use eyre::{eyre, WrapErr};
use owo_colors::OwoColorize;
use ogx_pg_config::{PgConfig, PgConfigSelector, Ogx};
use std::path::PathBuf;
use std::process::Stdio;

/// Stop a ogx-managed openGauss instance
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct Stop {
    /// The openGauss version to stop (`og3`)
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

impl CommandExecute for Stop {
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
            stop_opengauss(pg_config)?
        }

        Ok(())
    }
}

#[tracing::instrument(level = "error", skip_all, fields(pg_version = %pg_config.version()?))]
pub(crate) fn stop_opengauss(pg_config: &PgConfig) -> eyre::Result<()> {
    Ogx::home()?;
    let datadir = pg_config.data_dir()?;
    let bindir = pg_config.bin_dir()?;

    if status_opengauss(pg_config)? == false {
        // it's not running, no need to stop it
        tracing::debug!("Already stopped");
        return Ok(());
    }

    println!("{} openGauss v{}", "    Stopping".bold().green(), pg_config.major_version()?);
    let mut libpath = bindir.clone();
    libpath.pop();
    libpath.push("lib");
    let mut gauss_home = bindir.clone();
    gauss_home.pop();

    let mut command = std::process::Command::new(format!("{}/gs_ctl", bindir.display()));
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("stop")
        .arg("-m")
        .arg("fast")
        .arg("-D")
        .env("LD_LIBRARY_PATH", &libpath)
        .env("GAUSSHOME", &gauss_home)
        .arg(&datadir);

    let output = command.output()?;

    if !output.status.success() {
        Err(eyre!("{}", String::from_utf8(output.stderr)?,))
    } else {
        Ok(())
    }
}
