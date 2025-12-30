use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, PartialEq)]
enum NodeState {
    Normal,
    Islanded,
    BlackStart,
}

#[derive(Debug, PartialEq, Clone)]
enum RelayType {
    Source, // Battery, Solar, EV
    Load,   // Appliances, HVAC
    Grid,   // Main Grid Connection
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
enum Priority {
    Critical = 0, // Medical, Comms (Lowest number = Highest priority)
    High = 1,     // Fridge, Lights
    Medium = 2,   // HVAC
    Low = 3,      // TV, Washer
}

#[derive(Debug, Clone)]
struct Relay {
    id: String,
    name: String,
    relay_type: RelayType,
    priority: Priority,
    amperage: f32, // Max capacity or current draw
    is_closed: bool,
}

struct EdgeNode {
    id: String,
    state: NodeState,
    battery_soc: f32,
    relays: Vec<Relay>,
}

impl EdgeNode {
    fn new(id: &str) -> Self {
        // Mock Relays for a standard house setup
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
                id: "r_batt".to_string(),
                name: "Battery Bank".to_string(),
                relay_type: RelayType::Source,
                priority: Priority::Critical,
                amperage: 30.0,
                is_closed: true,
            },
            Relay {
                id: "r_crit".to_string(),
                name: "Critical Panel".to_string(),
                relay_type: RelayType::Load,
                priority: Priority::Critical,
                amperage: 15.0,
                is_closed: true,
            },
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

        Self {
            id: id.to_string(),
            state: NodeState::Normal,
            battery_soc: 1.0,
            relays,
        }
    }

    async fn run(&mut self) {
        info!("Node {} starting up...", self.id);

        loop {
            self.tick().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn tick(&mut self) {
        // Mock sensing loop
        let voltage = 120.0;

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

        // 2. Shed Non-Critical Loads if necessary (e.g., if we were real logic)
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
    let mut node = EdgeNode::new("node_01");
    info!("StreetGrid Firmware v0.1.0 - Multi-Relay Support");
    node.tick().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_initialization() {
        let node = EdgeNode::new("test_node");
        assert_eq!(node.relays.len(), 5);

        // Check for Grid relay
        let grid = node.relays.iter().find(|r| r.relay_type == RelayType::Grid).unwrap();
        assert!(grid.is_closed);
    }

    #[tokio::test]
    async fn test_load_shedding() {
        let mut node = EdgeNode::new("test_node");

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
        let mut node = EdgeNode::new("test_node");
        node.enter_island_mode();

        assert_eq!(node.state, NodeState::Islanded);

        let grid = node.relays.iter().find(|r| r.relay_type == RelayType::Grid).unwrap();
        assert_eq!(grid.is_closed, false); // Grid must be disconnected

        let aux = node.relays.iter().find(|r| r.id == "r_aux").unwrap();
        assert_eq!(aux.is_closed, false); // Low priority shed automatically
    }
}
