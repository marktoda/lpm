use std::cmp;
use anyhow::Result;
use log::info;
use serde::Deserialize;
use semver::Version;

pub trait PackageManager {
    fn get_latest_version_value(package: &str) -> Result<String>;
}

#[derive(Deserialize)]
struct Tags {
    latest: Option<String>,
    rc: Option<String>,
}

#[derive(Deserialize)]
struct NpmRegistryObject {
    #[serde(rename = "dist-tags")]
    tags: Tags,
}

pub struct Npm;

impl PackageManager for Npm {
    fn get_latest_version_value(package: &str) -> Result<String> {
        // TODO Find a better api url
        let body =
            reqwest::blocking::get(format!("https://registry.npmjs.org/{}", package).as_str())?
                .text()?;

        let registry_object: NpmRegistryObject = serde_json::from_reader(body.as_bytes())?;

        let latest = registry_object.tags.latest
            .map(|v| Version::parse(v.as_str()))
            .unwrap_or(Ok(Version::new(0, 0, 0)))
            .unwrap_or(Version::new(0, 0, 0));

        let rc = registry_object.tags.rc
            .map(|v| Version::parse(v.as_str()))
            .unwrap_or(Ok(Version::new(0, 0, 0)))
            .unwrap_or(Version::new(0, 0, 0));

        let newest = cmp::max(latest, rc);
        info!("Found latest version: {} for {}", newest, package);

        Ok(newest.to_string())
    }
}
