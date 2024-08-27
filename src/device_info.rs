use btleplug::platform::Peripheral;
use btleplug::api::{Peripheral as PeripheralTrait, CharPropFlags};
use log::{info, warn, debug};
use std::sync::Arc;

#[derive(Debug, Clone)]
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
            return Err(Box::new(e)); // Box the error before returning
        }

        info!("Connected to device with MAC={}", self.mac_address);

        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e)); // Box the error before returning
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

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to device with MAC={}", self.mac_address);
        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e)); // Box the error before returning
        }
        info!("Connected to device with MAC={}", self.mac_address);
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e)); // Box the error before returning
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }
        Ok(())
    }

    pub async fn retrieve_temperature_and_humidity(&self) -> Result<(f32, f32), Box<dyn std::error::Error>> {
        if let Err(e) = self.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                format!("Failed to connect to device {}: {}", self.mac_address, e),
            )));
        }
    
        let temperature_uuid = "00000002-0000-1000-8000-00805f9b34fb";
        let humidity_uuid = "00000004-0000-1000-8000-00805f9b34fb";
    
        // Attempt to read the temperature characteristic
        let temperature = match self.read_characteristic(temperature_uuid).await {
            Ok(value) => value,
            Err(e) => {
                self.disconnect().await?;
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to retrieve temperature: {}", e),
                )));
            }
        };
    
        // Attempt to read the humidity characteristic
        let humidity = match self.read_characteristic(humidity_uuid).await {
            Ok(value) => value,
            Err(e) => {
                self.disconnect().await?;
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to retrieve humidity: {}", e),
                )));
            }
        };
    
        let temp_value = Self::parse_temperature(&temperature);
        let hum_value = Self::parse_humidity(&humidity);
    
        self.disconnect().await?;
        
        Ok((temp_value, hum_value))
    }

    async fn read_characteristic(&self, uuid: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let characteristic = self.find_characteristic(uuid).ok_or_else(|| {
            let error_msg = format!("Characteristic with UUID {} not found", uuid);
            warn!("{}", error_msg);
            std::io::Error::new(std::io::ErrorKind::NotFound, error_msg)
        })?;

        match self.peripheral.read(&characteristic).await {
            Ok(value) => Ok(value),
            Err(e) => {
                warn!("Failed to read characteristic with UUID {}: {:?}", uuid, e);
                Err(Box::new(e))
            }
        }
    }

    fn find_characteristic(&self, uuid: &str) -> Option<btleplug::api::Characteristic> {
        for service in self.peripheral.services() {
            for characteristic in &service.characteristics {
                if characteristic.uuid.to_string() == uuid {
                    return Some(characteristic.clone());
                }
            }
        }
        None
    }

    fn parse_temperature(value: &[u8]) -> f32 {
        let raw_value = i16::from_le_bytes([value[0], value[1]]);
        raw_value as f32 / 100.0
    }

    fn parse_humidity(value: &[u8]) -> f32 {
        let raw_value = i16::from_le_bytes([value[0], value[1]]);
        raw_value as f32 / 100.0
    }
}
