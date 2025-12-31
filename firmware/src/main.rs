mod types;
mod node;
mod config;
mod comms;
mod hal;

use log::{info, error, warn};
use clap::Parser;
use crate::node::EdgeNode;
use crate::config::load_config;
use crate::comms::{LoRaCommunication, CommunicationLayer, OrchestratorClient};
use crate::hal::{RelayPin, AdcConfig, create_relay_driver, create_power_sensor};
use crate::types::MeshType;
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Loading configuration from {}", args.config);
    let config = load_config(&args.config)?;

    info!("StreetGrid Firmware v0.1.0 - Multi-Relay Support");
    info!("Node ID: {}", config.id);

    // Initialize communications
    let client: Option<OrchestratorClient> = if let Some(comms_config) = config.comms {
        if let Some(lora_config) = comms_config.lora {
            info!("Initializing LoRa communication with frequency {}", lora_config.frequency);
            let layer = Arc::new(LoRaCommunication::new(lora_config.frequency));
            Some(OrchestratorClient::new(layer))
        } else {
            None
        }
    } else {
        None
    };

    // Initialize HAL drivers
    let (relay_driver, relay_pins, power_sensor, voltage_ref) = if let Some(hw_config) = &config.hardware {
        // Build relay pins list
        let relay_pins_map = hw_config.relay_pins.clone().unwrap_or_default();
        let relay_pin_configs: Vec<RelayPin> = relay_pins_map.iter()
            .map(|(id, pin)| RelayPin {
                relay_id: id.clone(),
                gpio_pin: *pin,
                active_low: false, // Default to active-high
            })
            .collect();

        let driver = if !relay_pin_configs.is_empty() {
            match create_relay_driver(&relay_pin_configs) {
                Ok(d) => Some(d),
                Err(e) => {
                    warn!("Failed to initialize relay driver: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Build ADC config
        let (sensor, voltage_ref) = if let Some(adc_config) = &hw_config.adc {
            let adc_cfg = AdcConfig {
                i2c_bus: adc_config.i2c_bus.unwrap_or(1),
                address: adc_config.address.unwrap_or(0x48),
                ct_ratio: adc_config.ct_ratio.unwrap_or(100.0),
                voltage_ref: adc_config.voltage_ref.unwrap_or(120.0),
                burden_resistor: adc_config.burden_resistor.unwrap_or(33.0),
            };
            let vref = adc_cfg.voltage_ref;
            match create_power_sensor(adc_cfg) {
                Ok(s) => (Some(s), vref),
                Err(e) => {
                    warn!("Failed to initialize power sensor: {}", e);
                    (None, vref)
                }
            }
        } else {
            (None, 120.0)
        };

        (driver, relay_pins_map, sensor, voltage_ref)
    } else {
        (None, HashMap::new(), None, 120.0)
    };

    // Get mesh type from config
    let mesh_type = config.mesh_type.unwrap_or_default();

    let mut node = EdgeNode::new(
        &config.id,
        config.relays,
        relay_pins,
        client,
        relay_driver,
        power_sensor,
        voltage_ref,
        mesh_type,
    );

    node.run().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Relay, RelayType, Priority, NodeState, MeshType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_node_initialization() {
        let relays = vec![
            Relay {
                id: "r_grid".to_string(),
                name: "Main Grid Tie".to_string(),
                relay_type: RelayType::Grid,
                priority: Priority::Critical,
                amperage: 100.0,
                is_closed: true,
            },
        ];
        let node = EdgeNode::new("test_node", relays, HashMap::new(), None, None, None, 120.0, MeshType::AdHoc);
        assert_eq!(node.relays.len(), 1);

        // Check for Grid relay
        let grid = node.relays.iter().find(|r| r.relay_type == RelayType::Grid).unwrap();
        assert!(grid.is_closed);
    }

    #[tokio::test]
    async fn test_load_shedding() {
        let relays = vec![
            Relay {
                id: "r_hvac".to_string(),
                name: "HVAC".to_string(),
                relay_type: RelayType::Load,
                priority: Priority::Medium,
                amperage: 20.0,
                is_closed: true,
            },
            Relay {
                id: "r_aux".to_string(),
                name: "Living Room Outlets".to_string(),
                relay_type: RelayType::Load,
                priority: Priority::Low,
                amperage: 10.0,
                is_closed: true,
            },
        ];
        let mut node = EdgeNode::new("test_node", relays, HashMap::new(), None, None, None, 120.0, MeshType::AdHoc);

        // Ensure everything starts closed
        assert!(node.relays.iter().all(|r| r.is_closed));

        // Shed Low priority
        node.shed_load(Priority::Low);

        let aux = node.relays.iter().find(|r| r.id == "r_aux").unwrap();
        assert_eq!(aux.is_closed, false); // Low priority should be open

        let hvac = node.relays.iter().find(|r| r.id == "r_hvac").unwrap();
        assert_eq!(hvac.is_closed, true); // Medium priority should still be closed
    }

    #[tokio::test]
    async fn test_island_mode_transition() {
         let relays = vec![
            Relay {
                id: "r_grid".to_string(),
                name: "Main Grid Tie".to_string(),
                relay_type: RelayType::Grid,
                priority: Priority::Critical,
                amperage: 100.0,
                is_closed: true,
            },
            Relay {
                id: "r_aux".to_string(),
                name: "Living Room Outlets".to_string(),
                relay_type: RelayType::Load,
                priority: Priority::Low,
                amperage: 10.0,
                is_closed: true,
            },
        ];
        let mut node = EdgeNode::new("test_node", relays, HashMap::new(), None, None, None, 120.0, MeshType::AdHoc);
        node.enter_island_mode();

        assert_eq!(node.state, NodeState::Islanded);

        let grid = node.relays.iter().find(|r| r.relay_type == RelayType::Grid).unwrap();
        assert_eq!(grid.is_closed, false); // Grid must be disconnected

        let aux = node.relays.iter().find(|r| r.id == "r_aux").unwrap();
        assert_eq!(aux.is_closed, false); // Low priority shed automatically
    }

    #[tokio::test]
    async fn test_government_mesh_keeps_grid_connected() {
        let relays = vec![
            Relay {
                id: "r_grid".to_string(),
                name: "Main Grid Tie".to_string(),
                relay_type: RelayType::Grid,
                priority: Priority::Critical,
                amperage: 100.0,
                is_closed: true,
            },
            Relay {
                id: "r_aux".to_string(),
                name: "Living Room Outlets".to_string(),
                relay_type: RelayType::Load,
                priority: Priority::Low,
                amperage: 10.0,
                is_closed: true,
            },
        ];
        let mut node = EdgeNode::new("test_node", relays, HashMap::new(), None, None, None, 120.0, MeshType::GovernmentSanctioned);
        node.enter_island_mode();

        assert_eq!(node.state, NodeState::Islanded);

        // Government mesh: Grid relay stays CONNECTED (MID handles isolation)
        let grid = node.relays.iter().find(|r| r.relay_type == RelayType::Grid).unwrap();
        assert_eq!(grid.is_closed, true); // Grid stays connected!

        // But low priority loads are still shed
        let aux = node.relays.iter().find(|r| r.id == "r_aux").unwrap();
        assert_eq!(aux.is_closed, false); // Low priority shed
    }
}
