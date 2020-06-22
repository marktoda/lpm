use crate::package::{Bundle, Package, Typescript};
use log::info;
use solvent::DepGraph;
use std::fs;
use std::path::PathBuf;

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

    pub fn add(&mut self, package: Box<dyn Package>) {
        let graph = &mut self.graph;

        self.packages.iter().for_each(|other| {
            if package.depends_on(other) {
                info!(
                    "Package {:?} depends on {:?}",
                    package.get_name(),
                    other.get_name()
                );
                graph.register_dependency(package.get_path().clone(), other.get_path().clone());
            } else if other.depends_on(&package) {
                info!(
                    "Package {:?} depends on {:?}",
                    other.get_name(),
                    package.get_name()
                );
                graph.register_dependency(other.get_path().clone(), package.get_path().clone());
            }
        });

        self.packages.push(package);
    }

    pub fn update_dependencies(&mut self, path: PathBuf) {
        // TODO handle errors better in this fn
        let mut package = Typescript::new(path.clone());
        let mut processed_packages: Vec<PathBuf> = Vec::new();

        self.graph
            .dependencies_of(&package.get_path())
            .expect(format!("{:?} to have dependencies", package.get_path()).as_str())
            .filter(|dependency_path_result| **dependency_path_result.as_ref().unwrap() != path)
            .for_each(|dependency_path_result| {
                let dependency_path = dependency_path_result.unwrap().to_path_buf();
                let mut dependency = Typescript::new(dependency_path.clone());

                processed_packages.iter().for_each(|processed_package| {
                    // TODO Fix this hacky reinstantiation
                    dependency.update(Box::new(Typescript::new(processed_package.to_path_buf())));
                });

                dependency.prepare();

                package.update(Box::new(dependency.clone()));
                processed_packages.push(dependency_path);
            });

        package.prepare();
    }

    pub fn bundle_dependencies(&mut self, path: PathBuf) {
        // TODO handle errors better in this fn
        // TODO dedupe these fns
        let mut package = Typescript::new(path.clone());
        let mut processed_packages: Vec<PathBuf> = Vec::new();

        self.graph
            .dependencies_of(&package.get_path())
            .expect(format!("{:?} to have dependencies", package.get_path()).as_str())
            .filter(|dependency_path_result| **dependency_path_result.as_ref().unwrap() != path)
            .for_each(|dependency_path_result| {
                let dependency_path = dependency_path_result.unwrap().to_path_buf();
                let mut dependency =
                    Bundle::new(Box::new(Typescript::new(dependency_path.clone())));

                processed_packages.iter().for_each(|processed_package| {
                    // TODO Fix this hacky reinstantiation
                    dependency.update(Box::new(Bundle::new(Box::new(Typescript::new(
                        processed_package.to_path_buf(),
                    )))));
                });

                dependency.prepare();
                Registry::copy_tarball(&dependency, &package);

                package.update(Box::new(dependency));
                processed_packages.push(dependency_path.to_path_buf());
            });

        package.prepare();
    }

    fn copy_tarball(dependency: &Bundle, package: &dyn Package) {
        let mut package_build_path = package.get_path();
        package_build_path.push(dependency.get_local_bundle_file());
        let mut package_build_dir_path = package_build_path.clone();
        package_build_dir_path.pop();

        fs::create_dir_all(package_build_dir_path).expect("Unable to create tmp dir");
        fs::copy(dependency.get_tarball_file(), package_build_path)
            .expect("Unable to copy tarball");
    }
}

type Packages = Vec<Box<dyn Package>>;
