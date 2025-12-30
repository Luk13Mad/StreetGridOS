use anyhow::Result;
use async_trait::async_trait;
use prost::Message;
use log::{info, warn, debug};
use std::sync::Arc;
use tokio::sync::Mutex;

// Include the generated proto modules
pub mod streetgrid {
    include!(concat!(env!("OUT_DIR"), "/streetgrid.rs"));
}

pub use streetgrid::{NeighborhoodMessage, FeatureReport, NodeType, Heartbeat, LoadShed};

#[async_trait]
pub trait CommunicationLayer: Send + Sync {
    async fn send(&self, msg: NeighborhoodMessage) -> Result<()>;
    async fn receive(&self) -> Result<Option<NeighborhoodMessage>>;
}

pub struct LoRaCommunication {
    // In a real implementation, this would hold the SX126x driver instance
    // For now, we simulate it or just hold config
    pub frequency: u64,
}

impl LoRaCommunication {
    pub fn new(frequency: u64) -> Self {
        Self { frequency }
    }
}

#[async_trait]
impl CommunicationLayer for LoRaCommunication {
    async fn send(&self, msg: NeighborhoodMessage) -> Result<()> {
        // Serialize the message
        let mut buf = Vec::new();
        msg.encode(&mut buf)?;

        // Simulate sending via LoRa
        info!("(LoRa/{}Hz) Sending {} bytes: {:?}", self.frequency, buf.len(), msg);
        // Here we would call the driver's send function
        Ok(())
    }

    async fn receive(&self) -> Result<Option<NeighborhoodMessage>> {
        // In a real implementation, this would await an interrupt or poll the radio
        // For now, we just return None to simulate silence
        // Or we could simulate incoming messages for testing
        Ok(None)
    }
}
