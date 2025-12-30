use crate::types::{Relay, Priority, RelayType, NodeState};
use crate::comms::{IncomingCommand, OrchestratorClient};
use log::{info, warn, error};
use std::time::Duration;
use tokio::time::sleep;

pub struct EdgeNode {
    pub id: String,
    pub state: NodeState,
    pub battery_soc: f32,
    pub relays: Vec<Relay>,
    pub client: Option<OrchestratorClient>,
}

impl EdgeNode {
    pub fn new(id: &str, relays: Vec<Relay>, client: Option<OrchestratorClient>) -> Self {
        Self {
            id: id.to_string(),
            state: NodeState::Normal,
            battery_soc: 1.0,
            relays,
            client,
        }
    }

    pub async fn run(&mut self) {
        info!("Node {} starting up...", self.id);

        // Send Initial Setup Message (Feature Report)
        if let Some(client) = &self.client {
            let features: Vec<String> = self.relays.iter()
                .map(|r| r.relay_type.clone() as i32)
                .map(|t| t.to_string())
                .collect();

            if let Err(e) = client.send_feature_report(&self.id, features).await {
                error!("Failed to send initial feature report: {}", e);
            }
        }

        let mut last_heartbeat = std::time::Instant::now();

        loop {
            self.tick().await;

            let mut received_cmd = None;
            let mut should_send_heartbeat = false;

            if let Some(client) = &self.client {
                // Poll for messages
                if let Ok(Some(cmd)) = client.receive().await {
                    received_cmd = Some(cmd);
                }

                // Check heartbeat
                if last_heartbeat.elapsed() > Duration::from_secs(60) {
                     should_send_heartbeat = true;
                }
            }

            if let Some(cmd) = received_cmd {
                 match cmd {
                    IncomingCommand::LoadShed(ls) => self.handle_load_shed_command(ls),
                }
            }

            if should_send_heartbeat {
                 if let Some(client) = &self.client {
                     if let Err(e) = client.send_heartbeat(&self.id, self.battery_soc).await {
                         error!("Failed to send heartbeat: {}", e);
                     }
                     last_heartbeat = std::time::Instant::now();
                 }
            }

            sleep(Duration::from_secs(1)).await;
        }
    }

    fn handle_load_shed_command(&mut self, cmd: crate::comms::LoadShed) {
        if cmd.target_node_id == self.id {
            if cmd.shed_load {
                warn!("Received LoadShed command!");
                // Shed Medium priority and below? Or just call shed_load with a default?
                // The prompt says "Shed Non-Critical Loads".
                // I'll assume shedding Medium and Low.
                self.shed_load(Priority::Medium);
            } else {
                info!("Received LoadRestore command (ignored for now)");
            }
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
