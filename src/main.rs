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
                if let Err(e) = bluetooth_manager.scan(&mut device_storage, attempts).await {
                    error!("Failed to perform scan: {}", e);
                }
            }
            2 => {
                info!("User requested to list devices");
                ui.display_devices(&device_storage);
            }
            3 => {
                let device_id = ui.get_device_id();
                info!("User requested to retrieve config information for device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.list_available_info(device_id, &device_storage).await {
                    error!("Failed to retrieve available information: {}", e);
                }
            }
            4 => {
                let device_id = ui.get_device_id();
                info!("User requested to retrieve detailed information for device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.retrieve_device_info(device_id, &device_storage).await {
                    error!("Failed to retrieve device information: {}", e);
                }
            }
            5 => {
                let device_id = ui.get_device_id();
                info!("Get temperature and humidity data from MJ_HT_V1 sensor with device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.retrieve_temperature_and_humidity(device_id, &device_storage).await {
                    error!("Failed to retrieve temperature and humidity: {}", e);
                } else {
                    info!("Successfully retrieved temperature and humidity.");
                }
            }
            6 => {
                let device_id = ui.get_device_id();
                info!("Get all data from MJ_HT_V1 sensor with device ID: {}", device_id);
                if let Err(e) = bluetooth_manager.print_all_characteristics(device_id, &device_storage).await {
                    error!("Failed to retrieve all data: {}", e);
                } else {
                    info!("Successfully retrieved all data.");
                }
            }
            7 => {
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
