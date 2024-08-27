use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
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
                // Skip if this peripheral is already added to the list
                if all_devices.iter().any(|d: &BluetoothDevice| d.id == peripheral.id()) {
                    continue;
                }

                let properties = peripheral.properties().await?;
                let name = if let Some(props) = &properties {
                    props.local_name.clone()
                } else {
                    None
                };

                let device_name = match name {
                    Some(name) => name,
                    None => {
                        info!("Attempting to connect to peripheral {:?} to retrieve name...", peripheral.id());
                        if peripheral.connect().await.is_ok() {
                            peripheral.discover_services().await.ok();
                            let updated_properties = peripheral.properties().await?;
                            peripheral.disconnect().await.ok();

                            if let Some(props) = updated_properties {
                                props.local_name.unwrap_or("Unknown Device".to_string())
                            } else {
                                "Unknown Device".to_string()
                            }
                        } else {
                            warn!("Failed to connect to peripheral {:?}", peripheral.id());
                            "Unknown Device".to_string()
                        }
                    }
                };

                // Only add devices with the name "MJ_HT_V1"
                if device_name == "MJ_HT_V1" {
                    let address = peripheral.id().to_string();
                    let signal_strength = properties.and_then(|props| props.rssi); // Assuming rssi is available

                    info!("Discovered device: {} ({:?})", device_name, peripheral.id());
                    all_devices.push(BluetoothDevice::new(peripheral.id(), device_name, address, signal_strength));
                }
            }
        }

        if all_devices.is_empty() {
            info!("No devices named 'MJ_HT_V1' found.");
        } else {
            info!("Total devices named 'MJ_HT_V1' found: {}", all_devices.len());
        }

        Ok(all_devices)
    }

    pub async fn pair_with_device(&self, device: &BluetoothDevice) -> Result<(), Box<dyn Error>> {
        info!("Attempting to pair with device: {} ({:?})", device.name, device.id);
        let peripheral = self.adapter.peripheral(&device.id).await?;
        peripheral.connect().await?;
        info!("Successfully paired with device: {}", device.name);
        Ok(())
    }
}
