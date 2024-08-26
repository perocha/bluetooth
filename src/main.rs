mod bluetooth_manager;
mod bluetooth_device;

use env_logger;
use std::error::Error;
use std::io::{self, Write};
use tokio::runtime::Runtime;

use bluetooth_manager::BluetoothManager;
use bluetooth_device::BluetoothDevice;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init(); // Initialize the logger

    let rt = Runtime::new()?;
    rt.block_on(async {
        let manager = BluetoothManager::new().await?;

        let devices = manager.scan_for_devices().await?;
        if devices.is_empty() {
            println!("No devices found.");
            return Ok(());
        }

        println!("Found devices:");
        for (i, device) in devices.iter().enumerate() {
            println!("{}: {}", i + 1, device.name);
        }

        let selected_device = get_user_selection(&devices)?;
        manager.pair_with_device(selected_device).await?;
        Ok(())
    })
}

fn get_user_selection(devices: &[BluetoothDevice]) -> Result<&BluetoothDevice, Box<dyn Error>> {
    print!("Select a device to pair with (number): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let selection: usize = input.trim().parse()?;
    if selection == 0 || selection > devices.len() {
        return Err("Invalid selection".into());
    }

    Ok(&devices[selection - 1])
}
