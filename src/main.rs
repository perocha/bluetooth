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
                let duration = ui.get_scan_duration();
                info!("User requested a scan with {} attempt(s) and a duration of {} seconds", attempts, duration);
                if let Err(e) = bluetooth_manager.scan(&mut device_storage, duration, attempts).await {
                    error!("Failed to perform scan: {}", e);
                }
            }
            2 => {
                let max_devices = ui.get_max_devices_to_scan();
                info!("User requested to scan for MJ_HT_V1 devices");
                if let Err(e) = bluetooth_manager.scan_for_mj_ht_v1_devices(&mut device_storage, max_devices).await {
                    error!("Failed to scan for MJ_HT_V1 devices: {}", e);
                }
            }
            3 => {
                info!("User requested to list devices");
                ui.display_devices(&device_storage);
            }
            4 => {
                info!("User requested to list MJ_HT_V1 devices");
                ui.display_mj_ht_v1_devices(&device_storage);
            }
            5 => {
                let device_id = ui.get_device_id();
                info!("User requested to retrieve config information for device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.list_available_info(device_id, &device_storage).await {
                    error!("Failed to retrieve available information: {}", e);
                }
            }
            6 => {
                let device_id = ui.get_device_id();
                info!("User requested to retrieve detailed information for device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.retrieve_device_info(device_id, &device_storage).await {
                    error!("Failed to retrieve device information: {}", e);
                }
            }
            7 => {
                let device_id = ui.get_device_id();
                info!("Get temperature and humidity data from MJ_HT_V1 sensor with device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.retrieve_temperature_and_humidity(device_id, &device_storage).await {
                    error!("Failed to retrieve temperature and humidity: {}", e);
                } else {
                    info!("Successfully retrieved temperature and humidity.");
                }
            }
            8 => {
                let device_id = ui.get_device_id();
                info!("Get all data from MJ_HT_V1 sensor with device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.read_mj_ht_v1_information(device_id, &device_storage).await {
                    error!("Failed to retrieve all data: {}", e);
                } else {
                    info!("Successfully retrieved all data.");
                }
            }
            9 => {
                let device_id = ui.get_device_id();
                info!("User requested to connect to device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.connect_device(device_id, &device_storage).await {
                    error!("Failed to connect to device: {}", e);
                }
            }
            10 => {
                let device_id = ui.get_device_id();
                info!("User requested to disconnect from device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.disconnect_device(device_id, &device_storage).await {
                    error!("Failed to disconnect from device: {}", e);
                }
            }
            11 => {
                let device_id = ui.get_device_id();
                info!("User requested to discover services from device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.discover_services(device_id, &device_storage).await {
                    error!("Failed to discover services: {}", e);
                }
            }
            12 => {
                let device_id = ui.get_device_id();
                info!("User requested to read characteristic from device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.read_mj_ht_v1(device_id, &device_storage).await {
                    error!("Failed to read sensor: {}", e);
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
