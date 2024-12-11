use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceData {
    ///MAC-address for device
    pub address: MacAddress,
    pub name: Option<String>,
    pub rssi: i8,
}

impl From<[u8; 6]> for MacAddress {
    fn from(value: [u8; 6]) -> Self {
        Self(value)
    }
}

impl MacAddress {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl ToString for MacAddress {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(":")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scan {
    ///duration in millis for scan
    pub duration: u64,
    pub devices: Vec<DeviceData>,
}

impl Scan {
    pub fn to_vec(&self) -> Vec<u8> {
        bitcode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Scan> {
        bitcode::deserialize(bytes).ok()
    }
}
