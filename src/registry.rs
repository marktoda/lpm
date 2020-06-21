use log::debug;
use solvent::DepGraph;
use crate::package::{Package, PackageDef};

pub struct Registry {
    packages: Packages,
    graph: DepGraph<PackageDef>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            packages: Packages::new(),
            graph: DepGraph::new(),
        }
    }

    pub fn add(&mut self, def: PackageDef) {
        let package = Package::new(def.clone());
        let graph = &mut self.graph;

        self.packages.iter().for_each(|other_def| {
            let other = Package::new(other_def.clone());
            if package.depends_on(other_def.clone()) {
                debug!("{:?} depends on {:?}", package.get_name(), other.get_name());
                graph.register_dependency(def.clone(), other_def.clone())
            } else if other.depends_on(def.clone()) {
                debug!("{:?} depends on {:?}", other.get_name(), package.get_name());
                graph.register_dependency(other_def.clone(), def.clone())
            }
        });

        self.packages.push(def);
    }

    pub fn update_dependencies(&mut self, def: PackageDef) {
        let mut package = Package::new(def.clone());
        let mut processed_packages =  Packages::new();

        for dependency_def_result in self.graph.dependencies_of(&def).unwrap() {
            let dependency_def = dependency_def_result.unwrap();
            let mut dependency = Package::new(dependency_def.clone());

            processed_packages.iter().for_each(|processed_package| {
                dependency.update(processed_package.clone());
            });

            dependency.prepare();

            package.update(dependency_def.clone());
            processed_packages.push(dependency_def.clone());
        }
    }
}

type Packages = Vec<PackageDef>;

