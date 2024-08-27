use btleplug::api::{Central, Manager as _, Peripheral as PeripheralTrait}; // Import PeripheralTrait
use btleplug::platform::Adapter;
use std::error::Error;
use std::sync::Arc;
use crate::device_storage::DeviceStorage;
use crate::device_info::BluetoothDevice;
use log::{info, debug};

pub struct BluetoothManager {
    adapter: Adapter,
}

impl BluetoothManager {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        info!("Creating new BluetoothManager instance...");
        let manager = btleplug::platform::Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters.into_iter().next().ok_or("No Bluetooth adapter found")?;
        info!("Bluetooth adapter found: {:?}", adapter.adapter_info().await?);
        Ok(BluetoothManager { adapter })
    }

    pub async fn scan(&self, storage: &mut DeviceStorage, attempts: u8) -> Result<(), Box<dyn Error>> {
        info!("Starting scan with {} attempt(s)...", attempts);
        for attempt in 1..=attempts {
            info!("Scan attempt {}/{}", attempt, attempts);
            self.adapter.start_scan(btleplug::api::ScanFilter::default()).await?;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let peripherals = self.adapter.peripherals().await?;

            for peripheral in peripherals {
                let properties = peripheral.properties().await?;
                let name = properties.as_ref().and_then(|props| props.local_name.clone()).unwrap_or("Unknown Device".to_string());
                let rssi = properties.as_ref().and_then(|props| props.rssi).unwrap_or(0);
                let mac_address = peripheral.id().to_string();

                debug!("Device found: MAC={}, Name={}, RSSI={}", mac_address, name, rssi);

                // Add the device with its peripheral to the storage
                let device = BluetoothDevice::new(mac_address, name, rssi, Arc::new(peripheral));
                storage.add_or_update_device(device);
            }
        }

        info!("Scan completed.");
        Ok(())
    }

    pub async fn retrieve_device_info(&self, device_id: u32, storage: &DeviceStorage) -> Result<(), Box<dyn Error>> {
        info!("Retrieving information for device with ID: {}", device_id);
        if let Some(device) = storage.get_device(device_id) {
            info!("Device found: {:?}", device);
            device.retrieve_additional_info().await;
        } else {
            info!("Device with ID {} not found.", device_id);
        }

        Ok(())
    }
}
