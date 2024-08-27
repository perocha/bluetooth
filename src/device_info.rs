use btleplug::platform::Peripheral; // Use the concrete type directly
use btleplug::api::Peripheral as PeripheralTrait; // Import the necessary trait for Peripheral methods
use log::{info, warn, debug}; // Import logging macros
use std::sync::Arc;

#[derive(Debug)]
pub struct BluetoothDevice {
    pub mac_address: String,
    pub name: String,
    pub rssi: i16,
    pub peripheral: Arc<Peripheral>, // Reference to the associated peripheral
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

        // Connect to the peripheral
        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }

        info!("Connected to device with MAC={}", self.mac_address);

        // Discover services
        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return Err(Box::new(e));
        }

        // Iterate through available services and list them
        for service in self.peripheral.services() {
            info!("Service UUID: {:?}", service.uuid);

            // List characteristics of each service
            for characteristic in &service.characteristics {
                info!("Characteristic UUID: {:?}, Properties: {:?}", characteristic.uuid, characteristic.properties);
            }
        }

        // Disconnect from the device when done
        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }

        Ok(())
    }

    pub async fn retrieve_additional_info(&self) {
        info!("Connecting to device with MAC={}", self.mac_address);

        // Connect to the peripheral
        if let Err(e) = self.peripheral.connect().await {
            warn!("Failed to connect to device {}: {:?}", self.mac_address, e);
            return;
        }

        info!("Connected to device with MAC={}", self.mac_address);

        // Discover services
        if let Err(e) = self.peripheral.discover_services().await {
            warn!("Failed to discover services on device {}: {:?}", self.mac_address, e);
            return;
        }

        // Iterate through available services
        for service in self.peripheral.services() {
            info!("Service UUID: {:?}", service.uuid);

            // Iterate through characteristics of each service
            for characteristic in service.characteristics {
                info!("Characteristic UUID: {:?}", characteristic.uuid);

                // If the characteristic supports reading, try to read its value
                if characteristic.properties.contains(btleplug::api::CharPropFlags::READ) {
                    match self.peripheral.read(&characteristic).await {
                        Ok(value) => {
                            info!("Read value from characteristic {:?}: {:?}", characteristic.uuid, value);
                            // Here, we could parse the value depending on the characteristic's expected format
                        }
                        Err(err) => {
                            warn!("Failed to read characteristic {:?}: {:?}", characteristic.uuid, err);
                        }
                    }
                }
            }
        }

        // Disconnect from the device when done
        if let Err(e) = self.peripheral.disconnect().await {
            warn!("Failed to disconnect from device {}: {:?}", self.mac_address, e);
        } else {
            info!("Disconnected from device with MAC={}", self.mac_address);
        }
    }
}
