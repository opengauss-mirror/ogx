/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use crate::command::get::get_property;
use crate::command::run::exec_gsql;
use crate::command::start::start_opengauss;
use crate::CommandExecute;
use cargo_toml::Manifest;
use eyre::{eyre, WrapErr};
use owo_colors::OwoColorize;
use ogx_pg_config::{createdb, PgConfig, Ogx};
use std::path::PathBuf;

/// Connect, via gsql, to a openGauss instance
#[derive(clap::Args, Debug)]
#[clap(author)]
pub(crate) struct Connect {
    /// Do you want to run against openGauss `og3`?
    #[clap(env = "OG_VERSION")]
    og_version: Option<String>,
    /// The database to connect to (and create if the first time).  Defaults to a database with the same name as the current extension name
    #[clap(env = "DBNAME")]
    dbname: Option<String>,
    #[clap(from_global, parse(from_occurrences))]
    verbose: usize,
    /// Package to determine default `og_version` with (see `cargo help pkgid`)
    #[clap(long, short)]
    package: Option<String>,
    /// Path to Cargo.toml
    #[clap(long, parse(from_os_str))]
    manifest_path: Option<PathBuf>,
    /// Use an existing `pgcli` on the $PATH.
    #[clap(env = "OGX_PGCLI", long)]
    pgcli: bool,
}

impl CommandExecute for Connect {
    #[tracing::instrument(level = "error", skip(self))]
    fn execute(mut self) -> eyre::Result<()> {
        let ogx = Ogx::from_config()?;

        let og_version = match self.og_version {
            Some(og_version) => match ogx.get(&og_version) {
                Ok(_) => og_version,
                Err(err) => {
                    if self.dbname.is_some() {
                        return Err(err);
                    }
                    // It's actually the dbname! We should infer from the manifest.
                    self.dbname = Some(og_version);

                    let metadata =
                        crate::metadata::metadata(&Default::default(), self.manifest_path.as_ref())
                            .wrap_err("couldn't get cargo metadata")?;
                    crate::metadata::validate(&metadata)?;
                    let package_manifest_path =
                        crate::manifest::manifest_path(&metadata, self.package.as_ref())
                            .wrap_err("Couldn't get manifest path")?;
                    let package_manifest = Manifest::from_path(&package_manifest_path)
                        .wrap_err("Couldn't parse manifest")?;

                    let default_og_version = crate::manifest::default_og_version(&package_manifest)
                        .ok_or(eyre!("no provided `og$VERSION` flag."))?;
                    default_og_version
                }
            },
            None => {
                // We should infer from the manifest.
                println!("og_version is empty, parse it from metadata");
                let metadata =
                    crate::metadata::metadata(&Default::default(), self.manifest_path.as_ref())
                        .wrap_err("couldn't get cargo metadata")?;
                crate::metadata::validate(&metadata)?;
                let package_manifest_path =
                    crate::manifest::manifest_path(&metadata, self.package.as_ref())
                        .wrap_err("Couldn't get manifest path")?;
                let package_manifest = Manifest::from_path(&package_manifest_path)
                    .wrap_err("Couldn't parse manifest")?;

                let default_og_version = crate::manifest::default_og_version(&package_manifest)
                    .ok_or(eyre!("no provided `og$VERSION` flag."))?;
                default_og_version
            }
        };

        let dbname = match self.dbname {
            Some(dbname) => dbname,
            None => {
                // We should infer from package
                let metadata =
                    crate::metadata::metadata(&Default::default(), self.manifest_path.as_ref())
                        .wrap_err("couldn't get cargo metadata")?;
                crate::metadata::validate(&metadata)?;
                let package_manifest_path =
                    crate::manifest::manifest_path(&metadata, self.package.as_ref())
                        .wrap_err("Couldn't get manifest path")?;

                get_property(&package_manifest_path, "extname")
                    .wrap_err("could not determine extension name")?
                    .ok_or(eyre!("extname not found in control file"))?
            }
        };

        connect_gsql(Ogx::from_config()?.get(&og_version)?, &dbname, self.pgcli)
    }
}

#[tracing::instrument(level = "error", skip_all, fields(
    og_version = %pg_config.version()?,
    dbname,
))]
pub(crate) fn connect_gsql(pg_config: &PgConfig, dbname: &str, pgcli: bool) -> eyre::Result<()> {
    // restart openGauss
    start_opengauss(pg_config)?;

    // create the named database
    if !createdb(pg_config, dbname, false, true)? {
        println!("{} existing database {}", "    Re-using".bold().cyan(), dbname);
    }

    // run psql
    exec_gsql(&pg_config, dbname, pgcli)
}
