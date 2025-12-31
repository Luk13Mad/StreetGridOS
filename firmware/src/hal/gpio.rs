use anyhow::Result;

/// Trait for relay control abstraction.
/// Allows mocking for non-Pi development and testing.
pub trait RelayControl: Send + Sync {
    /// Set a relay state by GPIO pin number.
    /// `closed = true` means relay is engaged (circuit connected).
    fn set_relay(&mut self, pin: u8, closed: bool) -> Result<()>;
    
    /// Get current relay state.
    fn get_relay(&self, pin: u8) -> Result<bool>;
}

/// Pin configuration for a relay
#[derive(Debug, Clone)]
pub struct RelayPin {
    pub relay_id: String,
    pub gpio_pin: u8,
    pub active_low: bool, // If true, LOW = closed
}

// ============================================================================
// Real Raspberry Pi Implementation (only compiled on ARM)
// ============================================================================

#[cfg(target_os = "linux")]
pub mod rpi {
    use super::*;
    use rppal::gpio::{Gpio, OutputPin};
    use std::collections::HashMap;
    
    pub struct RpiRelayDriver {
        pins: HashMap<u8, OutputPin>,
        active_low: HashMap<u8, bool>,
    }
    
    impl RpiRelayDriver {
        pub fn new(relay_pins: &[RelayPin]) -> Result<Self> {
            let gpio = Gpio::new()?;
            let mut pins = HashMap::new();
            let mut active_low = HashMap::new();
            
            for rp in relay_pins {
                let pin = gpio.get(rp.gpio_pin)?.into_output();
                pins.insert(rp.gpio_pin, pin);
                active_low.insert(rp.gpio_pin, rp.active_low);
            }
            
            Ok(Self { pins, active_low })
        }
    }
    
    impl RelayControl for RpiRelayDriver {
        fn set_relay(&mut self, pin: u8, closed: bool) -> Result<()> {
            let output_pin = self.pins.get_mut(&pin)
                .ok_or_else(|| anyhow::anyhow!("Pin {} not configured", pin))?;
            
            let is_active_low = *self.active_low.get(&pin).unwrap_or(&false);
            
            // If active_low, invert the logic: closed (true) → LOW, open (false) → HIGH
            let level = if is_active_low { !closed } else { closed };
            
            if level {
                output_pin.set_high();
            } else {
                output_pin.set_low();
            }
            
            Ok(())
        }
        
        fn get_relay(&self, pin: u8) -> Result<bool> {
            let output_pin = self.pins.get(&pin)
                .ok_or_else(|| anyhow::anyhow!("Pin {} not configured", pin))?;
            
            let is_active_low = *self.active_low.get(&pin).unwrap_or(&false);
            let is_high = output_pin.is_set_high();
            
            // Invert if active_low
            Ok(if is_active_low { !is_high } else { is_high })
        }
    }
}

// ============================================================================
// Mock Implementation (for development and testing on non-Pi platforms)
// ============================================================================

pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use log::info;
    
    pub struct MockRelayDriver {
        states: Mutex<HashMap<u8, bool>>,
    }
    
    impl MockRelayDriver {
        pub fn new(_relay_pins: &[RelayPin]) -> Result<Self> {
            Ok(Self {
                states: Mutex::new(HashMap::new()),
            })
        }
    }
    
    impl RelayControl for MockRelayDriver {
        fn set_relay(&mut self, pin: u8, closed: bool) -> Result<()> {
            info!("[MOCK GPIO] Pin {} → {}", pin, if closed { "CLOSED" } else { "OPEN" });
            self.states.lock().unwrap().insert(pin, closed);
            Ok(())
        }
        
        fn get_relay(&self, pin: u8) -> Result<bool> {
            Ok(*self.states.lock().unwrap().get(&pin).unwrap_or(&false))
        }
    }
}

// ============================================================================
// Factory function to create appropriate driver
// ============================================================================

#[cfg(target_os = "linux")]
pub fn create_relay_driver(relay_pins: &[RelayPin]) -> Result<Box<dyn RelayControl>> {
    Ok(Box::new(rpi::RpiRelayDriver::new(relay_pins)?))
}

#[cfg(not(target_os = "linux"))]
pub fn create_relay_driver(relay_pins: &[RelayPin]) -> Result<Box<dyn RelayControl>> {
    log::warn!("Using MOCK relay driver (not on Raspberry Pi)");
    Ok(Box::new(mock::MockRelayDriver::new(relay_pins)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_relay_toggle() {
        let pins = vec![
            RelayPin { relay_id: "test".to_string(), gpio_pin: 17, active_low: false },
        ];
        let mut driver = mock::MockRelayDriver::new(&pins).unwrap();
        
        driver.set_relay(17, true).unwrap();
        assert!(driver.get_relay(17).unwrap());
        
        driver.set_relay(17, false).unwrap();
        assert!(!driver.get_relay(17).unwrap());
    }
}
