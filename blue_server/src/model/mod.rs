use std::sync::Arc;

use tokio::sync::RwLock;

mod device_data;
mod error;
mod limits;
mod shared_data;
mod snapshot;

pub use device_data::*;
pub use error::*;
pub use limits::*;
pub use shared_data::*;
pub use snapshot::*;

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
