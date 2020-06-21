extern crate app_dirs;
extern crate log;
extern crate solvent;

use app_dirs::*;

mod package;
mod registry;
mod util;
use package::{PackageDef};
use registry::Registry;

const _APP_INFO: AppInfo = AppInfo {
    name: "lpm",
    author: "toda.mark@gmail.com",
};

fn main() {
    env_logger::init();

    let mut registry = Registry::new();
    let bitgo_account_lib = PackageDef::new("/home/toda/dev/bitgo-account-lib");
    let sdk = PackageDef::new("/home/toda/dev/BitGoJS/modules/core");
    let statics = PackageDef::new("/home/toda/dev/BitGoJS/modules/statics");
    registry.add(statics.clone());
    registry.add(sdk.clone());
    registry.add(bitgo_account_lib.clone());

    registry.update_dependencies(sdk);
}

