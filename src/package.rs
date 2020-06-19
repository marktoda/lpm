use crate::util::run_basic_command;
use anyhow::Result;
use log::{debug, info, error};
use serde_json::Value;
use std::fs::File;
use std::io::Write;

pub trait Package {
    fn prepare(&self);
    fn get_name(&self) -> String;
    fn get_version_value(&self) -> String;
    fn depends_on(&self, dependency: &dyn Package) -> bool;
    fn update(&mut self, dependency: &dyn Package) -> bool;
}

pub struct Typescript {
    package_json: PackageJson,
    path: String,
    name: String,
}

impl Typescript {
    pub fn new(path: &str, name: &str) -> Typescript {
        let mut our_path = path.to_string();

        if our_path.ends_with('/') {
            our_path.pop();
        }

        let package_json = PackageJson::new(format!("{}/package.json", our_path).as_str()).expect("to work");

        Typescript {
            package_json,
            path: our_path,
            name: name.to_string(),
        }
    }
}

impl Package for Typescript {
    fn prepare(&self) {
        info!("Preparing typescript package: {}", self.path);

        let output = run_basic_command(format!("npm run build --prefix={}", &self.path).as_str())
            .expect("Failed to build");
        info!("Exit code: {} for package: {}", output.status, self.path);
        debug!("output: {:?}", output);
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_version_value(&self) -> String {
        format!("file:{}", self.path.as_str())
    }

    fn update(&mut self, dependency: &dyn Package) -> bool {
        dependency.prepare();
        if self.package_json.update(&dependency.get_name(), &dependency.get_version_value()) {
            self.package_json.write().map_or_else(|e| {
                error!("Received error writing package.json: {:?}", e);
                false
            }, |_| true)
        } else {
            false
        }
    }

    fn depends_on(&self, dependency: &dyn Package) -> bool {
        info!("Checking if {} depends on {}", self.get_name(), dependency.get_name());
        self.package_json.get(&dependency.get_name()).map_or(false, |_| true)
    }
}

struct PackageJson {
    path: String,
    data: Value,
}

impl PackageJson {
    pub fn new(path: &str) -> Result<PackageJson> {
        Ok(PackageJson {
            path: path.to_string(),
            data: serde_json::from_reader(File::open(path)?)?,
        })
    }

    fn get_mut(&mut self, package_name: &str) -> Option<&mut String> {
        self.data
            .get_mut("dependencies")
            .map(|dependencies| dependencies.get_mut(package_name))
            .map_or(None, |dependency| {
                match dependency {
                    Some(Value::String(value)) => {
                        Some(value)
                    }
                    _ => {
                        error!("Malformed package.json: {:?}", dependency);
                        None
                    },
                }
            })

    }

    pub fn get(&self, package_name: &str) -> Option<&str> {
        self.data
            .get("dependencies")
            .map(|dependencies| dependencies.get(package_name))
            .map_or(None, |dependency| {
                match dependency {
                    Some(Value::String(value)) => {
                        Some(value)
                    }
                    _ => {
                        error!("Malformed package.json: {:?}", dependency);
                        None
                    },
                }
            })
    }

    pub fn update(&mut self, package_name: &str, new_value: &str) -> bool {
        self.get_mut(package_name)
            .map_or(false, |value| {
                debug!("{} -- Previous value: {}, New value: {}", package_name, value, new_value);
                *value = new_value.to_string();
                true
            })
    }

    pub fn write(&self) -> std::io::Result<()> {
        debug!("Writing package.json to {}", self.path);
        let output = serde_json::to_string_pretty(&self.data)?;
        let mut file = File::create(self.path.as_str())?;
        file.write_all(&output.as_bytes())
    }
}
