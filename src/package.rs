use crate::util::run_basic_command;
use anyhow::Result;
use log::{debug, error, info};
use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

pub struct Package {
    package_json: PackageJson,
    pub path: PathBuf,
}

// TODO use a trait here so different package types can prepare / update differently
impl Package {
    pub fn new(path: PathBuf) -> Package {
        let mut package_json_path = path.clone();
        package_json_path.push("package.json");
        let package_json =
            PackageJson::new(package_json_path).expect("to work");

        Package {
            package_json,
            path,
        }
    }

    pub fn prepare(&self) {
        info!("Preparing typescript package: {}", self.get_name());

        let npm_install_output =
            run_basic_command(format!("npm install --prefix={:?}", self.path).as_str())
                .expect("Failed to install");
        debug!(
            "{} -- {:?} -- for `npm install` on package: {}",
            npm_install_output.status,
            npm_install_output,
            self.get_name()
        );

        let npm_build_output =
            run_basic_command(format!("npm run build --prefix={:?}", self.path).as_str())
                .expect("Failed to build");
        debug!(
            "{} -- {:?} -- for `npm run build` on package: {}",
            npm_build_output.status,
            npm_build_output,
            self.get_name()
        );
    }

    pub fn get_name(&self) -> String {
        self.package_json.name.clone()
    }

    pub fn get_version_value(&self) -> String {
        format!("file:{:?}", self.path)
    }

    pub fn update(&mut self, dependency: &Package) -> bool {
        info!(
            "Updating dependency {:?} for {:?}",
            dependency.get_name(),
            self.get_name()
        );
        if self
            .package_json
            .update(&dependency.get_name(), &dependency.get_version_value())
        {
            self.package_json.write().map_or_else(
                |e| {
                    error!("Received error writing package.json: {:?}", e);
                    false
                },
                |_| true,
            )
        } else {
            false
        }
    }

    pub fn depends_on(&self, dependency: &Package) -> bool {
        self.package_json
            .get(&dependency.get_name())
            .map_or(false, |_| true)
    }
}

struct PackageJson {
    path: PathBuf,
    name: String,
    data: Value,
}

impl PackageJson {
    pub fn new(path: PathBuf) -> Result<PackageJson> {
        debug!("Fetching package.json from {:?}", path);
        let data: Value = serde_json::from_reader(File::open(path.clone())?)?;
        Ok(PackageJson {
            path: path,
            name: data
                .get("name")
                .expect("Package.json to have a name")
                .to_string()
                .replace("\"", ""),
            data,
        })
    }

    fn get_mut(&mut self, package_name: &str) -> Option<&mut String> {
        self.data
            .get_mut("dependencies")
            .map(|dependencies| dependencies.get_mut(package_name))
            .map_or(None, |dependency| match dependency {
                Some(Value::String(value)) => Some(value),
                _ => {
                    debug!("Package not a dependency: {:?}", package_name);
                    None
                }
            })
    }

    pub fn get(&self, package_name: &str) -> Option<&str> {
        self.data
            .get("dependencies")
            .map(|dependencies| dependencies.get(package_name))
            .map_or(None, |dependency| match dependency {
                Some(Value::String(value)) => Some(value),
                _ => {
                    debug!("Package not a dependency: {:?}", package_name);
                    None
                }
            })
    }

    pub fn update(&mut self, package_name: &str, new_value: &str) -> bool {
        self.get_mut(package_name).map_or(false, |value| {
            debug!(
                "{} -- Previous value: {}, New value: {}",
                package_name, value, new_value
            );
            *value = new_value.to_string();
            true
        })
    }

    pub fn write(&self) -> std::io::Result<()> {
        debug!("Writing package.json to {:?}", self.path);
        let output = serde_json::to_string_pretty(&self.data)?;
        let mut file = File::create(self.path.clone())?;
        file.write_all(&output.as_bytes())
    }
}
