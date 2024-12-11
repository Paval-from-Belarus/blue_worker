use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::DeviceData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSnapshot {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_start: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_end: DateTime<Utc>,

    pub devices: Vec<DeviceData>,
}
