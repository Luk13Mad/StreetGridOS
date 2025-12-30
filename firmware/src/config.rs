use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;
use crate::types::Relay;

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeType {
    Participant,
    Managing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub node_type: NodeType,
    pub relays: Vec<Relay>,
}

pub fn load_config(path: &str) -> Result<Config> {
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}
