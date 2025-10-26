use crate::DeviceData;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Scan {
    ///duration in millis for scan
    pub duration: u64,
    #[cfg(feature = "alloc")]
    pub devices: alloc::vec::Vec<DeviceData>,
}

#[cfg(feature = "alloc")]
impl Scan {
    pub fn to_bytes(&self) -> alloc::vec::Vec<u8> {
        bitcode::serialize(self).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Scan> {
        bitcode::deserialize(bytes).ok()
    }

    pub fn to_json(&self) -> alloc::string::String {
        serde_json::to_string(self).expect("Scan is valid for json")
    }

    pub fn from_json(&self, json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}
