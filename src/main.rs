extern crate app_dirs;
extern crate clap;
extern crate log;
extern crate solvent;
extern crate tar;

use app_dirs::*;
use clap::{App, Arg, SubCommand};
use std::fs;
use std::path::PathBuf;

mod package;
mod registry;
mod state;
mod util;
use package::Typescript;
use registry::Registry;
use state::State;
use util::setup_env_logger_cli;

const APP_INFO: AppInfo = AppInfo {
    name: "lpm",
    author: "toda.mark@gmail.com",
};

fn main() {
    let state_dir = app_dir(AppDataType::UserData, &APP_INFO, "registry")
        .expect("To be able to create app dir");
    let mut state = State::load(state_dir.clone()).unwrap_or(State::new(state_dir));

    let matches = App::new("lpm")
        .version("1.0")
        .author("Mark Toda <toda.mark@gmail.com>")
        .about("Local package manager for typescript / javascript packages")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add a new local package to the registry")
                .arg_from_usage("<PATH> 'Path to package directory'"),
        )
        .subcommand(SubCommand::with_name("update").about(
            "Update all packages to introduce new code from its registered local dependencies",
        ))
        .subcommand(
            SubCommand::with_name("list")
                .alias("ls")
                .about("List currently added packages"),
        )
        .subcommand(SubCommand::with_name("clear").about("Clear current package list"))
        .subcommand(
            SubCommand::with_name("bundle")
                .about("Bundle local dependencies for release of the given package")
                .arg_from_usage("<PATH> 'Path to package directory for release'"),
        )
        .get_matches();

    setup_env_logger_cli(matches.occurrences_of("v"));

    match matches.subcommand() {
        ("add", Some(add_matches)) => {
            let path = add_matches.value_of("PATH").unwrap();
            let package_path = fs::canonicalize(PathBuf::from(path))
                .expect(format!("Invalid path: {}", path).as_str());
            state.package_paths.insert(package_path);
        }
        ("list", Some(_)) => {
            println!("Packages: ");
            state.package_paths.iter().for_each(|path| {
                println!("\t {:?}", path);
            });
        }
        ("clear", Some(_)) => {
            state.package_paths.clear();
        }
        ("update", Some(_)) => {
            // TODO be smarter here to avoid double-preparing
            let mut registry = load_registry(&state);
            state.package_paths.iter().for_each(|path| {
                registry.update_dependencies(PathBuf::from(path));
            });
        }
        ("bundle", Some(bundle_matches)) => {
            let path = bundle_matches.value_of("PATH").unwrap();
            let package_path = fs::canonicalize(PathBuf::from(path))
                .expect(format!("Invalid path: {}", path).as_str());
            let mut registry = load_registry(&state);
            registry.bundle_dependencies(PathBuf::from(package_path));
        }
        _ => unreachable!(),
    };

    state.store();
}

fn load_registry(state: &State) -> Registry {
    let mut registry = Registry::new();

    state
        .package_paths
        .iter()
        .for_each(|path| registry.add(Box::new(Typescript::new(PathBuf::from(path)))));
    registry
}
