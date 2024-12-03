use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

pub struct DeviceSharedState {}

pub type DeviceId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceData {
    /// unique id for device
    /// This id is assigned according to the value of device mac
    /// and lifetime appereance
    pub id: DeviceId,
    /// lifetime of device
    pub duration: u64,
    /// distance to the device
    pub distance: f32,
    pub mac_address: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSnapshot {
    // #[serde(with = "chrono::")]
    pub time_start: DateTime<Utc>,
    pub time_end: DateTime<Utc>,

    pub ids: Vec<DeviceId>,

    pub devices: HashMap<DeviceId, Vec<DeviceData>>,
}

impl DeviceSharedState {
    pub async fn take_snapshot(
        &self,
        time_start: DateTime<Utc>,
        time_end: DateTime<Utc>,
    ) -> Option<TimeSnapshot> {
        None
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
