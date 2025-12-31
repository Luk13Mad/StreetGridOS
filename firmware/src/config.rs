use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use anyhow::Result;
use crate::types::{Relay, MeshType};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub node_type: Option<String>,
    pub mesh_type: Option<MeshType>,
    pub relays: Vec<Relay>,
    pub comms: Option<CommsConfig>,
    pub hardware: Option<HardwareConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareConfig {
    pub relay_pins: Option<HashMap<String, u8>>,
    pub adc: Option<AdcHardwareConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdcHardwareConfig {
    pub i2c_bus: Option<u8>,
    pub address: Option<u8>,
    pub ct_ratio: Option<f32>,
    pub voltage_ref: Option<f32>,
    pub burden_resistor: Option<f32>,
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
