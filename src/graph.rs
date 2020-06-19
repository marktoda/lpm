use crate::package::{Package};
use std::collections::VecDeque;

pub trait DependencyGraph {
    fn add(&mut self, dependency: Box<dyn Package>);
    fn get(&mut self) -> Option<Box<dyn Package>>;
    fn get_dependencies(&mut self, dependent: &Box<dyn Package>) -> Vec<&Box<dyn Package>>;
}

pub struct OrderedDependencyGraph {
    packages: VecDeque<Box<dyn Package>>,
}

impl OrderedDependencyGraph {
    pub fn new() -> OrderedDependencyGraph {
        OrderedDependencyGraph {
            packages: VecDeque::new(),
        }
    }

}

impl DependencyGraph for OrderedDependencyGraph {
    fn add(&mut self, dependency: Box<dyn Package>) {
        self.packages.push_front(dependency);
    }

    fn get(&mut self) -> Option<Box<dyn Package>> {
        self.packages.pop_back()
    }

    fn get_dependencies(&mut self, dependent: &Box<dyn Package>) -> Vec<&Box<dyn Package>> {
        self.packages.iter().filter(|package| dependent.depends_on(&***package)).collect()
    }
}

// TODO Add a graph that calculates the dependency order itself
