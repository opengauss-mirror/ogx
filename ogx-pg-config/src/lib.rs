/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
//! Wrapper around Postgres' `pg_config` command-line tool
use eyre::{eyre, WrapErr};
use owo_colors::OwoColorize;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use url::Url;

pub static BASE_OPENGAUSS_PORT_NO: u16 = 28800;
pub static BASE_OPENGAUSS_TESTING_PORT_NO: u16 = 32200;

mod path_methods;
pub use path_methods::{get_target_dir, prefix_path};

#[derive(Clone)]
pub struct OgVersion {
    major: u16,
    middle: u16,
    minor: u16,
    url: Url,
    third: Url,
}

impl OgVersion {
    pub fn new(major: u16, middle: u16, minor: u16, url: Url, third: Url) -> OgVersion {
        OgVersion { major, middle, minor, url, third }
    }
}

impl Display for OgVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.middle, self.minor)
    }
}

#[derive(Clone)]
pub struct PgConfig {
    version: Option<OgVersion>,
    pg_config: Option<PathBuf>,
    base_port: u16,
    base_testing_port: u16,
}

impl Display for PgConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let major = self.major_version().expect("could not determine major version");
        let middle = self.middle_version().expect("could not determine middle version");
        let minor = self.minor_version().expect("could not determine minor version");
        let path = match self.pg_config.as_ref() {
            Some(path) => path.display().to_string(),
            None => self.version.as_ref().unwrap().url.to_string(),
        };
        write!(f, "{}.{}.{}={}", major, middle, minor, path)
    }
}

impl Default for PgConfig {
    fn default() -> Self {
        PgConfig {
            version: None,
            pg_config: None,
            base_port: BASE_OPENGAUSS_PORT_NO,
            base_testing_port: BASE_OPENGAUSS_TESTING_PORT_NO,
        }
    }
}

impl From<OgVersion> for PgConfig {
    fn from(version: OgVersion) -> Self {
        PgConfig { version: Some(version), pg_config: None, ..Default::default() }
    }
}

impl PgConfig {
    pub fn new(pg_config: PathBuf, base_port: u16, base_testing_port: u16) -> Self {
        PgConfig { version: None, pg_config: Some(pg_config), base_port, base_testing_port }
    }

    pub fn new_with_defaults(pg_config: PathBuf) -> Self {
        PgConfig {
            version: None,
            pg_config: Some(pg_config),
            base_port: BASE_OPENGAUSS_PORT_NO,
            base_testing_port: BASE_OPENGAUSS_TESTING_PORT_NO,
        }
    }

    pub fn from_path() -> Self {
        Self::new_with_defaults("pg_config".into())
    }

    pub fn is_real(&self) -> bool {
        self.pg_config.is_some()
    }

    pub fn label(&self) -> eyre::Result<String> {
        Ok(format!("og{}", self.major_version()?))
    }

    pub fn path(&self) -> Option<PathBuf> {
        self.pg_config.clone()
    }

    pub fn parent_path(&self) -> PathBuf {
        self.path().unwrap().parent().unwrap().to_path_buf()
    }

    pub fn major_version(&self) -> eyre::Result<u16> {
        match &self.version {
            Some(version) => Ok(version.major),
            None => {
                let version_string = self.run("--version")?;
                let version_parts = version_string.split_whitespace().collect::<Vec<&str>>();
                let version = match version_parts.get(3) {
                    Some(v) => v.split('.').nth(0).unwrap(),
                    None => {
                        return Err(eyre!("invalid version string: {}", version_string));
                    }
                };
                let version = match f64::from_str(version) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(eyre!("invalid major version number `{}`: {:?}", version, e));
                    }
                };
                Ok(version.floor() as u16)
            }
        }
    }

    pub fn middle_version(&self) -> eyre::Result<u16> {
        match &self.version {
            Some(version) => Ok(version.middle),
            None => {
                let version_string = self.run("--version")?;
                let version_parts = version_string.split_whitespace().collect::<Vec<&str>>();
                let version = match version_parts.get(3) {
                    Some(v) => v.split('.').nth(1).unwrap(),
                    None => {
                        return Err(eyre!("invalid version string: {}", version_string));
                    }
                };
                let version = match f64::from_str(version) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(eyre!("invalid middle version number `{}`: {:?}", version, e));
                    }
                };
                Ok(version.floor() as u16)
            }
        }
    }

    pub fn minor_version(&self) -> eyre::Result<u16> {
        match &self.version {
            Some(version) => Ok(version.minor),
            None => {
                let version_string = self.run("--version")?;
                let version_parts = version_string.split_whitespace().collect::<Vec<&str>>();
                let version = match version_parts.get(3) {
                    Some(v) => v.split('.').nth(2).unwrap(),
                    None => {
                        return Err(eyre!("invalid version string: {}", version_string));
                    }
                };
                let version = match u16::from_str(version) {
                    Ok(u) => u,
                    Err(e) => {
                        return Err(eyre!("invalid minor version number `{}`: {:?}", version, e));
                    }
                };
                Ok(version)
            }
        }
    }

    pub fn version(&self) -> eyre::Result<String> {
        let major = self.major_version()?;
        let middle = self.middle_version()?;
        let minor = self.minor_version()?;
        let version = format!("{}.{}.{}", major, middle, minor);
        Ok(version)
    }

    pub fn url(&self) -> Option<&Url> {
        match &self.version {
            Some(version) => Some(&version.url),
            None => None,
        }
    }

    pub fn third(&self) -> Option<&Url> {
        match &self.version {
            Some(version) => Some(&version.third),
            None => None,
        }
    }

    pub fn port(&self) -> eyre::Result<u16> {
        Ok(self.base_port + self.major_version()?)
    }

    pub fn test_port(&self) -> eyre::Result<u16> {
        Ok(self.base_testing_port + self.major_version()?)
    }

    pub fn host(&self) -> &'static str {
        "localhost"
    }

    pub fn bin_dir(&self) -> eyre::Result<PathBuf> {
        Ok(Path::new(&self.run("--bindir")?).to_path_buf())
    }

    pub fn postmaster_path(&self) -> eyre::Result<PathBuf> {
        let mut path = self.bin_dir()?;
        path.push("postmaster");
        Ok(path)
    }

    pub fn initdb_path(&self) -> eyre::Result<PathBuf> {
        let mut path = self.bin_dir()?;
        path.push("gs_initdb");
        Ok(path)
    }

    pub fn gsql_path(&self) -> eyre::Result<PathBuf> {
        let mut path = self.bin_dir()?;
        path.push("gsql");
        Ok(path)
    }

    pub fn data_dir(&self) -> eyre::Result<PathBuf> {
        let mut path = Ogx::home()?;
        path.push(format!("data-{}", self.major_version()?));
        Ok(path)
    }

    pub fn gcc_include_dir(&self) -> eyre::Result<PathBuf> {
        let mut path = self.bin_dir()?;
        path.pop();
        path.pop();
        path.push("tools/buildtools/gcc7.3/gcc/include");
        Ok(path)
    }

    pub fn log_file(&self) -> eyre::Result<PathBuf> {
        let mut path = Ogx::home()?;
        path.push(format!("{}.log", self.major_version()?));
        Ok(path)
    }

    pub fn includedir_server(&self) -> eyre::Result<PathBuf> {
        Ok(self.run("--includedir-server")?.into())
    }

    pub fn pkglibdir(&self) -> eyre::Result<PathBuf> {
        Ok(self.run("--pkglibdir")?.into())
    }

    pub fn sharedir(&self) -> eyre::Result<PathBuf> {
        Ok(self.run("--sharedir")?.into())
    }

    pub fn cppflags(&self) -> eyre::Result<OsString> {
        Ok(self.run("--cppflags")?.into())
    }

    pub fn extension_dir(&self) -> eyre::Result<PathBuf> {
        let mut path = self.sharedir()?;
        path.push("extension");
        Ok(path)
    }

    fn run(&self, arg: &str) -> eyre::Result<String> {
        let pg_config = self.pg_config.clone().unwrap_or_else(|| {
            std::env::var("PG_CONFIG").unwrap_or_else(|_| "pg_config".to_string()).into()
        });
        let mut gauss_home = pg_config.clone();
        gauss_home.pop();
        gauss_home.pop();

        match Command::new(&pg_config).arg(arg).env("GAUSSHOME", &gauss_home).output() {
            Ok(output) => Ok(String::from_utf8(output.stdout).unwrap().trim().to_string()),
            Err(e) => match e.kind() {
                ErrorKind::NotFound => {
                    Err(e).wrap_err_with(|| format!("Unable to find `{}`", "pg_config".yellow()))
                }
                _ => Err(e.into()),
            },
        }
    }
}

pub struct Ogx {
    pg_configs: Vec<PgConfig>,
    base_port: u16,
    base_testing_port: u16,
}

impl Default for Ogx {
    fn default() -> Self {
        Self {
            pg_configs: vec![],
            base_port: BASE_OPENGAUSS_PORT_NO,
            base_testing_port: BASE_OPENGAUSS_TESTING_PORT_NO,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigToml {
    configs: HashMap<String, PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_testing_port: Option<u16>,
}

pub enum PgConfigSelector<'a> {
    All,
    Specific(&'a str),
}

impl<'a> PgConfigSelector<'a> {
    pub fn new(label: &'a str) -> Self {
        if label == "all" {
            PgConfigSelector::All
        } else {
            PgConfigSelector::Specific(label)
        }
    }
}

impl Ogx {
    pub fn new(base_port: u16, base_testing_port: u16) -> Self {
        Ogx { pg_configs: vec![], base_port, base_testing_port }
    }

    pub fn from_config() -> eyre::Result<Self> {
        match std::env::var("OGX_PG_CONFIG_PATH") {
            Ok(pg_config) => {
                // we have an environment variable that tells us the pg_config to use
                let mut ogx = Ogx::default();
                ogx.push(PgConfig::new(pg_config.into(), ogx.base_port, ogx.base_testing_port));
                Ok(ogx)
            }
            Err(_) => {
                // we'll get what we need from cargo-ogx' config.toml file
                let path = Ogx::config_toml()?;
                if !path.exists() {
                    return Err(eyre!(
                        "{} not found.  Have you run `{}` yet?",
                        path.display(),
                        "cargo ogx init".bold().yellow()
                    ));
                }

                match toml::from_str::<ConfigToml>(&std::fs::read_to_string(&path)?) {
                    Ok(configs) => {
                        let mut ogx = Ogx::new(
                            configs.base_port.unwrap_or(BASE_OPENGAUSS_PORT_NO),
                            configs.base_testing_port.unwrap_or(BASE_OPENGAUSS_TESTING_PORT_NO),
                        );

                        for (_, v) in configs.configs {
                            ogx.push(PgConfig::new(v, ogx.base_port, ogx.base_testing_port));
                        }
                        Ok(ogx)
                    }
                    Err(e) => {
                        Err(e).wrap_err_with(|| format!("Could not read `{}`", path.display()))
                    }
                }
            }
        }
    }

    pub fn push(&mut self, pg_config: PgConfig) {
        self.pg_configs.push(pg_config);
    }

    pub fn iter(
        &self,
        which: PgConfigSelector,
    ) -> impl std::iter::Iterator<Item = eyre::Result<&PgConfig>> {
        match which {
            PgConfigSelector::All => {
                let mut configs = self.pg_configs.iter().collect::<Vec<_>>();
                configs.sort_by(|a, b| {
                    a.major_version()
                        .expect("no major version")
                        .cmp(&b.major_version().expect("no major version"))
                });

                configs.into_iter().map(|c| Ok(c)).collect::<Vec<_>>().into_iter()
            }
            PgConfigSelector::Specific(label) => vec![self.get(label)].into_iter(),
        }
    }

    pub fn get(&self, label: &str) -> eyre::Result<&PgConfig> {
        for pg_config in self.pg_configs.iter() {
            if pg_config.label()? == label {
                return Ok(pg_config);
            }
        }
        Err(eyre!("openGauss `{}` is not managed by ogx", label))
    }

    pub fn home() -> Result<PathBuf, std::io::Error> {
        std::env::var("OGX_HOME").map_or_else(
            |_| {
                let mut dir = match dirs::home_dir() {
                    Some(dir) => dir,
                    None => {
                        return Err(std::io::Error::new(
                            ErrorKind::NotFound,
                            "You don't seem to have a home directory",
                        ));
                    }
                };
                dir.push(".ogx");
                if !dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(&dir) {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidInput,
                            format!("could not create OGX_HOME at `{}`: {:?}", dir.display(), e),
                        ));
                    }
                }

                Ok(dir)
            },
            |v| Ok(v.into()),
        )
    }

    /// Get the postmaster stub directory
    ///
    /// We isolate postmaster stubs to an independent directory instead of alongside the postmaster
    /// because in the case of `cargo ogx install` the `pg_config` may not necessarily be one managed
    /// by ogx.
    pub fn postmaster_stub_dir() -> Result<PathBuf, std::io::Error> {
        let mut stub_dir = Self::home()?;
        stub_dir.push("gauss_stubs");
        Ok(stub_dir)
    }

    pub fn config_toml() -> Result<PathBuf, std::io::Error> {
        let mut path = Ogx::home()?;
        path.push("config.toml");
        Ok(path)
    }
}

pub const SUPPORTED_MAJOR_VERSIONS: &[u16] = &[3];

pub fn createdb(
    pg_config: &PgConfig,
    dbname: &str,
    is_test: bool,
    if_not_exists: bool,
) -> eyre::Result<bool> {
    if if_not_exists && does_db_exist(pg_config, dbname)? {
        return Ok(false);
    }

    println!("{} database {}", "     Creating".bold().green(), dbname);
    let mut command = Command::new(pg_config.gsql_path()?);
    command
        .env_remove("PGDATABASE")
        .env_remove("PGHOST")
        .env_remove("PGPORT")
        .env_remove("PGUSER")
        .arg("-h")
        .arg(pg_config.host())
        .arg("-p")
        .arg(if is_test {
            pg_config.test_port()?.to_string()
        } else {
            pg_config.port()?.to_string()
        })
        .arg("-c 'create database ")
        .arg(dbname)
        .arg("'")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let command_str = format!("{:?}", command);

    let child = command.spawn().wrap_err_with(|| {
        format!("Failed to spawn process for creating database using command: '{command_str}': ")
    })?;

    let output = child.wait_with_output().wrap_err_with(|| {
        format!(
            "failed waiting for spawned process to create database using command: '{command_str}': "
        )
    })?;

    if !output.status.success() {
        return Err(eyre!(
            "problem running createdb: {}\n\n{}{}",
            command_str,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        ));
    }

    Ok(true)
}

fn does_db_exist(pg_config: &PgConfig, dbname: &str) -> eyre::Result<bool> {
    let mut command = Command::new(pg_config.gsql_path()?);
    command
        .arg("-XqAt")
        .env_remove("PGUSER")
        .arg("-h")
        .arg(pg_config.host())
        .arg("-p")
        .arg(pg_config.port()?.to_string())
        .arg("template1")
        .arg("-c")
        .arg(&format!(
            "select count(*) from pg_database where datname = '{}';",
            dbname.replace("'", "''")
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let command_str = format!("{:?}", command);
    let output = command.output()?;

    if !output.status.success() {
        return Err(eyre!(
            "problem checking if database '{}' exists: {}\n\n{}{}",
            dbname,
            command_str,
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        ));
    } else {
        let count = i32::from_str(&String::from_utf8(output.stdout).unwrap().trim())
            .wrap_err("result is not a number")?;
        Ok(count > 0)
    }
}
