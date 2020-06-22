extern crate app_dirs;
extern crate clap;
extern crate log;
extern crate solvent;
extern crate tar;

use anyhow::Result;
use app_dirs::*;
use clap::{App, Arg, SubCommand};
use std::path::PathBuf;

mod cache;
mod package;
mod package_manager;
mod registry;
mod state;
mod util;
use package::Typescript;
use registry::Registry;
use state::State;
use util::{get_path, setup_env_logger_cli};

const APP_INFO: AppInfo = AppInfo {
    name: "lpm",
    author: "toda.mark@gmail.com",
};

fn main() {
    result_main().unwrap();
}

fn result_main() -> Result<()> {
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
        .subcommand(
            SubCommand::with_name("update")
            .about("Update packages to introduce new code from its registered local dependencies. If no path is given, all packages are updated.")
            .args_from_usage("[PATH]        'Path to package directory for to update'
                             -a, --all      'Update all packages with local versions. This is the default'"),
            )
        .subcommand(
            SubCommand::with_name("reset")
            .about("Reset packages to remotely a published version. If no path is given, all packages are updated.")
            .args_from_usage("[PATH]                    'Path to package directory for to update'
                             -a, --all                  'Update all packages with remote versions. This is the default'
                             -v --version [VERSION]     'Specific vresion to update to. If none is given, the latest will be used.'
                             -l --latest                'Use the latest available remote version. This is the default'"),
            )
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
            let path = get_path(add_matches.value_of("PATH").unwrap());
            state.package_paths.insert(path);
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
        ("update", Some(update_matches)) => {
            let mut registry = load_registry(&state);

            if update_matches.is_present("all") && update_matches.is_present("PATH") {
                panic!("Both all and package path provided. Please provide one or the other");
            } else if update_matches.is_present("PATH") {
                let path = get_path(update_matches.value_of("PATH").unwrap());
                registry.update_dependencies(path);
            } else {
                // update all packages
                // TODO be smarter here to avoid double-preparing
                state.package_paths.iter().for_each(|path| {
                    registry.update_dependencies(PathBuf::from(path));
                });
            }
        }
        ("reset", Some(reset_matches)) => {
            let mut registry = load_registry(&state);

            if reset_matches.is_present("all") && reset_matches.is_present("PATH") {
                panic!("Both all and package path provided. Please provide one or the other");
            } else if reset_matches.is_present("PATH") {
                let path = get_path(reset_matches.value_of("PATH").unwrap());
                if reset_matches.is_present("version") {
                    registry.reset_dependency(
                        path,
                        reset_matches.value_of("version").map(|v| v.to_string()),
                    )?;
                } else {
                    registry.reset_dependency(path, None)?;
                }
            } else {
                // update all packages
                state.package_paths.iter().for_each(|path| {
                    registry
                        .reset_dependency(PathBuf::from(path), None)
                        .expect("Unable to reset dependency");
                });
            }
        }
        ("bundle", Some(bundle_matches)) => {
            let mut registry = load_registry(&state);
            let path = get_path(bundle_matches.value_of("PATH").unwrap());
            registry.bundle_dependencies(path);
        }
        _ => unreachable!(),
    };

    state.store();
    Ok(())
}

fn load_registry(state: &State) -> Registry {
    let mut registry = Registry::new();

    state
        .package_paths
        .iter()
        .for_each(|path| registry.add(Typescript::new(PathBuf::from(path))));
    registry
}
