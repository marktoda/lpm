use env_logger::Env;
use log::debug;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

pub fn run_basic_command(command: &str) -> Result<Output, std::io::Error> {
    debug!("running command: {:?}", command);
    Command::new("sh").arg("-c").arg(command).output()
}

pub fn run_basic_command_expect(command: &str, error: &str) -> Output {
    let result = run_basic_command(command);
    let output = result.expect(error);

    debug!(
        "{} -- {:?} -- for command `{}`",
        output.status, output, command
    );
    output
}

pub fn setup_env_logger(default_level: &str) {
    // always override with env if given
    env_logger::from_env(Env::default().default_filter_or(default_level)).init();
}

pub fn setup_env_logger_cli(v_occurrences: u64) {
    let log_level = match v_occurrences {
        0 => "info",
        1 => "debug",
        2 | _ => "trace",
    };

    setup_env_logger(log_level);
}

pub fn get_path(path: &str) -> PathBuf {
    fs::canonicalize(PathBuf::from(path)).expect(format!("Invalid path: {}", path).as_str())
}
