use btleplug::platform::Peripheral;
use btleplug::api::{Peripheral as PeripheralTrait, CharPropFlags};
use log::{info, warn, debug};
use std::sync::Arc;

#[derive(Debug)]
pub struct BluetoothDevice {
    pub mac_address: String,
    pub name: String,
    pub rssi: i16,
    pub peripheral: Arc<Peripheral>,
}

impl BluetoothDevice {
    pub fn new(mac_address: String, name: String, rssi: i16, peripheral: Arc<Peripheral>) -> Self {
        debug!("Creating new BluetoothDevice: MAC={}, Name={}, RSSI={}", mac_address, name, rssi);
        BluetoothDevice {
            mac_address,
            name,
            rssi,
            peripheral,
        }
    }

    pub async fn list_available_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }

        info!("Connected to device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }

        for service in self.peripheral.services() {
            info!("Service UUID: {:?}", service.uuid);

            for characteristic in &service.characteristics {
                info!("Characteristic UUID: {:?}, Properties: {:?}", characteristic.uuid, characteristic.properties);
            }
        }

        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }

        Ok(())
    }

    pub async fn retrieve_additional_info(&self) {
        info!("Connecting to device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return;
        }

        info!("Connected to device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return;
        }

        for service in self.peripheral.services() {
            info!("Service UUID: {:?}", service.uuid);

            for characteristic in service.characteristics {
                info!("Characteristic UUID: {:?}", characteristic.uuid);

                if characteristic.properties.contains(CharPropFlags::READ) {
                    match self.peripheral.read(&characteristic).await {
                        Ok(value) => {
                            info!("Read value from characteristic {:?}: {:?}", characteristic.uuid, value);
                        }
                        Err(err) => {
                            warn!("Failed to read characteristic {:?}: {:?}", characteristic.uuid, err);
                        }
                    }
                }
            }
        }

        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }
    }

    /// Retrieve temperature and humidity from an MJ_HT_V1 device
    pub async fn retrieve_temperature_and_humidity(&self) -> Result<(f32, f32), Box<dyn std::error::Error>> {
        info!("Connecting to MJ_HT_V1 device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }

        info!("Connected to device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }

        let mut temperature = None;
        let mut humidity = None;

        for service in self.peripheral.services() {
            for characteristic in &service.characteristics {
                if characteristic.properties.contains(CharPropFlags::READ) {
                    let uuid = characteristic.uuid.to_string();
                    
                    // Example UUIDs, these need to be replaced with the actual UUIDs for temperature and humidity
                    if uuid == "00002A6E-0000-1000-8000-00805F9B34FB" { // Replace with actual UUID for temperature
                        match self.peripheral.read(characteristic).await {
                            Ok(value) => {
                                // Convert the value to a temperature reading
                                temperature = Some(Self::parse_temperature(&value));
                            }
                            Err(err) => {
                                warn!("Failed to read temperature characteristic: {:?}", err);
                            }
                        }
                    } else if uuid == "00002A6F-0000-1000-8000-00805F9B34FB" { // Replace with actual UUID for humidity
                        match self.peripheral.read(characteristic).await {
                            Ok(value) => {
                                // Convert the value to a humidity reading
                                humidity = Some(Self::parse_humidity(&value));
                            }
                            Err(err) => {
                                warn!("Failed to read humidity characteristic: {:?}", err);
                            }
                        }
                    }
                }
            }
        }

        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }

        if let (Some(temp), Some(hum)) = (temperature, humidity) {
            Ok((temp, hum))
        } else {
            Err("Failed to retrieve temperature and humidity".into())
        }
    }

    fn parse_temperature(value: &[u8]) -> f32 {
        // Assuming the temperature is in the first two bytes and is little-endian
        let raw_value = i16::from_le_bytes([value[0], value[1]]);
        raw_value as f32 / 100.0 // Example scaling factor, adjust based on actual data format
    }

    fn parse_humidity(value: &[u8]) -> f32 {
        // Assuming the humidity is in the first two bytes and is little-endian
        let raw_value = i16::from_le_bytes([value[0], value[1]]);
        raw_value as f32 / 100.0 // Example scaling factor, adjust based on actual data format
    }
}
