use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum NodeState {
    Normal,
    AlertSent,  // Waiting for orchestrator response after voltage drop
    Islanded,
    BlackStart,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MeshType {
    AdHoc,              // Option 1: Node disconnects from utility grid
    GovernmentSanctioned, // Option 2: MID handles isolation, node keeps grid connected
}

impl Default for MeshType {
    fn default() -> Self {
        MeshType::AdHoc
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RelayType {
    Source, // Battery, Solar, EV
    Load,   // Appliances, HVAC
    Grid,   // Main Grid Connection
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Serialize, Deserialize, Copy)]
pub enum Priority {
    Critical = 0, // Medical, Comms (Lowest number = Highest priority)
    High = 1,     // Fridge, Lights
    Medium = 2,   // HVAC
    Low = 3,      // TV, Washer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relay {
    pub id: String,
    pub name: String,
    pub relay_type: RelayType,
    pub priority: Priority,
    pub amperage: f32, // Max capacity or current draw
    pub is_closed: bool,
}
