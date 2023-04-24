use ogx_pg_config::{PgConfig, Ogx};

pub(crate) fn ogx_default(supported_major_versions: &[u16]) -> eyre::Result<Ogx> {
    let mut ogx = Ogx::default();
    rss::OpenGaussVersion::new(supported_major_versions)?
        .into_iter()
        .for_each(|version| ogx.push(PgConfig::from(version)));

    Ok(ogx)
}

mod rss {
    use owo_colors::OwoColorize;
    use ogx_pg_config::OgVersion;
    use url::Url;

    pub(super) struct OpenGaussVersion;

    impl OpenGaussVersion {
        pub(super) fn new(_supported_major_versions: &[u16]) -> eyre::Result<Vec<OgVersion>> {
            let mut versions = Vec::new();
            versions.push(OgVersion::new(
                3,
                1,
                0,
                Url::parse(&format!("https://gitee.com/opengauss/openGauss-server/repository/archive/v3.1.0.zip")
                ).expect("invalid url"),
                Url::parse(&format!("https://opengauss.obs.cn-south-1.myhuaweicloud.com/3.1.0/binarylibs/openGauss-third_party_binarylibs_openEuler_x86_64.tar.gz")
                ).expect("invalid url")    
            ));

            println!(
                "{} openGauss {}",
                "  Discovered".white().bold(),
                versions.iter().map(|ver| format!("v{ver}")).collect::<Vec<_>>().join(", ")
            );

            Ok(versions)
        }
    }
}
