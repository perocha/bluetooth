use crate::device_storage::DeviceStorage;

pub struct UserInterface;

impl UserInterface {
    pub fn new() -> Self {
        UserInterface {}
    }

    pub fn display_menu(&self) {
        println!("1. Scan");
        println!("2. Scan for MJ_HT_V1 devices");
        println!("3. List devices");
        println!("4. List MJ_HT_V1 devices");
        println!("5. Retrieve config info");
        println!("6. Retrieve detailed info");
        println!("7. Retrieve temperature and humidity data");
        println!("8. Retrieve all data");
        println!("9. Connect to device");
        println!("10. Disconnect from device");
        println!("11. Discover services");
        println!("12. Read characteristic");
        println!("20. Exit");
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

    pub fn get_scan_duration(&self) -> u8 {
        println!("Enter the scan duration in seconds:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().parse().expect("Please enter a valid number")
    }

    pub fn get_max_devices_to_scan(&self) -> u8 {
        println!("Enter the maximum number of MJ_HT_V1 devices to scan for:");
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

    // Adjusted method to return a Result<String, std::io::Error>
    pub fn get_service_uuid(&self) -> Result<String, std::io::Error> {
        println!("Enter the service UUID:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
    
    // This method is already correctly returning a String, but adjust for error handling
    pub fn get_characteristic_uuid(&self) -> Result<String, std::io::Error> {
        println!("Enter the characteristic UUID:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }
}
