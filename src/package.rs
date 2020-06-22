use crate::util::run_basic_command_expect;
use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::{debug, error, info};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub trait Package {
    fn prepare(&self);
    fn get_name(&self) -> String;
    fn get_path(&self) -> PathBuf;
    fn get_version_value(&self) -> String;
    fn update(&mut self, dependency: Box<dyn Package>) -> bool;
    fn depends_on(&self, dependency: &Box<dyn Package>) -> bool;
}

#[derive(Clone, Debug)]
pub struct Typescript {
    package_json: PackageJson,
    path: PathBuf,
}

impl Typescript {
    pub fn new(path: PathBuf) -> Typescript {
        let mut package_json_path = path.clone();
        package_json_path.push("package.json");
        let package_json = PackageJson::new(package_json_path).expect("to work");

        Typescript { package_json, path }
    }
}

impl Package for Typescript {
    fn prepare(&self) {
        info!("Preparing package: {}", self.get_name());

        run_basic_command_expect(
            format!("npm install --prefix={:?}", self.path).as_str(),
            "Failed to install",
        );

        run_basic_command_expect(
            format!("npm run build --prefix={:?}", self.path).as_str(),
            "Failed to build",
        );
    }

    fn get_name(&self) -> String {
        self.package_json.name.clone()
    }

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    fn get_version_value(&self) -> String {
        format!("file:{}", self.path.to_string_lossy())
    }

    fn update(&mut self, dependency: Box<dyn Package>) -> bool {
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

    fn depends_on(&self, dependency: &Box<dyn Package>) -> bool {
        self.package_json
            .get(&dependency.get_name())
            .map_or(false, |_| true)
    }
}

pub struct Bundle {
    inner: Box<dyn Package>,
}

impl Bundle {
    pub fn new(inner: Box<dyn Package>) -> Bundle {
        Bundle { inner }
    }

    pub fn get_tarball_file(&self) -> String {
        format!("/tmp/lpm/{}/build.tar.gz", self.get_name())
    }

    pub fn get_local_bundle_file(&self) -> String {
        format!(".lpm/{}/build.tar.gz", self.get_name())
    }

    fn get_tarball_dir(&self) -> String {
        format!("/tmp/lpm/{}", self.get_name())
    }
}

impl Package for Bundle {
    fn prepare(&self) {
        info!("Creating tarball bundle of {}", self.get_name());
        fs::create_dir_all(self.get_tarball_dir()).expect("Unable to create tmp dir");
        let tarball = File::create(self.get_tarball_file()).expect("Unable to create tarball");
        let enc = GzEncoder::new(tarball, Compression::default());
        let mut tar = tar::Builder::new(enc);
        let mut dist = self.get_path().clone();
        dist.push("dist");
        tar.append_dir_all("package/dist", dist)
            .expect("Unable to create tar archive");
        let mut package_json = self.get_path().clone();
        package_json.push("package.json");
        tar.append_file(
            "package/package.json",
            &mut File::open(package_json).expect("to access package.json"),
        )
        .expect("Unable to add package.json to tar archive ");
        let mut readme = self.get_path().clone();
        readme.push("README.md");
        tar.append_file(
            "package/README.md",
            &mut File::open(readme).expect("to access Readme"),
        )
        .expect("Unable to add package.json to tar archive ");
    }

    fn get_name(&self) -> String {
        self.inner.get_name()
    }

    fn get_path(&self) -> PathBuf {
        self.inner.get_path()
    }

    fn get_version_value(&self) -> String {
        format!("file:{}", self.get_local_bundle_file())
    }

    fn update(&mut self, dependency: Box<dyn Package>) -> bool {
        self.inner.update(dependency)
    }

    fn depends_on(&self, dependency: &Box<dyn Package>) -> bool {
        self.inner.depends_on(dependency)
    }
}

#[derive(Clone, Debug)]
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
