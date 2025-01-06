#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MacAddress([u8; 6]);

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
