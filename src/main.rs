extern crate app_dirs;
extern crate log;
extern crate solvent;

use app_dirs::*;

mod package;
mod registry;
mod util;
use package::Package;
use registry::Registry;

const _APP_INFO: AppInfo = AppInfo {
    name: "lpm",
    author: "toda.mark@gmail.com",
};

fn main() {
    env_logger::init();

    let mut registry = Registry::new();
    let bitgo_account_lib = Package::new("/home/toda/dev/bitgo-account-lib");
    let sdk_path = "/home/toda/dev/BitGoJS/modules/core";
    let sdk = Package::new(sdk_path);
    let statics = Package::new("/home/toda/dev/BitGoJS/modules/statics");
    registry.add(statics);
    registry.add(sdk);
    registry.add(bitgo_account_lib);

    registry.update_dependencies(sdk_path);
}

