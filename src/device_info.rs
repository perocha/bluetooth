use btleplug::platform::Peripheral;
use btleplug::api::{Peripheral as PeripheralTrait, CharPropFlags};
use log::{info, warn, debug};
use std::sync::Arc;
use tokio_stream::StreamExt;

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

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to device with MAC={}", self.mac_address);
        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }
        info!("Connected to device with MAC={}", self.mac_address);
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }
        Ok(())
    }

    pub async fn retrieve_temperature_and_humidity(&self) -> Result<(f32, f32), Box<dyn std::error::Error>> {
        let max_retries = 3;
        let mut attempt = 0;
    
        while attempt < max_retries {
            attempt += 1;
            match self.connect().await {
                Ok(_) => break,
                Err(e) => {
                    warn!("Attempt {}/{}: Failed to connect to device {}: {:?}", attempt, max_retries, self.mac_address, e);
                    if attempt == max_retries {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::NotConnected,
                            format!("Failed to connect to device {} after {} attempts: {}", self.mac_address, max_retries, e),
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    
        let service_uuid = "226c0000-6476-4566-7562-66734470666d";
        let temperature_uuid = "226caa55-6476-4566-7562-66734470666d";
        let humidity_uuid = "226cbb55-6476-4566-7562-66734470666d";
    
        // Adding delay to ensure device is ready
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
        // Attempt to subscribe to the temperature notifications
        if let Err(e) = self.subscribe_to_notifications(service_uuid, temperature_uuid).await {
            self.disconnect().await?;
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to subscribe to temperature notifications: {}", e),
            )));
        }
    
        // Attempt to subscribe to the humidity notifications
        if let Err(e) = self.subscribe_to_notifications(service_uuid, humidity_uuid).await {
            self.disconnect().await?;
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to subscribe to humidity notifications: {}", e),
            )));
        }
    
        // Wait for notifications and process them
        let mut temperature: Option<f32> = None;
        let mut humidity: Option<f32> = None;
        let mut notification_stream = self.peripheral.notifications().await?;
    
        while let Some(notification) = notification_stream.next().await {
            if notification.uuid.to_string() == temperature_uuid {
                temperature = Some(Self::parse_temperature(&notification.value));
            } else if notification.uuid.to_string() == humidity_uuid {
                humidity = Some(Self::parse_humidity(&notification.value));
            }
    
            // Exit the loop once both values are received
            if temperature.is_some() && humidity.is_some() {
                break;
            }
        }
    
        self.disconnect().await?;
    
        if let (Some(temp), Some(hum)) = (temperature, humidity) {
            Ok((temp, hum))
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to retrieve temperature and humidity from notifications",
            )))
        }
    }
    
    async fn subscribe_to_notifications(&self, service_uuid: &str, characteristic_uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let characteristic = self.find_characteristic(service_uuid, characteristic_uuid).ok_or_else(|| {
            let error_msg = format!("Characteristic with UUID {} not found in service {}", characteristic_uuid, service_uuid);
            warn!("{}", error_msg);
            std::io::Error::new(std::io::ErrorKind::NotFound, error_msg)
        })?;
    
        self.peripheral.subscribe(&characteristic).await.map_err(|e| {
            warn!("Failed to subscribe to characteristic with UUID {}: {:?}", characteristic_uuid, e);
            Box::new(e) as Box<dyn std::error::Error>
        })
    }
    
    fn find_characteristic(&self, service_uuid: &str, characteristic_uuid: &str) -> Option<btleplug::api::Characteristic> {
        for service in self.peripheral.services() {
            if service.uuid.to_string() == service_uuid {
                for characteristic in &service.characteristics {
                    if characteristic.uuid.to_string() == characteristic_uuid {
                        return Some(characteristic.clone());
                    }
                }
            }
        }
        None
    }
    
    async fn read_characteristic(&self, service_uuid: &str, characteristic_uuid: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let characteristic = self.find_characteristic(service_uuid, characteristic_uuid).ok_or_else(|| {
            let error_msg = format!("Characteristic with UUID {} not found", characteristic_uuid);
            warn!("{}", error_msg);
            std::io::Error::new(std::io::ErrorKind::NotFound, error_msg)
        })?;

        match self.peripheral.read(&characteristic).await {
            Ok(value) => Ok(value),
            Err(e) => {
                warn!("Failed to read characteristic with UUID {}: {:?}", characteristic_uuid, e);
                Err(Box::new(e))
            }
        }
    }

    fn parse_temperature(value: &[u8]) -> f32 {
        let raw_value = i16::from_le_bytes([value[0], value[1]]);
        raw_value as f32 / 100.0
    }

    fn parse_humidity(value: &[u8]) -> f32 {
        let raw_value = i16::from_le_bytes([value[2], value[3]]); // Assuming humidity is in the next two bytes
        raw_value as f32 / 100.0
    }
}
