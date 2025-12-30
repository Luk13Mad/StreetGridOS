mod types;
mod node;
mod config;

use log::info;
use clap::Parser;
use crate::node::EdgeNode;
use crate::config::load_config;
use anyhow::Result;

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
    info!("Node Type: {:?}", config.node_type);

    let mut node = EdgeNode::new(&config.id, config.node_type, config.relays);

    node.run().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Relay, RelayType, Priority, NodeState};
    use crate::config::NodeType;

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
        let node = EdgeNode::new("test_node", NodeType::Participant, relays);
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
        let mut node = EdgeNode::new("test_node", NodeType::Participant, relays);

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
        let mut node = EdgeNode::new("test_node", NodeType::Participant, relays);
        node.enter_island_mode();

        assert_eq!(node.state, NodeState::Islanded);

        let grid = node.relays.iter().find(|r| r.relay_type == RelayType::Grid).unwrap();
        assert_eq!(grid.is_closed, false); // Grid must be disconnected

        let aux = node.relays.iter().find(|r| r.id == "r_aux").unwrap();
        assert_eq!(aux.is_closed, false); // Low priority shed automatically
    }
}
