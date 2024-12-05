use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceData {
    pub mac_address: String,
    pub name: String,

    pub lifetime: Vec<DeviceLifetimeStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLifetimeStep {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_start: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_end: DateTime<Utc>,
    /// distance to the device
    pub distance: f32,
}

impl DeviceLifetimeStep {
    pub fn time_start_millis(&self) -> u64 {
        self.time_start.timestamp_millis() as u64
    }

    pub fn time_end_millis(&self) -> u64 {
        self.time_end.timestamp_millis() as u64
    }
}
