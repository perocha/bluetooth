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

    pub fn list_devices(&self) -> Vec<(u32, &BluetoothDevice)> {
        debug!("Listing all devices...");
        // Return a vector of tuples containing the internal ID and a reference to the device
        self.devices.iter().map(|(&id, device)| (id, device)).collect()
    }

    /// Lists only devices that are MJ_HT_V1 sensors.
    pub fn list_mj_ht_v1_devices(&self) -> Vec<(u32, &BluetoothDevice)> {
        debug!("Listing all MJ_HT_V1 devices...");
        // Filter the devices where the name or other criteria match MJ_HT_V1 sensors.
        self.devices.iter()
            .filter(|(_, device)| device.name.contains("MJ_HT_V1"))
            .map(|(&id, device)| (id, device))
            .collect()
    }

    // Count the number of devices with a specific name
    pub fn count_devices_by_name(&self, name: &str) -> usize {
        self.devices.values().filter(|d| d.name == name).count()
    }
}
