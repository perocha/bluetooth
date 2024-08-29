use btleplug::platform::Peripheral;
use btleplug::api::{Peripheral as PeripheralTrait, CharPropFlags};
use log::{info, warn, debug, error};
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
        self.connect().await?;

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
    
        self.disconnect().await?;
        Ok(())
    }

    pub async fn retrieve_additional_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connect().await?;

        info!("Connected to device with MAC={}", self.mac_address);
    
        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
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
    
        self.disconnect().await?;
        Ok(())
    }

    // Helper method to connect with retry logic and exponential backoff
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to device with MAC={}", self.mac_address);
        for attempt in 1..=3 {
            if self.peripheral.connect().await.is_ok() {
                info!("Connected to device with MAC={}", self.mac_address);
                return Ok(());
            } else {
                warn!("Attempt {}/3: Failed to connect to device {}", attempt, self.mac_address);
                tokio::time::sleep(std::time::Duration::from_secs(2_u64.pow(attempt))).await; // Exponential backoff
            }
        }
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to connect after multiple attempts")))
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

    pub async fn subscribe_to_mj_ht_v1_notifications(&self) -> Result<(), Box<dyn std::error::Error>> {
        let max_retries = 3;
        let mut attempt = 0;
    
        while attempt < max_retries {
            attempt += 1;
    
            // Ensure the device is connected
            if !self.peripheral.is_connected().await? {
                self.connect().await?;
            }
    
            // Introduce a longer delay to ensure the device is ready
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            // Subscribe to temperature notifications
            let temperature_uuid = "226caa55-6476-4566-7562-66734470666d";
            let service_uuid = "226c0000-6476-4566-7562-66734470666d";
    
            match self.subscribe_to_notifications(service_uuid, temperature_uuid).await {
                Ok(_) => {
                    info!("Successfully subscribed to temperature notifications.");
                }
                Err(e) => {
                    warn!("Attempt {}/{}: Failed to subscribe to temperature notifications: {:?}", attempt, max_retries, e);
                    if attempt == max_retries {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to subscribe after {} attempts: {}", max_retries, e),
                        )));
                    }
                    continue; // Retry subscription
                }
            }
    
            // Subscribe to humidity notifications
            let humidity_uuid = "226cbb55-6476-4566-7562-66734470666d";
    
            match self.subscribe_to_notifications(service_uuid, humidity_uuid).await {
                Ok(_) => {
                    info!("Successfully subscribed to humidity notifications.");
                    return Ok(());
                }
                Err(e) => {
                    warn!("Attempt {}/{}: Failed to subscribe to humidity notifications: {:?}", attempt, max_retries, e);
                    if attempt == max_retries {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to subscribe after {} attempts: {}", max_retries, e),
                        )));
                    }
                }
            }
        }
    
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to subscribe to notifications after multiple attempts",
        )))
    }
    
    async fn subscribe_to_notifications(&self, service_uuid: &str, characteristic_uuid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let characteristic = self.find_characteristic(service_uuid, characteristic_uuid).ok_or_else(|| {
            let error_msg = format!("Characteristic with UUID {} not found in service {}", characteristic_uuid, service_uuid);
            warn!("{}", error_msg);
            std::io::Error::new(std::io::ErrorKind::NotFound, error_msg)
        })?;
    
        // Check if the characteristic has the Notify property
        if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
            let error_msg = format!("Characteristic with UUID {} does not support notifications", characteristic_uuid);
            warn!("{}", error_msg);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_msg)));
        }
    
        for attempt in 1..=3 {
            if !self.peripheral.is_connected().await? {
                info!("Connecting to device...");
                self.peripheral.connect().await?;
            }
    
            match self.peripheral.subscribe(&characteristic).await {
                Ok(_) => {
                    info!("Successfully subscribed to characteristic with UUID {}", characteristic_uuid);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Attempt {}/3: Failed to subscribe to characteristic with UUID {}: {:?}", attempt, characteristic_uuid, e);
                    if attempt < 3 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    } else {
                        return Err(Box::new(e));
                    }
                }
            }
        }
    
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to subscribe after 3 attempts")))
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
    
    // Improved method to read characteristic with retry and delay logic
    pub async fn read_characteristic(&self, service_uuid: &str, characteristic_uuid: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let characteristic = self.find_characteristic(service_uuid, characteristic_uuid).ok_or_else(|| {
            let error_msg = format!("Characteristic with UUID {} not found", characteristic_uuid);
            warn!("{}", error_msg);
            std::io::Error::new(std::io::ErrorKind::NotFound, error_msg)
        })?;

        // Add a slight delay before attempting to read
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        for attempt in 1..=3 {
            if !self.peripheral.is_connected().await? {
                info!("Not connected to device, reconnecting to device...");
                let connect_result = self.connect().await;
                // Log the result of the connect method
                match &connect_result {
                    Ok(_) => info!("Successfully connected to the device."),
                    Err(e) => error!("Failed to connect to the device: {:?}", e),
                }
                // Propagate the result of the connect method
                connect_result?;
            }

            match self.peripheral.read(&characteristic).await {
                Ok(value) => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await; // Delay between reads
                    return Ok(value);
                }
                Err(e) => {
                    warn!("Attempt {}/3: Failed to read characteristic with UUID {}: {:?}", attempt, characteristic_uuid, e);
                    if attempt < 3 {
                        tokio::time::sleep(std::time::Duration::from_secs(2_u64.pow(attempt))).await; // Exponential backoff
                    } else {
                        return Err(Box::new(e));
                    }
                }
            }
        }
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read characteristic after 3 attempts")))
    }

    fn parse_temperature(value: &[u8]) -> f32 {
        let raw_value = i16::from_le_bytes([value[0], value[1]]);
        raw_value as f32 / 100.0
    }

    fn parse_humidity(value: &[u8]) -> f32 {
        let raw_value = i16::from_le_bytes([value[2], value[3]]); // Assuming humidity is in the next two bytes
        raw_value as f32 / 100.0
    }

    pub async fn read_mj_ht_v1_information(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connect().await?;

        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services: {:?}", e);
            return Err(Box::new(e));
        }

        let characteristics = vec![
            ("Device Name", "00001800-0000-1000-8000-00805f9b34fb", "00002a00-0000-1000-8000-00805f9b34fb"),
            ("Appearance", "00001800-0000-1000-8000-00805f9b34fb", "00002a01-0000-1000-8000-00805f9b34fb"),
            ("Peripheral Preferred Connection Parameters", "00001800-0000-1000-8000-00805f9b34fb", "00002a04-0000-1000-8000-00805f9b34fb"),
            ("Firmware Version", "0000180a-0000-1000-8000-00805f9b34fb", "00002a26-0000-1000-8000-00805f9b34fb"),
            ("Manufacturer Name", "0000180a-0000-1000-8000-00805f9b34fb", "00002a29-0000-1000-8000-00805f9b34fb"),
            ("Battery Level", "0000180f-0000-1000-8000-00805f9b34fb", "00002a19-0000-1000-8000-00805f9b34fb"),
        ];

        for (name, service_uuid, characteristic_uuid) in characteristics {
            match self.read_characteristic(service_uuid, characteristic_uuid).await {
                Ok(value) => {
                    let output = match name {
                        "Device Name" | "Firmware Version" | "Manufacturer Name" => {
                            String::from_utf8_lossy(&value).to_string()
                        }
                        "Appearance" | "Peripheral Preferred Connection Parameters" => {
                            format!("{:?}", value)
                        }
                        "Battery Level" => {
                            format!("{}%", value[0])
                        }
                        _ => format!("{:?}", value),
                    };
                    println!("{}: {}", name, output);
                }
                Err(e) => {
                    println!("Failed to read {}: {:?}", name, e);
                }
            }
        }

        self.disconnect().await?;
        Ok(())
    }
}
