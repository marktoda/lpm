use log::info;
use solvent::DepGraph;
use crate::package::{Package};

pub struct Registry {
    packages: Packages,
    graph: DepGraph<String>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            packages: Packages::new(),
            graph: DepGraph::new(),
        }
    }

    pub fn add(&mut self, package: Package) {
        let graph = &mut self.graph;

        self.packages.iter().for_each(|other| {
            if package.depends_on(&other) {
                info!("Package {:?} depends on {:?}", package.get_name(), other.get_name());
                graph.register_dependency(package.path.clone(), other.path.clone());
            } else if other.depends_on(&package) {
                info!("Package {:?} depends on {:?}", other.get_name(), package.get_name());
                graph.register_dependency(other.path.clone(), package.path.clone());
            }
        });

        self.packages.push(package);
    }

    pub fn update_dependencies(&mut self, path: &str) {
        // TODO handle errors better in this fn
        let mut package = Package::new(path);
        let mut processed_packages =  Packages::new();

        self.graph.dependencies_of(&package.path).expect(format!("{} to have dependencies", package.path).as_str())
            .filter(|dependency_path_result| **dependency_path_result.as_ref().unwrap() != path.to_string())
            .for_each(|dependency_path_result| {
            let dependency_path = dependency_path_result.unwrap();
            let mut dependency = Package::new(dependency_path);

            processed_packages.iter().for_each(|processed_package| {
                dependency.update(&processed_package);
            });

            dependency.prepare();

            package.update(&dependency);
            processed_packages.push(dependency);
        });

        package.prepare();
    }
}

type Packages = Vec<Package>;

