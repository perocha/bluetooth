use btleplug::platform::PeripheralId;

pub struct BluetoothDevice {
    pub id: PeripheralId,
    pub name: String,
}

impl BluetoothDevice {
    pub fn new(id: PeripheralId, name: String) -> Self {
        BluetoothDevice { id, name }
    }
}