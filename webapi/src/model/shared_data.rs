use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use blue_types::Scan;
use chrono::{DateTime, Utc};

use super::{DeviceData, DeviceLifetimeStep, StateError, TimeLimits, TimeSnapshot};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeviceSharedState {
    devices_map: HashMap<String, DeviceData>,
    world_limits: Option<TimeLimits>,

    #[serde(skip, default)]
    config_path: String,
}

impl DeviceSharedState {
    fn with_config_name(name: &'static str) -> Self {
        Self {
            devices_map: HashMap::new(),
            world_limits: None,
            config_path: name.to_string(),
        }
    }

    pub async fn load_from_config(path: &'static str) -> Self {
        let Ok(raw_data) = tokio::fs::read_to_string(path).await else {
            return Self::with_config_name(path);
        };

        match serde_json::from_str::<Self>(&raw_data) {
            Ok(mut state) => {
                state.config_path = path.to_string();

                state
            }
            Err(cause) => {
                log::warn!("Failed to load shared data: {cause}");
                Self::with_config_name(path)
            }
        }
    }

    pub async fn put_devices(&mut self, scan: Scan) -> Result<(), StateError> {
        log::debug!("Incoming scan: {:?}", scan);

        let Scan { duration, devices } = scan;

        let scan_start_time = SystemTime::now() - Duration::from_millis(duration);
        let scan_end_time = SystemTime::now();

        for device in devices.into_iter() {
            let mac_address = device.address.to_string();

            self.devices_map
                .entry(mac_address.clone())
                .and_modify(|data| {
                    if let Some(ref name) = device.name
                        && data.name.is_empty()
                    {
                        data.name = name.clone();
                        //otherwise skip new name
                    }

                    let scan_start_ms = scan_start_time
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    let scan_end_ms = scan_end_time
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;

                    let maybe_overlapping = data
                        .lifetime
                        .iter_mut()
                        .find(|step| step.time_start_millis() > scan_start_ms)
                        .cloned();

                    let distance = rssi_to_distance(device.rssi as f32);

                    if let Some(mut step) = maybe_overlapping {
                        assert!(step.time_end_millis() < scan_end_ms);
                        step.time_end = scan_end_time.into();

                        let avg_distance = (step.distance + distance) / 2.0;
                        step.distance = avg_distance;
                    } else {
                        data.lifetime.push(DeviceLifetimeStep {
                            distance,
                            time_start: scan_start_time.into(),
                            time_end: scan_end_time.into(),
                        });
                    }

                    assert!(data.lifetime.is_sorted_by_key(|e| e.time_start));
                })
                .or_insert(DeviceData {
                    mac_address,
                    name: device.name.unwrap_or("".to_string()),
                    lifetime: vec![],
                });
        }

        if let Some(ref mut limits) = self.world_limits {
            limits.epoch_end = scan_end_time.into();
        } else {
            self.world_limits = Some(TimeLimits {
                epoch_start: scan_start_time.into(),
                epoch_end: scan_end_time.into(),
            });
        }

        let Ok(raw_data) = serde_json::to_vec(self) else {
            return Ok(());
        };

        let _ = tokio::fs::write(&self.config_path, raw_data).await;

        Ok(())
    }

    pub async fn take_limits(&self) -> Option<TimeLimits> {
        self.world_limits.clone()
    }

    pub async fn take_snapshot(
        &self,
        time_start: DateTime<Utc>,
        time_end: DateTime<Utc>,
    ) -> Option<TimeSnapshot> {
        let devices = self
            .devices_map
            .values()
            .cloned()
            .collect::<Vec<DeviceData>>();

        Some(TimeSnapshot {
            time_start,
            time_end,
            devices,
        })
    }
}

#[allow(unused)]
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

fn rssi_to_distance(rssi: f32) -> f32 {
    const A: f32 = -50.0; // RSSI value at 1 meter; adjust based on testing.
    const N: f32 = 2.0; // Path loss exponent; adjust depending on the environment.

    if rssi >= A {
        return 1.0; // If RSSI is stronger than A, assume distance is less than 1 meter
    }

    10.0f32.powf((A - rssi) / (10.0 * N))
}
