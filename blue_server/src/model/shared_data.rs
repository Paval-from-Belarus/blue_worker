use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use blue_types::Scan;
use chrono::{DateTime, Utc};

use super::{DeviceData, DeviceLifetimeStep, StateError, TimeLimits, TimeSnapshot};

#[derive(Default)]
pub struct DeviceSharedState {
    pub devices_map: HashMap<String, DeviceData>,
    pub world_limits: Option<TimeLimits>,
}

impl DeviceSharedState {
    pub async fn put_devices(&mut self, scan: Scan) -> Result<(), StateError> {
        let Scan { duration, devices } = scan;

        let scan_start_time = SystemTime::now() - Duration::from_millis(duration);
        let scan_end_time = SystemTime::now();

        for device in devices.into_iter() {
            let mac_address =
                String::from_utf8(device.address.as_bytes()).expect("Mac address is valid string");

            self.devices_map
                .entry(&mac_address)
                .and_modify(|data| {
                    let new_lifetimes = vec![];

                    if let Some(ref name) = device.name && data.name.is_empty() {
                        data.name = name.clone();
                        //otherwise skip new name
                    }

                    // data.lifetime.extend_from_slice(other);

                    // assert!(data.lifetime.is_sorted_by_key(|e| e.time_start));
                })
                .or_insert(DeviceData {
                    mac_address,
                    name: device.name.or_else(""),
                    lifetime: vec![],
                });
        }

        if let Some(ref mut limits) = self.world_limits {
            limits.epoch_end = scan_end_time;
        } else {
            self.world_limits = Some(TimeLimits {
                epoch_start: scan_start_time,
                epoch_end: scan_end_time,
            });
        }

        Ok(())
    }

    pub async fn take_limits(&self) -> Option<TimeLimits> {
        self.world_limits.clone()
    }

    pub async fn take_snapshot(
        &self,
        _time_start: DateTime<Utc>,
        _time_end: DateTime<Utc>,
    ) -> Option<TimeSnapshot> {
        None
    }
}

fn dummy_data() {
    // Some(TimeSnapshot {
    //     time_start: Utc::now(),
    //     time_end: Utc::now() + Duration::from_millis(1000),
    //     devices: vec![DeviceData {
    //         name: "Paval".to_string(),
    //         mac_address: "1:1:1:1:1".to_string(),
    //
    //         lifetime: vec![
    //             DeviceLifetimeStep {
    //                 distance: 1.0,
    //                 time_start: Utc::now(),
    //                 time_end: Utc::now() + Duration::from_millis(300),
    //             },
    //             DeviceLifetimeStep {
    //                 distance: 1.2,
    //                 time_start: Utc::now() + Duration::from_millis(500),
    //                 time_end: Utc::now() + Duration::from_millis(800),
    //             },
    //             DeviceLifetimeStep {
    //                 distance: 1.0,
    //                 time_start: Utc::now() + Duration::from_millis(900),
    //                 time_end: Utc::now() + Duration::from_millis(1000),
    //             },
    //         ],
    //     }],
    // })
}
