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


fn rssi_to_distance(rssi: f32) -> f32 {
    const A: f32 = -50.0; // RSSI value at 1 meter; adjust based on testing.
    const N: f32 = 2.0; // Path loss exponent; adjust depending on the environment.

    // Calculate distance in meters
    if rssi >= A {
        return 1.0; // If RSSI is stronger than A, assume distance is less than 1 meter
    }

    10.0f32.powf((A - rssi) / (10.0 * N))
}

impl From<blue_types::DeviceData> for DeviceData {
    fn from(value: blue_types::DeviceData) -> Self {
        
    }
}
