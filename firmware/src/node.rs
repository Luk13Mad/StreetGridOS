use crate::types::{Relay, Priority, RelayType, NodeState};
use crate::comms::{CommunicationLayer, NeighborhoodMessage, FeatureReport};
use log::{info, warn, error};
use std::time::Duration;
use tokio::time::sleep;
use std::sync::Arc;

pub struct EdgeNode {
    pub id: String,
    pub state: NodeState,
    pub battery_soc: f32,
    pub relays: Vec<Relay>,
    pub comms: Option<Arc<dyn CommunicationLayer>>,
}

impl EdgeNode {
    pub fn new(id: &str, relays: Vec<Relay>, comms: Option<Arc<dyn CommunicationLayer>>) -> Self {
        Self {
            id: id.to_string(),
            state: NodeState::Normal,
            battery_soc: 1.0,
            relays,
            comms,
        }
    }

    pub async fn run(&mut self) {
        info!("Node {} starting up...", self.id);

        // Send Initial Setup Message (Feature Report)
        if let Some(comms) = &self.comms {
            let feature_report = FeatureReport {
                node_id: self.id.clone(),
                features: self.relays.iter().map(|r| r.relay_type.clone() as i32).map(|t| t.to_string()).collect(), // Just sending relay types as features for now
            };

            let msg = NeighborhoodMessage {
                payload: Some(crate::comms::streetgrid::neighborhood_message::Payload::FeatureReport(feature_report)),
            };

            if let Err(e) = comms.send(msg).await {
                error!("Failed to send initial feature report: {}", e);
            }
        }

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
