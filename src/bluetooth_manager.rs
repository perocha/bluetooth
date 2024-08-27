use btleplug::api::{Central, Manager as _, Peripheral as PeripheralTrait, ScanFilter};
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
            self.adapter.start_scan(ScanFilter::default()).await?;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let peripherals = self.adapter.peripherals().await?;

            for peripheral in peripherals {
                if let Some(device) = self.create_bluetooth_device(peripheral).await {
                    storage.add_or_update_device(device);
                }
            }
        }
        info!("Scan completed.");
        Ok(())
    }

    pub async fn retrieve_device_info(&self, device_id: u32, storage: &DeviceStorage) -> Result<(), Box<dyn std::error::Error>> {
        self.with_device(device_id, storage, |device| async move {
            info!("Retrieving detailed information...");
            device.retrieve_additional_info().await;
            Ok(())
        }).await
    }
    
    pub async fn list_available_info(&self, device_id: u32, storage: &DeviceStorage) -> Result<(), Box<dyn std::error::Error>> {
        self.with_device(device_id, storage, |device| async move {
            info!("Listing available information...");
            device.list_available_info().await?;
            Ok(())
        }).await
    }
    
    pub async fn retrieve_temperature_and_humidity(&self, device_id: u32, storage: &DeviceStorage) -> Result<(), Box<dyn std::error::Error>> {
        self.with_device(device_id, storage, |device| async move {
            info!("Retrieving temperature and humidity...");
            let (temp, hum) = device.retrieve_temperature_and_humidity().await?;
            info!("Temperature: {:.2}°C, Humidity: {:.2}%", temp, hum);
            Ok(())
        }).await
    }

    pub async fn print_all_characteristics(&self, device_id: u32, storage: &DeviceStorage) -> Result<(), Box<dyn std::error::Error>> {
        self.with_device(device_id, storage, |device| async move {
            info!("Printing all characteristics...");
            device.print_all_characteristics().await?;
            Ok(())
        }).await
    }
    
    /// Helper method to reduce code duplication when working with devices.
    async fn with_device<F, Fut>(
        &self,
        device_id: u32,
        storage: &DeviceStorage,
        f: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(Arc<BluetoothDevice>) -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        if let Some(device) = storage.get_device(device_id).map(|d| Arc::new(d.clone())) {
            f(device).await
        } else {
            Err("Device not found".into())
        }
    }
    
    /// Helper method to create a BluetoothDevice from a peripheral.
    async fn create_bluetooth_device(&self, peripheral: btleplug::platform::Peripheral) -> Option<BluetoothDevice> {
        let properties = peripheral.properties().await.ok()?;
        let name = properties.as_ref().and_then(|props| props.local_name.clone()).unwrap_or("Unknown Device".to_string());
        let rssi = properties.as_ref().and_then(|props| props.rssi).unwrap_or(0);
        let mac_address = peripheral.id().to_string();

        debug!("Device found: MAC={}, Name={}, RSSI={}", mac_address, name, rssi);

        Some(BluetoothDevice::new(mac_address, name, rssi, Arc::new(peripheral)))
    }
}
