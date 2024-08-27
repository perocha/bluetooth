use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, CharPropFlags};
use btleplug::platform::{Adapter, Manager};
use tokio::time::{self, Duration};
use std::error::Error;
use log::{info, warn};

use crate::bluetooth_device::BluetoothDevice;

pub struct BluetoothManager {
    adapter: Adapter,
}

impl BluetoothManager {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters.into_iter().next().ok_or("No Bluetooth adapter found")?;
        info!("Bluetooth adapter found: {:?}", adapter.adapter_info().await?);
        Ok(BluetoothManager { adapter })
    }

    pub async fn scan_for_devices(&self) -> Result<Vec<BluetoothDevice>, Box<dyn Error>> {
        let mut all_devices = Vec::new();
        let scan_attempts = 3;
        let scan_duration = Duration::from_secs(5);

        for attempt in 1..=scan_attempts {
            info!("Starting Bluetooth scan attempt {}...", attempt);
            self.adapter.start_scan(ScanFilter::default()).await?;
            time::sleep(scan_duration).await;

            let peripherals = self.adapter.peripherals().await?;
            info!("Found {} peripherals on attempt {}", peripherals.len(), attempt);

            for peripheral in peripherals {
                if all_devices.iter().any(|d: &BluetoothDevice| d.id == peripheral.id()) {
                    continue; // Skip already discovered devices
                }

                let properties = peripheral.properties().await?;
                let name = properties.as_ref().and_then(|props| props.local_name.clone());
                
                let device_name = name.unwrap_or_else(|| "Unknown Device".to_string());
                let address = peripheral.id().to_string();
                let signal_strength = properties.as_ref().and_then(|props| props.rssi);

                info!("Discovered device: {} ({:?})", device_name, peripheral.id());
                all_devices.push(BluetoothDevice::new(peripheral.id(), device_name, address, signal_strength));
            }
        }

        // Once the scan is complete, gather more info for MJ_HT_V1 devices
        for device in &all_devices {
            if device.name == "MJ_HT_V1" {
                match self.adapter.peripheral(&device.id).await {
                    Ok(peripheral) => {
                        if let Err(err) = self.retrieve_device_info(&peripheral).await {
                            warn!("Failed to retrieve additional information for {}: {:?}", device.name, err);
                        }
                    }
                    Err(err) => {
                        warn!("Peripheral not found for device: {:?}, error: {:?}", device.id, err);
                    }
                }
            }
        }

        if all_devices.is_empty() {
            info!("No devices found.");
        } else {
            info!("Total devices found: {}", all_devices.len());
        }

        Ok(all_devices)
    }

    async fn retrieve_device_info(&self, peripheral: &impl Peripheral) -> Result<(), Box<dyn Error>> {
        info!("Attempting to connect to peripheral {:?} to retrieve information...", peripheral.id());

        let mut attempts = 0;
        let max_attempts = 3;
        loop {
            match peripheral.connect().await {
                Ok(_) => break,
                Err(e) if attempts < max_attempts => {
                    attempts += 1;
                    warn!("Failed to connect (attempt {}): {:?}", attempts, e);
                    time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    return Err(format!("Failed to connect after {} attempts: {:?}", max_attempts, e).into());
                }
            }
        }

        peripheral.discover_services().await?;

        let services = peripheral.services();
        for service in services {
            info!("Service UUID: {:?}", service.uuid);

            for characteristic in service.characteristics {
                info!("Characteristic UUID: {:?}", characteristic.uuid);

                if characteristic.properties.contains(CharPropFlags::READ) {
                    match peripheral.read(&characteristic).await {
                        Ok(value) => {
                            info!("Read value: {:?}", value);
                            // Parse and use the value as needed
                        }
                        Err(err) => {
                            warn!("Failed to read characteristic {:?}: {:?}", characteristic.uuid, err);
                        }
                    }
                }
            }
        }

        peripheral.disconnect().await?;
        info!("Disconnected from peripheral {:?}", peripheral.id());

        Ok(())
    }

    pub async fn pair_with_device(&self, device: &BluetoothDevice) -> Result<(), Box<dyn Error>> {
        info!("Attempting to pair with device: {} ({:?})", device.name, device.id);
        let peripheral = self.adapter.peripheral(&device.id).await?;
        peripheral.connect().await?;
        info!("Successfully paired with device: {}", device.name);
        Ok(())
    }
}
