use anyhow::Result;
use serde::Deserialize;

pub trait PackageManager {
    fn get_latest_version_value(package: &str) -> Result<String>;
}

#[derive(Deserialize)]
struct Tags {
    latest: String,
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
        Ok(registry_object.tags.latest)
    }
}
