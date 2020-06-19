use log::debug;
use std::process::{Command, Output};

pub fn run_basic_command(command: &str) -> Result<Output, std::io::Error> {
    debug!("running command: {:?}", command);
    Command::new("sh").arg("-c").arg(command).output()
}
