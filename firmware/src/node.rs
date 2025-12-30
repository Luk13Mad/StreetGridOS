use crate::types::{Relay, Priority, RelayType, NodeState};
use crate::config::NodeType;
use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

pub struct EdgeNode {
    pub id: String,
    pub node_type: NodeType,
    pub state: NodeState,
    pub battery_soc: f32,
    pub relays: Vec<Relay>,
}

impl EdgeNode {
    pub fn new(id: &str, node_type: NodeType, relays: Vec<Relay>) -> Self {
        Self {
            id: id.to_string(),
            node_type,
            state: NodeState::Normal,
            battery_soc: 1.0,
            relays,
        }
    }

    pub async fn run(&mut self) {
        info!("Node {} starting up...", self.id);

        loop {
            self.tick().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn tick(&mut self) {
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

    pub fn enter_island_mode(&mut self) {
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

    pub fn shed_load(&mut self, priority_threshold: Priority) {
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
