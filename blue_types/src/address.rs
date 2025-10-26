#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

impl core::fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MacAddress(")?;
        for (i, byte) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ":")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ")")
    }
}

impl core::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut alloc::fmt::Formatter<'_>) -> alloc::fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ":")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
