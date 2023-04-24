/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use cargo_metadata::{Metadata, MetadataCommand};
use eyre::eyre;
use semver::{Version, VersionReq};
use std::path::Path;

pub fn metadata(
    features: &clap_cargo::Features,
    manifest_path: Option<impl AsRef<Path>>,
) -> eyre::Result<Metadata> {
    let mut metadata_command = MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
        metadata_command.manifest_path(manifest_path.as_ref().to_owned());
    }
    features.forward_metadata(&mut metadata_command);
    let metadata = metadata_command.exec()?;
    Ok(metadata)
}

#[tracing::instrument(level = "error", skip_all)]
pub fn validate(metadata: &Metadata) -> eyre::Result<()> {
    let cargo_ogx_version = env!("CARGO_PKG_VERSION");
    let cargo_ogx_version_req = VersionReq::parse(&format!("~{}", cargo_ogx_version))?;

    let ogx_packages = metadata.packages.iter().filter(|package| {
        package.name == "ogx"
            || package.name == "ogx-utils"
            || package.name == "ogx-macros"
            || package.name == "ogx-tests"
    });

    for package in ogx_packages {
        let package_semver = metadata_version_to_semver(package.version.clone());
        if !cargo_ogx_version_req.matches(&package_semver) {
            return Err(eyre!(
                r#"`{}-{}` shouldn't be used with `cargo-ogx-{}`, please use `{} = "~{}"` in your `Cargo.toml`."#,
                package.name,
                package.version,
                cargo_ogx_version,
                package.name,
                cargo_ogx_version,
            ));
        } else {
            tracing::trace!(
                "`{}-{}` is compatible with `cargo-ogx-{}`.",
                package.name,
                package.version,
                cargo_ogx_version,
            )
        }
    }

    Ok(())
}

fn metadata_version_to_semver(metadata_version: cargo_metadata::Version) -> semver::Version {
    Version {
        major: metadata_version.major,
        minor: metadata_version.minor,
        patch: metadata_version.patch,
        pre: metadata_version.pre,
        build: metadata_version.build,
    }
}
