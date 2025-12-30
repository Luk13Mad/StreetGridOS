use log::{info, warn, error};
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use serde::Deserialize;
use std::env;

#[derive(Debug, PartialEq, Deserialize)]
enum NodeState {
    Normal,
    Islanded,
    BlackStart,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
enum RelayType {
    Source, // Battery, Solar, EV
    Load,   // Appliances, HVAC
    Grid,   // Main Grid Connection
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Deserialize)]
enum Priority {
    Critical = 0, // Medical, Comms (Lowest number = Highest priority)
    High = 1,     // Fridge, Lights
    Medium = 2,   // HVAC
    Low = 3,      // TV, Washer
}

#[derive(Debug, Clone, Deserialize)]
struct Relay {
    id: String,
    name: String,
    relay_type: RelayType,
    priority: Priority,
    amperage: f32, // Max capacity or current draw
    #[serde(default = "default_relay_closed")]
    is_closed: bool,
}

fn default_relay_closed() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct NodeConfig {
    id: String,
    #[serde(default = "default_node_type")]
    node_type: String, // "anchor" or "participant"
    relays: Vec<Relay>,
}

fn default_node_type() -> String {
    "participant".to_string()
}

struct EdgeNode {
    id: String,
    state: NodeState,
    node_type: String,
    battery_soc: f32,
    relays: Vec<Relay>,
}

impl EdgeNode {
    fn from_config(config: NodeConfig) -> Self {
        Self {
            id: config.id,
            state: NodeState::Normal,
            node_type: config.node_type,
            battery_soc: 1.0,
            relays: config.relays,
        }
    }

    async fn run(&mut self) {
        info!("Node {} ({}) starting up...", self.id, self.node_type);

        loop {
            self.tick().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn tick(&mut self) {
        // Mock sensing loop
        let voltage = 120.0;

        // Log status to use fields and avoid warnings
        info!("Tick: V={:.1} SOC={:.2} Relays={}", voltage, self.battery_soc, self.relays.len());
        for r in &self.relays {
             if r.is_closed {
                 // Log amperage to use the field
                 info!("  -> Relay {}: Closed ({}A)", r.name, r.amperage);
             }
        }

        // Simple safety logic: Under-voltage handling
        if voltage < 110.0 {
            if self.state != NodeState::Islanded {
                warn!("Under-voltage detected! Switching to Island mode.");
                self.enter_island_mode();
            }
        }
    }

    fn enter_island_mode(&mut self) {
        self.state = NodeState::Islanded;

        // 1. Disconnect Main Grid
        for relay in &mut self.relays {
            if relay.relay_type == RelayType::Grid {
                info!("Opening Relay: {}", relay.name);
                relay.is_closed = false;
            }
        }

        // 2. Shed Non-Critical Loads if necessary
        // For demonstration, let's shed Low priority immediately upon islanding
        self.shed_load(Priority::Low);
    }

    fn shed_load(&mut self, priority_threshold: Priority) {
        for relay in &mut self.relays {
            if relay.relay_type == RelayType::Load && relay.priority >= priority_threshold {
                if relay.is_closed {
                    info!("Shedding Load Relay: {} (Priority: {:?})", relay.name, relay.priority);
                    relay.is_closed = false;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config.yaml"
    };

    info!("Loading configuration from {}", config_path);

    let config_content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read config file: {}", e);
            return;
        }
    };

    let config: NodeConfig = match serde_yaml::from_str(&config_content) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse config file: {}", e);
            return;
        }
    };

    let mut node = EdgeNode::from_config(config);
    info!("StreetGrid Firmware v0.1.0 - Config Loaded");

    // Enter main loop
    node.run().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
        id: "test_node"
        node_type: "participant"
        relays:
          - id: "r1"
            name: "Grid"
            relay_type: Grid
            priority: Critical
            amperage: 100.0
        "#;

        let config: NodeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.id, "test_node");
        assert_eq!(config.relays.len(), 1);
        assert_eq!(config.relays[0].relay_type, RelayType::Grid);
    }

    #[tokio::test]
    async fn test_node_from_config() {
        let config = NodeConfig {
            id: "test_node".to_string(),
            node_type: "participant".to_string(),
            relays: vec![
                Relay {
                    id: "r1".to_string(),
                    name: "Load".to_string(),
                    relay_type: RelayType::Load,
                    priority: Priority::Low,
                    amperage: 10.0,
                    is_closed: true,
                }
            ]
        };

        let mut node = EdgeNode::from_config(config);
        assert_eq!(node.id, "test_node");
        assert_eq!(node.relays[0].id, "r1");

        // Test logic
        node.shed_load(Priority::Low);
        assert_eq!(node.relays[0].is_closed, false);
    }
}
