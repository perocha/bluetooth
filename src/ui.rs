use crate::device_storage::DeviceStorage;

pub struct UserInterface;

impl UserInterface {
    pub fn new() -> Self {
        UserInterface {}
    }

    pub fn display_menu(&self) {
        println!("1. Scan");
        println!("2. List devices");
        println!("3. List MJ_HT_V1 devices");
        println!("4. Retrieve config info");
        println!("5. Retrieve detailed info");
        println!("6. Retrieve temperature and humidity data");
        println!("7. Retrieve all data");
        println!("10. Exit");
    }

    pub fn get_user_choice(&self) -> u8 {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().parse().expect("Please enter a valid number")
    }

    pub fn get_scan_attempts(&self) -> u8 {
        println!("Enter the number of scan attempts:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().parse().expect("Please enter a valid number")
    }

    pub fn display_devices(&self, storage: &DeviceStorage) {
        for (id, device) in storage.list_devices() {
            println!("ID: {}, MAC: {}, Name: {}, RSSI: {}", id, device.mac_address, device.name, device.rssi);
        }
    }

    // Display only MJ_HT_V1 devices
    pub fn display_mj_ht_v1_devices(&self, storage: &DeviceStorage) {
        for (id, device) in storage.list_mj_ht_v1_devices() {
            println!("ID: {}, MAC: {}, Name: {}, RSSI: {}", id, device.mac_address, device.name, device.rssi);
        }
    }

    pub fn get_device_id(&self) -> u32 {
        println!("Enter the internal ID of the device:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().parse().expect("Please enter a valid number")
    }
}
