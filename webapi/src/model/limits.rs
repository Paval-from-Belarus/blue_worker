use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLimits {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub epoch_start: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub epoch_end: DateTime<Utc>,
}
