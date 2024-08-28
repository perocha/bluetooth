mod bluetooth_manager;
mod device_storage;
mod ui;
mod device_info;

use bluetooth_manager::BluetoothManager;
use device_storage::DeviceStorage;
use ui::UserInterface;
use log::{info, debug, error};  // Import the logging macros

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();  // Initialize the logger

    // Initialize Bluetooth Manager, Device Storage, and UI
    info!("Initializing Bluetooth Manager, Device Storage, and UI...");
    let bluetooth_manager = BluetoothManager::new().await?;
    let mut device_storage = DeviceStorage::new();
    let ui = UserInterface::new();

    info!("Starting the main application loop...");
    // Main application loop
    loop {
        ui.display_menu();
        let choice = ui.get_user_choice();
        debug!("User selected menu option: {}", choice);

        match choice {
            1 => {
                let attempts = ui.get_scan_attempts();
                info!("User requested a scan with {} attempt(s)", attempts);
                let duration = ui.get_scan_duration();
                info!("User requested a scan with a duration of {} seconds", duration);
                if let Err(e) = bluetooth_manager.scan(&mut device_storage, duration, attempts).await {
                    error!("Failed to perform scan: {}", e);
                }
            }
            2 => {
                info!("User requested to list devices");
                ui.display_devices(&device_storage);
            }
            3 => {
                info!("User requested to list MJ_HT_V1 devices");
                ui.display_mj_ht_v1_devices(&device_storage);
            }
            4 => {
                let device_id = ui.get_device_id();
                info!("User requested to retrieve config information for device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.list_available_info(device_id, &device_storage).await {
                    error!("Failed to retrieve available information: {}", e);
                }
            }
            5 => {
                let device_id = ui.get_device_id();
                info!("User requested to retrieve detailed information for device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.retrieve_device_info(device_id, &device_storage).await {
                    error!("Failed to retrieve device information: {}", e);
                }
            }
            6 => {
                let device_id = ui.get_device_id();
                info!("Get temperature and humidity data from MJ_HT_V1 sensor with device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.retrieve_temperature_and_humidity(device_id, &device_storage).await {
                    error!("Failed to retrieve temperature and humidity: {}", e);
                } else {
                    info!("Successfully retrieved temperature and humidity.");
                }
            }
            7 => {
                let device_id = ui.get_device_id();
                info!("Get all data from MJ_HT_V1 sensor with device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.read_mj_ht_v1_information(device_id, &device_storage).await {
                    error!("Failed to retrieve all data: {}", e);
                } else {
                    info!("Successfully retrieved all data.");
                }
            }
            8 => {
                let device_id = ui.get_device_id();
                info!("User requested to connect to device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.connect_device(device_id, &device_storage).await {
                    error!("Failed to connect to device: {}", e);
                }
            }
            9 => {
                let device_id = ui.get_device_id();
                info!("User requested to disconnect from device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.disconnect_device(device_id, &device_storage).await {
                    error!("Failed to disconnect from device: {}", e);
                }
            }
            10 => {
                let device_id = ui.get_device_id();
                let service_uuid = ui.get_service_uuid()?.to_string();
                let characteristic_uuid = ui.get_characteristic_uuid()?.to_string();
                info!("User requested to read characteristic {} from service {} of device ID: {}", characteristic_uuid, service_uuid, device_id);

                // Connect to the device before reading the characteristic
                if let Err(e) = bluetooth_manager.connect_device(device_id, &device_storage).await {
                    error!("Failed to connect to device: {}", e);
                    continue;
                }
                // Read the characteristic
                if let Err(e) = bluetooth_manager.read_characteristic(device_id, &device_storage, &service_uuid, &characteristic_uuid).await {
                    error!("Failed to read characteristic: {}", e);
                }
                // Disconnect from the device after reading the characteristic
                if let Err(e) = bluetooth_manager.disconnect_device(device_id, &device_storage).await {
                    error!("Failed to disconnect from device: {}", e);
                }
            }
            20 => {
                info!("User selected exit. Terminating the application...");
                break;
            }
            _ => {
                debug!("User selected an invalid option.");
                println!("Invalid option. Please try again.");
            }
        }
    }

    info!("Application has exited.");
    Ok(())
}
