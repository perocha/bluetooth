use btleplug::platform::PeripheralId;

pub struct BluetoothDevice {
    pub id: PeripheralId,
    pub name: String,
    // Add more fields as needed
    pub address: String,
    pub signal_strength: Option<i16>, // Example for RSSI (signal strength)
}

impl BluetoothDevice {
    pub fn new(id: PeripheralId, name: String, address: String, signal_strength: Option<i16>) -> Self {
        BluetoothDevice { id, name, address, signal_strength }
    }
}
