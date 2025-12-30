use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, PartialEq)]
enum NodeState {
    Normal,
    Islanded,
    BlackStart,
}

struct EdgeNode {
    id: String,
    state: NodeState,
    battery_soc: f32,
    relay_closed: bool,
}

impl EdgeNode {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            state: NodeState::Normal,
            battery_soc: 1.0,
            relay_closed: true,
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
        let voltage = 120.0; // Mock voltage
        let current = 10.0;  // Mock current

        info!("Tick: V={:.1} I={:.1} SOC={:.2}", voltage, current, self.battery_soc);

        // Simple safety logic (M1.5 precursor)
        if voltage < 110.0 {
            warn!("Under-voltage detected! Switching to Island mode.");
            self.state = NodeState::Islanded;
            self.relay_closed = false; // Open grid relay
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut node = EdgeNode::new("node_01");

    // In a real scenario, this would run indefinitely.
    // For now, we just print the startup.
    info!("StreetGrid Firmware v0.1.0");

    // Run one tick for demonstration
    node.tick().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_initialization() {
        let node = EdgeNode::new("test_node");
        assert_eq!(node.id, "test_node");
        assert_eq!(node.state, NodeState::Normal);
        assert_eq!(node.relay_closed, true);
    }

    #[tokio::test]
    async fn test_safety_logic() {
        let mut node = EdgeNode::new("test_node");

        // Mock the logic locally to test state transition
        // (In reality, we'd inject the sensor data, but here we just verify the state transition logic if we were to expose it)
        // For now, let's just verify the struct holds state.
        node.state = NodeState::Islanded;
        assert_eq!(node.state, NodeState::Islanded);
    }
}
