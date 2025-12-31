use anyhow::Result;

/// Trait for power sensing abstraction.
/// Allows mocking for non-Pi development and testing.
pub trait PowerSensor: Send + Sync {
    /// Read raw ADC value from a channel (0-3 for ADS1115).
    fn read_raw(&mut self, channel: u8) -> Result<i16>;
    
    /// Read current in Amps from CT clamp.
    fn read_current_amps(&mut self, channel: u8) -> Result<f32>;
    
    /// Read power in Watts (current × voltage reference).
    fn read_watts(&mut self, channel: u8) -> Result<f32>;
}

/// ADC configuration
#[derive(Debug, Clone)]
pub struct AdcConfig {
    pub i2c_bus: u8,
    pub address: u8,
    pub ct_ratio: f32,      // e.g., 100.0 for 100A:50mA CT
    pub voltage_ref: f32,   // Reference voltage for power calculation (e.g., 120.0V)
    pub burden_resistor: f32, // Burden resistor value in ohms
}

impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            i2c_bus: 1,
            address: 0x48, // Default ADS1115 address
            ct_ratio: 100.0,
            voltage_ref: 120.0,
            burden_resistor: 33.0, // Common value for 100A CT
        }
    }
}

// ============================================================================
// Real Raspberry Pi Implementation (only compiled on ARM)
// ============================================================================

#[cfg(target_os = "linux")]
pub mod rpi {
    use super::*;
    use ads1x1x::{Ads1x1x, ChannelSelection, DynamicOneShot, FullScaleRange, SlaveAddr};
    use linux_embedded_hal::I2cdev;
    
    pub struct Ads1115Sensor {
        adc: Ads1x1x<I2cdev, ads1x1x::ic::Ads1115, ads1x1x::ic::Resolution16Bit, DynamicOneShot>,
        config: AdcConfig,
    }
    
    impl Ads1115Sensor {
        pub fn new(config: AdcConfig) -> Result<Self> {
            let i2c = I2cdev::new(format!("/dev/i2c-{}", config.i2c_bus))?;
            let address = SlaveAddr::new_default(); // 0x48
            let mut adc = Ads1x1x::new_ads1115(i2c, address);
            
            // Set gain for ±4.096V range (good for CT clamp readings)
            adc.set_full_scale_range(FullScaleRange::Within4_096V)
                .map_err(|e| anyhow::anyhow!("Failed to set ADC range: {:?}", e))?;
            
            Ok(Self { adc, config })
        }
        
        fn channel_selection(channel: u8) -> ChannelSelection {
            match channel {
                0 => ChannelSelection::SingleA0,
                1 => ChannelSelection::SingleA1,
                2 => ChannelSelection::SingleA2,
                3 => ChannelSelection::SingleA3,
                _ => ChannelSelection::SingleA0,
            }
        }
    }
    
    impl PowerSensor for Ads1115Sensor {
        fn read_raw(&mut self, channel: u8) -> Result<i16> {
            let ch = Self::channel_selection(channel);
            self.adc.read(ch)
                .map_err(|e| anyhow::anyhow!("ADC read error: {:?}", e))
        }
        
        fn read_current_amps(&mut self, channel: u8) -> Result<f32> {
            let raw = self.read_raw(channel)?;
            
            // Convert raw ADC to voltage (±4.096V range, 16-bit signed)
            let voltage = (raw as f32 / 32768.0) * 4.096;
            
            // V = I_secondary × R_burden
            // I_secondary = V / R_burden
            // I_primary = I_secondary × CT_ratio
            let secondary_current = voltage / self.config.burden_resistor;
            let primary_current = secondary_current * self.config.ct_ratio;
            
            Ok(primary_current.abs())
        }
        
        fn read_watts(&mut self, channel: u8) -> Result<f32> {
            let amps = self.read_current_amps(channel)?;
            Ok(amps * self.config.voltage_ref)
        }
    }
}

// ============================================================================
// Mock Implementation (for development and testing on non-Pi platforms)
// ============================================================================

pub mod mock {
    use super::*;
    use log::debug;
    
    pub struct MockAdcSensor {
        config: AdcConfig,
        /// Simulated current values per channel (in Amps)
        simulated_amps: [f32; 4],
    }
    
    impl MockAdcSensor {
        pub fn new(config: AdcConfig) -> Result<Self> {
            Ok(Self {
                config,
                simulated_amps: [0.0, 0.0, 0.0, 0.0],
            })
        }
        
        /// Set simulated current for testing
        pub fn set_simulated_current(&mut self, channel: u8, amps: f32) {
            if (channel as usize) < self.simulated_amps.len() {
                self.simulated_amps[channel as usize] = amps;
            }
        }
    }
    
    impl PowerSensor for MockAdcSensor {
        fn read_raw(&mut self, channel: u8) -> Result<i16> {
            // Simulate raw ADC value from current
            let amps = self.simulated_amps.get(channel as usize).copied().unwrap_or(0.0);
            let secondary_current = amps / self.config.ct_ratio;
            let voltage = secondary_current * self.config.burden_resistor;
            let raw = ((voltage / 4.096) * 32768.0) as i16;
            debug!("[MOCK ADC] Channel {} → raw {}", channel, raw);
            Ok(raw)
        }
        
        fn read_current_amps(&mut self, channel: u8) -> Result<f32> {
            let amps = self.simulated_amps.get(channel as usize).copied().unwrap_or(0.0);
            debug!("[MOCK ADC] Channel {} → {} A", channel, amps);
            Ok(amps)
        }
        
        fn read_watts(&mut self, channel: u8) -> Result<f32> {
            let amps = self.read_current_amps(channel)?;
            let watts = amps * self.config.voltage_ref;
            debug!("[MOCK ADC] Channel {} → {} W", channel, watts);
            Ok(watts)
        }
    }
}

// ============================================================================
// Factory function to create appropriate sensor
// ============================================================================

#[cfg(target_os = "linux")]
pub fn create_power_sensor(config: AdcConfig) -> Result<Box<dyn PowerSensor>> {
    Ok(Box::new(rpi::Ads1115Sensor::new(config)?))
}

#[cfg(not(target_os = "linux"))]
pub fn create_power_sensor(config: AdcConfig) -> Result<Box<dyn PowerSensor>> {
    log::warn!("Using MOCK ADC sensor (not on Raspberry Pi)");
    Ok(Box::new(mock::MockAdcSensor::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_adc_current_reading() {
        let config = AdcConfig::default();
        let mut sensor = mock::MockAdcSensor::new(config).unwrap();
        
        sensor.set_simulated_current(0, 15.0); // 15 Amps
        
        let amps = sensor.read_current_amps(0).unwrap();
        assert!((amps - 15.0).abs() < 0.01);
    }
    
    #[test]
    fn test_mock_adc_power_reading() {
        let config = AdcConfig {
            voltage_ref: 120.0,
            ..Default::default()
        };
        let mut sensor = mock::MockAdcSensor::new(config).unwrap();
        
        sensor.set_simulated_current(0, 10.0); // 10 Amps
        
        let watts = sensor.read_watts(0).unwrap();
        assert!((watts - 1200.0).abs() < 0.01); // 10A × 120V = 1200W
    }
}
