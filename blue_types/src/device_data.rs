use crate::MacAddress;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceData {
    ///MAC-address for device
    pub address: MacAddress,
    pub name: Option<String>,
    pub rssi: i8,
}
