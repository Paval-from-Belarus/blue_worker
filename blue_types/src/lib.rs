#[derive(serde::Serialize, serde::Deserialize)]
pub struct MacAddress([u8; 6]);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeviceData {
    pub timestamp: u64,
    ///MAC-address for device
    pub address: MacAddress,
    pub name: String,
    pub rssi: u8,
}
