use crate::types::{Relay, Priority, RelayType, NodeState, MeshType};
use crate::comms::{IncomingCommand, OrchestratorClient, EnterIsland, EnterBlackStart, ActivateRelayByIndex, ActivateRelayByPriority};
use crate::hal::{RelayControl, PowerSensor};
use log::{info, warn, error};
use std::time::Duration;
use std::collections::HashMap;

/// Under-voltage threshold in volts - triggers voltage alert
const UNDERVOLTAGE_THRESHOLD: f32 = 110.0;

pub struct EdgeNode {
    pub id: String,
    pub state: NodeState,
    pub mesh_type: MeshType,
    pub battery_soc: f32,
    pub relays: Vec<Relay>,
    pub relay_pins: HashMap<String, u8>,
    pub client: Option<OrchestratorClient>,
    pub relay_driver: Option<Box<dyn RelayControl>>,
    pub power_sensor: Option<Box<dyn PowerSensor>>,
    pub voltage_ref: f32,
    /// Track last voltage reading for alerts
    last_voltage: f32,
}

impl EdgeNode {
    pub fn new(
        id: &str,
        relays: Vec<Relay>,
        relay_pins: HashMap<String, u8>,
        client: Option<OrchestratorClient>,
        relay_driver: Option<Box<dyn RelayControl>>,
        power_sensor: Option<Box<dyn PowerSensor>>,
        voltage_ref: f32,
        mesh_type: MeshType,
    ) -> Self {
        Self {
            id: id.to_string(),
            state: NodeState::Normal,
            mesh_type,
            battery_soc: 1.0,
            relays,
            relay_pins,
            client,
            relay_driver,
            power_sensor,
            voltage_ref,
            last_voltage: voltage_ref,
        }
    }

    pub async fn run(&mut self) {
        info!("Node {} starting up (MeshType: {:?})...", self.id, self.mesh_type);

        // Send Initial Setup Message (Feature Report with full relay metadata)
        if let Some(client) = &self.client {
            let relay_infos: Vec<crate::comms::RelayInfo> = self.relays.iter()
                .enumerate()
                .map(|(i, r)| crate::comms::RelayInfo {
                    index: i as u32,
                    id: r.id.clone(),
                    name: r.name.clone(),
                    relay_type: r.relay_type.clone() as i32,
                    priority: r.priority as i32,
                    amperage: r.amperage,
                    is_closed: r.is_closed,
                })
                .collect();

            let mesh_type_str = match self.mesh_type {
                MeshType::AdHoc => "AdHoc",
                MeshType::GovernmentSanctioned => "GovernmentSanctioned",
            };

            if let Err(e) = client.send_feature_report(&self.id, relay_infos, mesh_type_str).await {
                error!("Failed to send initial feature report: {}", e);
            }
        }

        // Event-driven intervals (no busy polling!)
        let mut adc_interval = tokio::time::interval(Duration::from_secs(5));
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(60));
        let mut message_poll_interval = tokio::time::interval(Duration::from_millis(100));

        // First tick fires immediately; skip it for heartbeat
        heartbeat_interval.tick().await;

        info!("Entering event loop (ADC: 5s, Heartbeat: 60s)");

        loop {
            tokio::select! {
                // Event 1: ADC/Voltage check (every 5 seconds)
                _ = adc_interval.tick() => {
                    self.check_voltage().await;
                }

                // Event 2: Heartbeat timer (every 60 seconds)
                _ = heartbeat_interval.tick() => {
                    self.send_heartbeat().await;
                }

                // Event 3: Check for incoming LoRa messages
                // NOTE: This is a low-frequency poll (100ms) because the current LoRa mock/stub
                // returns immediately from receive(). Once we implement the real SX126x driver
                // (M3), we can replace this with a true async receive that awaits a GPIO
                // interrupt (DIO1 pin) when a packet arrives, eliminating polling entirely.
                _ = message_poll_interval.tick() => {
                    if let Some(cmd) = self.poll_for_command().await {
                        match cmd {
                            IncomingCommand::LoadShed(ls) => self.handle_load_shed_command(ls),
                            IncomingCommand::EnterIsland(ei) => self.handle_enter_island_command(ei),
                            IncomingCommand::EnterBlackStart(ebs) => self.handle_enter_blackstart_command(ebs),
                            IncomingCommand::ActivateRelayByIndex(ar) => self.handle_activate_relay_by_index(ar),
                            IncomingCommand::ActivateRelayByPriority(arp) => self.handle_activate_relay_by_priority(arp),
                        }
                    }
                }
            }
        }
    }

    /// Check voltage and send alert if under threshold
    async fn check_voltage(&mut self) {
        let voltage = if let Some(sensor) = &mut self.power_sensor {
            match sensor.read_watts(0) {
                Ok(watts) => {
                    info!("Power reading: {} W", watts);
                    self.voltage_ref
                }
                Err(e) => {
                    warn!("ADC read failed: {}, using default voltage", e);
                    self.voltage_ref
                }
            }
        } else {
            self.voltage_ref
        };

        self.last_voltage = voltage;

        // Under-voltage detection flow
        if voltage < UNDERVOLTAGE_THRESHOLD {
            match self.state {
                NodeState::Normal => {
                    warn!("Under-voltage detected ({:.1}V < {:.1}V)! Sending alert to orchestrator.", 
                          voltage, UNDERVOLTAGE_THRESHOLD);
                    self.send_voltage_alert(voltage).await;
                    self.state = NodeState::AlertSent;
                }
                NodeState::AlertSent => {
                    // Waiting for orchestrator response
                }
                NodeState::Islanded | NodeState::BlackStart => {
                    // Already islanded
                }
            }
        }
    }

    /// Send voltage alert to orchestrator
    async fn send_voltage_alert(&self, voltage: f32) {
        if let Some(client) = &self.client {
            if let Err(e) = client.send_voltage_alert(&self.id, voltage).await {
                error!("Failed to send voltage alert: {}", e);
            }
        }
    }

    /// Send heartbeat to orchestrator
    async fn send_heartbeat(&self) {
        if let Some(client) = &self.client {
            if let Err(e) = client.send_heartbeat(&self.id, self.battery_soc).await {
                error!("Failed to send heartbeat: {}", e);
            } else {
                info!("Heartbeat sent");
            }
        }
    }

    /// Poll for incoming command.
    /// 
    /// NOTE: This is a temporary polling approach. The current LoRa communication layer
    /// (comms.rs) returns immediately from receive() because the SX126x driver is stubbed.
    /// 
    /// For true async (M3 milestone): The real SX126x driver should use a tokio::sync::Notify
    /// or mpsc channel that gets signaled when the radio's DIO1 interrupt fires, indicating
    /// a packet has been received. Then we can await that signal directly in tokio::select!
    /// instead of polling on an interval.
    async fn poll_for_command(&self) -> Option<IncomingCommand> {
        if let Some(client) = &self.client {
            match client.receive().await {
                Ok(Some(cmd)) => Some(cmd),
                _ => None,
            }
        } else {
            None
        }
    }

    fn handle_load_shed_command(&mut self, cmd: crate::comms::LoadShed) {
        if cmd.target_node_id == self.id {
            if cmd.shed_load {
                warn!("Received LoadShed command!");
                self.shed_load(Priority::Medium);
            } else {
                info!("Received LoadRestore command (ignored for now)");
            }
        }
    }

    fn handle_enter_island_command(&mut self, cmd: EnterIsland) {
        if cmd.target_node_id == self.id {
            warn!("Received EnterIsland command from orchestrator!");
            self.enter_island_mode();
        }
    }

    fn handle_enter_blackstart_command(&mut self, cmd: EnterBlackStart) {
        if cmd.target_node_id == self.id {
            warn!("Received EnterBlackStart command from orchestrator!");
            self.enter_blackstart_mode();
        }
    }

    fn handle_activate_relay_by_index(&mut self, cmd: ActivateRelayByIndex) {
        if cmd.target_node_id == self.id {
            let index = cmd.relay_index as usize;
            if index < self.relays.len() {
                let relay = &mut self.relays[index];
                info!("Activating relay by index {}: {}", index, relay.name);
                relay.is_closed = true;
                
                // Set physical relay
                let relay_id = relay.id.clone();
                self.set_physical_relay(&relay_id, true);
            } else {
                warn!("ActivateRelayByIndex: index {} out of bounds (max {})", index, self.relays.len() - 1);
            }
        }
    }

    fn handle_activate_relay_by_priority(&mut self, cmd: ActivateRelayByPriority) {
        if cmd.target_node_id == self.id {
            // Convert proto priority to our Priority enum
            let priority = match cmd.priority {
                0 => Priority::Critical,
                1 => Priority::High,
                2 => Priority::Medium,
                _ => Priority::Low,
            };
            info!("Activating all relays with priority {:?}", priority);
            self.activate_relays_by_priority(priority);
        }
    }

    /// Enter BlackStart mode - awaiting targeted relay activation
    /// NOTE: Island mode is always entered first, which sheds all loads.
    /// BlackStart is the recovery phase where we selectively re-enable relays.
    pub fn enter_blackstart_mode(&mut self) {
        self.state = NodeState::BlackStart;
        info!("Entering BlackStart mode (loads already shed from island mode)");

        // Loads are already shed from island mode - no need to shed again.
        // We keep grid connected so orchestrator can manage power flow from available sources.
    }

    /// Activate all relays matching a specific priority
    fn activate_relays_by_priority(&mut self, priority: Priority) {
        let to_activate: Vec<String> = self.relays.iter()
            .filter(|r| r.priority == priority && !r.is_closed)
            .map(|r| r.id.clone())
            .collect();

        for relay in &mut self.relays {
            if relay.priority == priority && !relay.is_closed {
                info!("Activating relay: {} (Priority: {:?})", relay.name, relay.priority);
                relay.is_closed = true;
            }
        }

        for relay_id in to_activate {
            self.set_physical_relay(&relay_id, true);
        }
    }

    // Old tick() removed - replaced by check_voltage()

    /// Enter island mode - behavior depends on mesh type
    pub fn enter_island_mode(&mut self) {
        self.state = NodeState::Islanded;
        info!("Entering island mode (MeshType: {:?})", self.mesh_type);

        // 1. Shed ALL loads (regardless of priority)
        self.shed_all_loads();

        // 2. Disconnect from utility grid ONLY in AdHoc mode
        match self.mesh_type {
            MeshType::AdHoc => {
                info!("AdHoc mesh: Disconnecting from utility grid");
                self.disconnect_grid();
            }
            MeshType::GovernmentSanctioned => {
                info!("GovernmentSanctioned mesh: Grid relay stays connected (MID handles isolation)");
                // Do NOT disconnect - the MID at the transformer handles this
            }
        }
    }

    /// Shed ALL load relays
    fn shed_all_loads(&mut self) {
        let load_ids: Vec<String> = self.relays.iter()
            .filter(|r| r.relay_type == RelayType::Load && r.is_closed)
            .map(|r| r.id.clone())
            .collect();

        for relay in &mut self.relays {
            if relay.relay_type == RelayType::Load && relay.is_closed {
                info!("Shedding Load Relay: {} (Priority: {:?})", relay.name, relay.priority);
                relay.is_closed = false;
            }
        }

        for relay_id in load_ids {
            self.set_physical_relay(&relay_id, false);
        }
    }

    /// Disconnect from the utility grid by opening all Grid relays
    fn disconnect_grid(&mut self) {
        let grid_relay_ids: Vec<String> = self.relays.iter()
            .filter(|r| r.relay_type == RelayType::Grid)
            .map(|r| r.id.clone())
            .collect();

        for relay in &mut self.relays {
            if relay.relay_type == RelayType::Grid {
                info!("Opening Grid Relay: {}", relay.name);
                relay.is_closed = false;
            }
        }

        for relay_id in grid_relay_ids {
            self.set_physical_relay(&relay_id, false);
        }
    }

    pub fn shed_load(&mut self, priority_threshold: Priority) {
        // Collect IDs to shed first to avoid borrow issues
        let to_shed: Vec<String> = self.relays.iter()
            .filter(|r| r.relay_type == RelayType::Load && r.priority >= priority_threshold && r.is_closed)
            .map(|r| r.id.clone())
            .collect();

        for relay in &mut self.relays {
            if relay.relay_type == RelayType::Load && relay.priority >= priority_threshold {
                if relay.is_closed {
                    info!("Shedding Load Relay: {} (Priority: {:?})", relay.name, relay.priority);
                    relay.is_closed = false;
                }
            }
        }

        for relay_id in to_shed {
            self.set_physical_relay(&relay_id, false);
        }
    }

    /// Set a physical relay via HAL driver.
    fn set_physical_relay(&mut self, relay_id: &str, closed: bool) {
        if let Some(pin) = self.relay_pins.get(relay_id) {
            if let Some(driver) = &mut self.relay_driver {
                if let Err(e) = driver.set_relay(*pin, closed) {
                    error!("Failed to set relay {} (pin {}): {}", relay_id, pin, e);
                }
            }
        }
    }
}


