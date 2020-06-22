use crate::package::{Bundle, Package, Typescript};
use log::info;
use anyhow::Result;
use solvent::DepGraph;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct Registry {
    packages: Packages,
    graph: DepGraph<PathBuf>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            packages: Packages::new(),
            graph: DepGraph::new(),
        }
    }

    pub fn add(&mut self, package: impl Package + 'static) {
        let graph = &mut self.graph;

        self.packages.values().for_each(|other| {
            if package.depends_on(&other.get_name()) {
                info!(
                    "Package {:?} depends on {:?}",
                    package.get_name(),
                    other.get_name()
                );
                graph.register_dependency(package.get_path().clone(), other.get_path().clone());
            } else if other.depends_on(&package.get_name()) {
                info!(
                    "Package {:?} depends on {:?}",
                    other.get_name(),
                    package.get_name()
                );
                graph.register_dependency(other.get_path().clone(), package.get_path().clone());
            }
        });

        self.packages.insert(package.get_path().clone(), Box::new(package));
    }

    pub fn update_dependencies(&mut self, path: PathBuf) {
        self.for_each_dependency(path.clone(), move |dependent: &mut Box<dyn Package>, mut dependency: Box<dyn Package>, processed_packages: &Vec<PathBuf>| {
            processed_packages.iter().for_each(|processed_package| {
                // TODO Fix this hacky reinstantiation
                dependency.update(Box::new(Typescript::new(processed_package.to_path_buf())));
            });

            dependency.prepare();

            dependent.update(dependency);
        });

        Typescript::new(path).prepare();
    }

    pub fn bundle_dependencies(&mut self, path: PathBuf) {
        self.for_each_dependency(path.clone(), move |dependent: &mut Box<dyn Package>, dependency: Box<dyn Package>, processed_packages: &Vec<PathBuf>| {
            let mut dependency_bundle = Bundle::new(dependency);

            processed_packages.iter().for_each(|processed_package| {
                // TODO Fix this hacky reinstantiation
                dependency_bundle.update(Box::new(Bundle::new(Box::new(Typescript::new(processed_package.to_path_buf())))));
            });

            dependency_bundle.prepare();
            Registry::copy_tarball(&dependency_bundle, Box::new(Typescript::new(dependent.get_path())));

            dependent.update(Box::new(dependency_bundle));
        });

        Typescript::new(path).prepare();
    }


    pub fn reset_dependency(&mut self, dependency_path: PathBuf, version: Option<String>) -> Result<()> {
        let dependency = Typescript::new(dependency_path);
        // update the given dependency in all packages
        for (_, dependent) in self.packages.iter_mut() {
            if dependent.depends_on(&dependency.get_name()) {
                dependent.reset(dependency.get_name(), version.clone())?;
                dependent.prepare();
            }
        };
        Ok(())
    }

    pub fn for_each_dependency(&mut self, path: PathBuf, f: impl Fn(&mut Box<dyn Package>, Box<dyn Package>, &Vec<PathBuf>)) {
        // TODO handle errors better in this fn
        // TODO Remove need to instantiate concrete types here so this func can work for different
        // package types
        let package = self.packages.get_mut(&path).unwrap();
        let mut processed_packages: Vec<PathBuf> = Vec::new();

        self.graph
            .dependencies_of(&package.get_path())
            .expect(format!("{:?} to have dependencies", package.get_path()).as_str())
            .filter(|dependency_path_result| **dependency_path_result.as_ref().unwrap() != path)
            .for_each(|dependency_path_result| {
                let dependency_path = dependency_path_result.unwrap().to_path_buf();
                let dependency = Box::new(Typescript::new(dependency_path.clone()));

                f(package, dependency.clone(), &processed_packages);
                processed_packages.push(dependency.get_path());
            });
    }

    fn copy_tarball(dependency: &Bundle, package: Box<dyn Package>) {
        let mut package_build_path = package.get_path();
        package_build_path.push(dependency.get_local_bundle_file());
        let mut package_build_dir_path = package_build_path.clone();
        package_build_dir_path.pop();

        fs::create_dir_all(package_build_dir_path).expect("Unable to create tmp dir");
        fs::copy(dependency.get_tarball_file(), package_build_path)
            .expect("Unable to copy tarball");
    }
}

type Packages = HashMap<PathBuf, Box<dyn Package>>;
