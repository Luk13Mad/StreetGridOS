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

pub use streetgrid::{NeighborhoodMessage, FeatureReport, Heartbeat, LoadShed};

#[async_trait]
pub trait CommunicationLayer: Send + Sync {
    async fn send(&self, msg: NeighborhoodMessage) -> Result<()>;
    async fn receive(&self) -> Result<Option<NeighborhoodMessage>>;
}

pub enum IncomingCommand {
    LoadShed(LoadShed),
}

pub struct OrchestratorClient {
    layer: Arc<dyn CommunicationLayer>,
}

impl OrchestratorClient {
    pub fn new(layer: Arc<dyn CommunicationLayer>) -> Self {
        Self { layer }
    }

    pub async fn send_heartbeat(&self, node_id: &str, battery_level: f32) -> Result<()> {
        let heartbeat = Heartbeat {
            node_id: node_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
            battery_level,
        };
        let msg = NeighborhoodMessage {
            payload: Some(streetgrid::neighborhood_message::Payload::Heartbeat(heartbeat)),
        };
        self.layer.send(msg).await
    }

    pub async fn send_feature_report(&self, node_id: &str, features: Vec<String>) -> Result<()> {
        let report = FeatureReport {
            node_id: node_id.to_string(),
            features,
        };
        let msg = NeighborhoodMessage {
            payload: Some(streetgrid::neighborhood_message::Payload::FeatureReport(report)),
        };
        self.layer.send(msg).await
    }

    pub async fn receive(&self) -> Result<Option<IncomingCommand>> {
        let msg = self.layer.receive().await?;
        match msg {
            Some(m) => match m.payload {
                Some(streetgrid::neighborhood_message::Payload::LoadShed(ls)) => {
                    Ok(Some(IncomingCommand::LoadShed(ls)))
                }
                _ => Ok(None), // Ignore other messages
            },
            None => Ok(None),
        }
    }
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
