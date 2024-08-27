use std::collections::HashMap;
use crate::device_info::BluetoothDevice;
use log::debug;

pub struct DeviceStorage {
    devices: HashMap<u32, BluetoothDevice>,
    next_id: u32,
}

impl DeviceStorage {
    pub fn new() -> Self {
        DeviceStorage {
            devices: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_or_update_device(&mut self, device: BluetoothDevice) {
        debug!("Adding or updating device with MAC: {}", device.mac_address);

        // Check if the device with the same MAC address already exists
        if let Some((_id, existing_device)) = self.devices.iter_mut()
                                                         .find(|(_, d)| d.mac_address == device.mac_address) {
            // Update the existing device's information
            debug!("Updating existing device with MAC: {}", device.mac_address);
            existing_device.name = device.name;
            existing_device.rssi = device.rssi;
            existing_device.peripheral = device.peripheral.clone(); // Ensure peripheral is updated
        } else {
            // Add new device with a new internal ID
            debug!("Adding new device with MAC: {} as ID: {}", device.mac_address, self.next_id);
            self.devices.insert(self.next_id, device);
            self.next_id += 1;
        }
    }

    pub fn get_device(&self, id: u32) -> Option<&BluetoothDevice> {
        debug!("Retrieving device with ID: {}", id);
        self.devices.get(&id)
    }

    pub fn list_devices(&self) -> Vec<&BluetoothDevice> {
        debug!("Listing all devices...");
        self.devices.values().collect()
    }
}
