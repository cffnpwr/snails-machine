use crate::Direction;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::Path,
    thread::{self, JoinHandle},
};

#[derive(Debug, Deserialize)]
pub struct ConfigTransition {
    pub from: String,
    pub to: String,
    pub read: String,
    pub write: String,
    pub direction: Direction,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub transitions: Vec<ConfigTransition>,
    pub initial_state: String,
    pub accept_states: Vec<String>,
    pub blank: String,
}
impl Config {
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;

        let mut handles = Vec::new();
        for file_type in ["json", "toml", "yaml"] {
            let content = content.clone();
            let handle: JoinHandle<Result<Config>> = thread::spawn(move || match file_type {
                "json" => serde_json::from_str(&content).context("Invalid JSON"),
                "toml" => toml::from_str(&content).context("Invalid TOML"),
                "yaml" => serde_yaml::from_str(&content).context("Invalid YAML"),
                _ => unreachable!(),
            });

            handles.push(handle);
        }
        let config = handles.into_iter().find_map(|h| {
            let result = h.join().unwrap();
            if result.is_ok() {
                result.ok()
            } else {
                None
            }
        });

        config.ok_or_else(|| anyhow!("Invalid config file"))
    }
}
