use std::collections::HashMap;
use crate::device_info::BluetoothDevice;
use log::{info, debug}; // Import the logging macros

pub struct DeviceStorage {
    devices: HashMap<u32, BluetoothDevice>,
    next_id: u32,
}

impl DeviceStorage {
    pub fn new() -> Self {
        debug!("Initializing new DeviceStorage.");
        DeviceStorage {
            devices: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_or_update_device(&mut self, device: BluetoothDevice) {
        for (id, existing_device) in self.devices.iter_mut() {
            if existing_device.mac_address == device.mac_address {
                info!("Updating existing device with ID: {} | MAC: {}", id, existing_device.mac_address);
                existing_device.name = device.name;
                existing_device.rssi = device.rssi;
                return;
            }
        }

        info!("Adding new device with MAC: {}", device.mac_address);
        self.devices.insert(self.next_id, device);
        self.next_id += 1;
    }

    pub fn get_device(&self, id: u32) -> Option<&BluetoothDevice> {
        debug!("Fetching device with ID: {}", id);
        self.devices.get(&id)
    }

    pub fn list_devices(&self) -> Vec<&BluetoothDevice> {
        debug!("Listing all devices. Total count: {}", self.devices.len());
        self.devices.values().collect()
    }
}
