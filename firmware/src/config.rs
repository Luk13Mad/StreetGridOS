use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;
use crate::types::Relay;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub relays: Vec<Relay>,
    pub comms: Option<CommsConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommsConfig {
    pub lora: Option<LoRaConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoRaConfig {
    pub frequency: u64,
    pub bandwidth: u64,
    pub tx_power: i32,
    pub spreading_factor: u8,
}

pub fn load_config(path: &str) -> Result<Config> {
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}
