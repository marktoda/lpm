use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub package_paths: HashSet<PathBuf>,
    state_file: PathBuf,
}

impl State {
    pub fn new(state_dir: PathBuf) -> State {
        State {
            package_paths: HashSet::new(),
            state_file: State::get_state_file(state_dir),
        }
    }

    pub fn store(&self) {
        let serialized = serde_json::to_string(&self).expect("to be able to serialize");
        let mut file = File::create(&self.state_file).expect("state file to  exist");
        file.write_all(&serialized.as_bytes())
            .expect("to be able to write to file");
    }

    pub fn load(state_dir: PathBuf) -> Result<State> {
        let file = File::open(State::get_state_file(state_dir).clone())?;
        let loaded_state: State = serde_json::from_reader(file)?;

        Ok(loaded_state)
    }

    fn get_state_file(state_dir: PathBuf) -> PathBuf {
        let mut state_file = state_dir.clone();
        state_file.push("serialized.json");
        state_file
    }
}
