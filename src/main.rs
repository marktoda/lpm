extern crate app_dirs;
extern crate log;

use app_dirs::*;
use log::info;

mod package;
mod util;
mod graph;
use package::{Typescript};
use graph::{DependencyGraph, OrderedDependencyGraph};

const _APP_INFO: AppInfo = AppInfo {
    name: "lpm",
    author: "toda.mark@gmail.com",
};

fn main() {
    env_logger::init();

    let mut graph = OrderedDependencyGraph::new();
    let statics = Typescript::new("/home/toda/dev/BitGoJS/modules/statics", "@bitgo/statics");
    let account_lib = Typescript::new("/home/toda/dev/bitgo-account-lib", "@bitgo/account-lib");
    graph.add(Box::new(account_lib));
    graph.add(Box::new(statics));
    update_dependencies(&mut graph);
}

fn update_dependencies(graph: &mut dyn DependencyGraph) {
    let mut first = graph.get().unwrap();
    info!("first: {}", first.get_name());
    let dependencies = graph.get_dependencies(&first);
    dependencies.iter().for_each(|dep| {
        info!("{:?}", dep.get_name());
        first.update(&***dep);
    });

    let second = graph.get().unwrap();
    info!("second: {}", second.get_name());
    let other_deps = graph.get_dependencies(&second);
    other_deps.iter().for_each(|dep| info!("{:?}", dep.get_name()));
}

// Where should I store my app's per-user configuration data?
// println!("{:?}", app_root(AppDataType::UserConfig, &APP_INFO));
// Windows: "%APPDATA%\SuperDev\CoolApp"
//   (e.g.: "C:\Users\Rusty\AppData\Roaming\SuperDev\CoolApp")
//   macOS: "$HOME/Library/Application Support/CoolApp"
//   (e.g.: "/Users/Rusty/Library/Application Support/CoolApp")
//    *nix: "$HOME/.config/CoolApp" (or "$XDG_CONFIG_HOME/CoolApp", if defined)
//   (e.g.: "/home/rusty/.config/CoolApp")

// How about nested cache data?
// println!("{:?}", app_dir(AppDataType::UserCache, &APP_INFO, "cache/images"));
// Windows: "%LOCALAPPDATA%\SuperDev\CoolApp\cache\images"
//   (e.g.: "C:\Users\Rusty\AppData\Local\SuperDev\CoolApp\cache\images")
//   macOS: "$HOME/Library/Caches/CoolApp/cache/images"
//   (e.g.: "/Users/Rusty/Library/Caches/CoolApp/cache/images")
//    *nix: "$HOME/.cache/CoolApp/cache/images"
//          (or "$XDG_CACHE_HOME/CoolApp/cache/images", if defined)
//   (e.g.: "/home/rusty/.cache/CoolApp/cache/images")

// Remove "get_" prefix to recursively create nonexistent directories:
// app_root(AppDataType::UserConfig, &APP_INFO)
// app_dir(AppDataType::UserCache, &APP_INFO, "cache/images")
