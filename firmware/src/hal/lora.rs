use anyhow::Result;
use log::info;

/// LoRa configuration
#[derive(Debug, Clone)]
pub struct LoRaHalConfig {
    pub spi_bus: u8,
    pub spi_cs: u8,
    pub frequency: u64,
    pub bandwidth: u32,
    pub spreading_factor: u8,
    pub tx_power: i8,
}

impl Default for LoRaHalConfig {
    fn default() -> Self {
        Self {
            spi_bus: 0,
            spi_cs: 0,
            frequency: 915_000_000, // US ISM band
            bandwidth: 125_000,
            spreading_factor: 7,
            tx_power: 14,
        }
    }
}

/// Low-level LoRa radio trait.
/// This is the HAL-level interface; higher-level protocol is in comms.rs.
pub trait LoRaRadio: Send + Sync {
    /// Transmit raw bytes over LoRa.
    fn transmit(&mut self, data: &[u8]) -> Result<()>;
    
    /// Receive raw bytes. Returns None if no data available.
    fn receive(&mut self) -> Result<Option<Vec<u8>>>;
    
    /// Get RSSI of last received packet.
    fn last_rssi(&self) -> Option<i16>;
    
    /// Set radio to standby mode (low power).
    fn standby(&mut self) -> Result<()>;
}

// ============================================================================
// Real Raspberry Pi Implementation (only compiled on ARM)
// Placeholder for SX126x driver - full implementation is M3
// ============================================================================

#[cfg(target_os = "linux")]
pub mod rpi {
    use super::*;
    
    /// Placeholder for real SX126x driver.
    /// Full implementation requires sx126x-rs crate or custom SPI driver.
    pub struct Sx126xRadio {
        config: LoRaHalConfig,
        // In real implementation: SPI device handle, GPIO pins for reset/busy/dio1
    }
    
    impl Sx126xRadio {
        pub fn new(config: LoRaHalConfig) -> Result<Self> {
            info!("Initializing SX126x radio at {} Hz (STUB)", config.frequency);
            // TODO M3: Initialize SPI, configure radio
            Ok(Self { config })
        }
    }
    
    impl LoRaRadio for Sx126xRadio {
        fn transmit(&mut self, data: &[u8]) -> Result<()> {
            info!("[SX126x STUB] TX {} bytes", data.len());
            // TODO M3: Actual SPI transmission
            Ok(())
        }
        
        fn receive(&mut self) -> Result<Option<Vec<u8>>> {
            // TODO M3: Poll DIO1 interrupt, read FIFO
            Ok(None)
        }
        
        fn last_rssi(&self) -> Option<i16> {
            None
        }
        
        fn standby(&mut self) -> Result<()> {
            info!("[SX126x STUB] Entering standby");
            Ok(())
        }
    }
}

// ============================================================================
// Mock Implementation (for development and testing on non-Pi platforms)
// ============================================================================

pub mod mock {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    
    pub struct MockLoRaRadio {
        config: LoRaHalConfig,
        tx_log: Mutex<Vec<Vec<u8>>>,
        rx_queue: Mutex<VecDeque<Vec<u8>>>,
    }
    
    impl MockLoRaRadio {
        pub fn new(config: LoRaHalConfig) -> Result<Self> {
            info!("[MOCK LoRa] Initialized at {} Hz", config.frequency);
            Ok(Self {
                config,
                tx_log: Mutex::new(Vec::new()),
                rx_queue: Mutex::new(VecDeque::new()),
            })
        }
        
        /// Inject a message to be received (for testing).
        pub fn inject_rx(&self, data: Vec<u8>) {
            self.rx_queue.lock().unwrap().push_back(data);
        }
        
        /// Get transmitted messages (for testing).
        pub fn get_tx_log(&self) -> Vec<Vec<u8>> {
            self.tx_log.lock().unwrap().clone()
        }
    }
    
    impl LoRaRadio for MockLoRaRadio {
        fn transmit(&mut self, data: &[u8]) -> Result<()> {
            info!("[MOCK LoRa] TX {} bytes: {:02x?}", data.len(), data);
            self.tx_log.lock().unwrap().push(data.to_vec());
            Ok(())
        }
        
        fn receive(&mut self) -> Result<Option<Vec<u8>>> {
            Ok(self.rx_queue.lock().unwrap().pop_front())
        }
        
        fn last_rssi(&self) -> Option<i16> {
            Some(-50) // Simulated good signal
        }
        
        fn standby(&mut self) -> Result<()> {
            info!("[MOCK LoRa] Standby");
            Ok(())
        }
    }
}

// ============================================================================
// Factory function to create appropriate radio
// ============================================================================

#[cfg(target_os = "linux")]
pub fn create_lora_radio(config: LoRaHalConfig) -> Result<Box<dyn LoRaRadio>> {
    Ok(Box::new(rpi::Sx126xRadio::new(config)?))
}

#[cfg(not(target_os = "linux"))]
pub fn create_lora_radio(config: LoRaHalConfig) -> Result<Box<dyn LoRaRadio>> {
    log::warn!("Using MOCK LoRa radio (not on Raspberry Pi)");
    Ok(Box::new(mock::MockLoRaRadio::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_lora_tx_rx() {
        let config = LoRaHalConfig::default();
        let mut radio = mock::MockLoRaRadio::new(config).unwrap();
        
        // Test TX
        radio.transmit(&[0x01, 0x02, 0x03]).unwrap();
        let log = radio.get_tx_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0], vec![0x01, 0x02, 0x03]);
        
        // Test RX
        radio.inject_rx(vec![0xAA, 0xBB]);
        let rx = radio.receive().unwrap();
        assert_eq!(rx, Some(vec![0xAA, 0xBB]));
        
        // No more data
        let rx = radio.receive().unwrap();
        assert_eq!(rx, None);
    }
}
