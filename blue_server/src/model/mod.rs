use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

use tokio::sync::RwLock;

#[derive(Default)]
pub struct DeviceSharedState {}

pub type DeviceId = u64;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSnapshot {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_start: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub time_end: DateTime<Utc>,

    pub devices: Vec<DeviceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLimits {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub min: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub max: DateTime<Utc>,
}

impl DeviceSharedState {
    pub async fn take_limits(&self) -> Option<TimeLimits> {
        Some(TimeLimits {
            min: Utc::now(),
            max: Utc::now() + Duration::from_secs(10),
        })
    }

    pub async fn take_snapshot(
        &self,
        _time_start: DateTime<Utc>,
        _time_end: DateTime<Utc>,
    ) -> Option<TimeSnapshot> {
        Some(TimeSnapshot {
            time_start: Utc::now(),
            time_end: Utc::now() + Duration::from_millis(1000),
            devices: vec![DeviceData {
                name: "Paval".to_string(),
                mac_address: "1:1:1:1:1".to_string(),

                lifetime: vec![
                    DeviceLifetimeStep {
                        distance: 1.0,
                        time_start: Utc::now(),
                        time_end: Utc::now() + Duration::from_millis(300),
                    },
                    DeviceLifetimeStep {
                        distance: 1.2,
                        time_start: Utc::now() + Duration::from_millis(500),
                        time_end: Utc::now() + Duration::from_millis(800),
                    },
                    DeviceLifetimeStep {
                        distance: 1.0,
                        time_start: Utc::now() + Duration::from_millis(900),
                        time_end: Utc::now() + Duration::from_millis(1000),
                    },
                ],
            }],
        })
    }
}

pub type DeviceStateLock = Arc<RwLock<DeviceSharedState>>;

#[macro_export]
macro_rules! devices_lock {
    ($req: ident) => {
        $req.app_data::<$crate::model::DeviceStateLock>()
            .unwrap()
            .read()
            .await
    };
}

#[macro_export]
macro_rules! devices_mut_lock {
    ($req: ident) => {
        $req.app_data::<$crate::model::DeviceStateLock>()
            .unwrap()
            .write()
            .await
    };
}
