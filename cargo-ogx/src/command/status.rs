/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use eyre::eyre;
use owo_colors::OwoColorize;
use ogx_pg_config::{PgConfig, PgConfigSelector, Ogx};
use std::path::PathBuf;
use std::process::Stdio;

use crate::CommandExecute;

/// Is a ogx-managed openGauss instance running?
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct Status {
    /// The openGauss version
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

impl CommandExecute for Status {
    #[tracing::instrument(level = "error", skip(self))]
    fn execute(self) -> eyre::Result<()> {
        let ogx = Ogx::from_config()?;

        let pg_version = match self.pg_version {
            Some(s) => s,
            None => "all".to_string(),
        };

        for pg_config in ogx.iter(PgConfigSelector::new(&pg_version)) {
            let pg_config = pg_config?;
            if status_opengauss(pg_config)? {
                println!("openGauss v{} is {}", pg_config.major_version()?, "running".bold().green())
            } else {
                println!("openGauss v{} is {}", pg_config.major_version()?, "stopped".bold().red())
            }
        }

        Ok(())
    }
}

#[tracing::instrument(level = "error", skip_all, fields(pg_version = %pg_config.version()?))]
pub(crate) fn status_opengauss(pg_config: &PgConfig) -> eyre::Result<bool> {
    let datadir = pg_config.data_dir()?;
    let bindir = pg_config.bin_dir()?;

    if !datadir.exists() {
        // openGauss couldn't possibly be running if there's no data directory
        // and even if it were, we'd have no way of knowing
        return Ok(false);
    }
    let mut libpath = bindir.clone();
    libpath.pop();
    libpath.push("lib");

    let mut command = std::process::Command::new(format!("{}/gs_ctl", bindir.display()));
    command.stdout(Stdio::piped()).stderr(Stdio::piped())
        .arg("status").arg("-D").arg(&datadir)
        .env("LD_LIBRARY_PATH", &libpath);
    let command_str = format!("{:?}", command);
    tracing::debug!(command = %command_str, "Running");

    let output = command.output()?;
    let code = output.status.code().unwrap();
    tracing::trace!(status_code = %code, command = %command_str, "Finished");

    let is_running = code == 0; // running
    let is_stopped = code == 3; // not running

    if !is_running && !is_stopped {
        return Err(eyre!(
            "problem running gs_ctl: {}\n\n{}",
            command_str,
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    // a status code of zero means it's running
    Ok(is_running)
}
