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
        info!("Starting Bluetooth scan...");
        self.adapter.start_scan(ScanFilter::default()).await?;
        time::sleep(Duration::from_secs(5)).await;

        let peripherals = self.adapter.peripherals().await?;
        info!("Found {} peripherals", peripherals.len());
        let mut devices = Vec::new();

        for peripheral in peripherals {
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

            info!("Discovered device: {} ({:?})", device_name, peripheral.id());
            devices.push(BluetoothDevice::new(peripheral.id(), device_name));
        }

        Ok(devices)
    }

    pub async fn pair_with_device(&self, device: &BluetoothDevice) -> Result<(), Box<dyn Error>> {
        info!("Attempting to pair with device: {} ({:?})", device.name, device.id);
        let peripheral = self.adapter.peripheral(&device.id).await?;
        peripheral.connect().await?;
        info!("Successfully paired with device: {}", device.name);
        Ok(())
    }
}
