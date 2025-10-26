use crate::MacAddress;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceData {
    ///MAC-address for device
    pub address: MacAddress,
    #[cfg(feature = "alloc")]
    pub name: Option<alloc::string::String>,
    pub rssi: i8,
}
