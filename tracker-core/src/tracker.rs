use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime},
};

use nalgebra::{DMatrix, DVector, Point2, SMatrix, SMatrixView, SVector};
use ukf_rs::noise;

use crate::model::Pose2D;

const STATE_SIZE: usize = 6;
const MEAS_SIZE: usize = 2;
const SIGMA_ORDER: usize = ukf_rs::sigma_order(STATE_SIZE);

const STATE_X: usize = 0;
const STATE_Y: usize = 3;

const BASE_COV_NOISE: f32 = 10.0;

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("No previous update time available")]
    NoPreviousUpdate,
    #[error("Least squares error: {0}")]
    LstsqError(&'static str),
    #[error("UKF math error: {0}")]
    UkfError(#[from] ukf_rs::MathError),
}

///Example
///```
///use tracker::{Tracker, RssiEvent, TrackerConfig};
///
///let config = toml::from_str(r#"
///     max_age = 5
///     lstsq_epsilon = 1e-6
///     min_devices = 3
///     predict_epoch = 100
///     state_coords_noise = 1.0
///     meas_distance_noise = 1.0
///     [devices]
///     [devices.device_1]
///     device_coords = [0.0, 0.0]
///     tx_power = -59.0
///"#).unwrap();
///
///let mut tracker = Tracker::new(config);
///
///let event = RssiEvent {
///    rssi: -65,
///    time: std::time::Instant::now(),
/// };
///
/// tracker.push_rssi("device_1".to_string(), event).unwrap();
///
///match tracker.estimated_pose() {
///    Some(pose) => println!("Estimated pose: {:?}", pose),
///    None => println!("No valid pose estimated"),
///}
///```
pub struct Tracker {
    pub events: HashMap<DeviceId, Vec<RssiEvent>>,
    pub config: TrackerConfig,

    pub kf: ukf_rs::filter::Filter<STATE_SIZE, MEAS_SIZE, SIGMA_ORDER>,
    pub last_update: Option<SystemTime>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TrackerConfig {
    /// Maximum age of RSSI events to consider
    #[serde(
        serialize_with = "serialize_millis",
        deserialize_with = "deserialize_millis"
    )]
    pub max_age: Duration,

    /// The epsilon value for the least squares solver
    pub lstsq_epsilon: f64,

    /// The minimum number of devices required for trilateration
    pub min_devices: usize,

    /// Duration of each prediction epoch
    /// This value is used to compute the process noise
    /// Therefore it should match the rate of incoming measurements
    /// If measurements are irregular, consider using the average rate
    /// or a fixed value based on expected update frequency
    #[serde(
        serialize_with = "serialize_millis",
        deserialize_with = "deserialize_millis"
    )]
    pub predict_epoch: Duration,

    pub state_coords_noise: f32,
    pub meas_distance_noise: f32,

    pub devices: HashMap<DeviceId, DeviceConfig>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DeviceConfig {
    pub device_coords: Point2<f64>,
    pub tx_power: f64,
}

pub type DeviceId = String;

#[derive(Debug, Clone)]
pub struct RssiEvent {
    pub rssi: isize,
    pub time: Instant,
}

impl Tracker {
    pub fn new(config: TrackerConfig) -> Self {
        use ukf_rs::{SigmaMetadata, estimate_merwe_weights};
        let weights =
            estimate_merwe_weights::<STATE_SIZE, SIGMA_ORDER>(SigmaMetadata {
                alpha: 0.1,
                beta: 2.0,
                kappa: 0.0,
                // kappa: STATE_SIZE as f32 - 3.0,
            });

        let kf = ukf_rs::filter::Filter::with_weights(weights);

        Self {
            events: HashMap::new(),
            config,
            kf,
            last_update: None,
        }
    }

    pub fn push_rssi(
        &mut self,
        device_id: DeviceId,
        event: RssiEvent,
    ) -> Result<(), UpdateError> {
        self.events.entry(device_id).or_default().push(event);

        self.events.retain(|_, events| {
            events.retain(|e| e.time.elapsed() < self.config.max_age);
            !events.is_empty()
        });

        let (coords, distances) = self
            .events
            .iter()
            .filter_map(|(device_id, events)| {
                let avg_rssi = events.iter().map(|e| e.rssi).sum::<isize>()
                    / events.len() as isize;

                let config = self.config.devices.get(device_id)?;

                let distance = estimate_distance(avg_rssi, config.tx_power)?;

                (config.device_coords, distance).into()
            })
            .collect::<(Vec<_>, Vec<_>)>();

        if coords.len() < self.config.min_devices {
            tracing::debug!("Not enough data points for trilateration");
            return Ok(());
        }

        let new_pose =
            estimate_pose(&coords, &distances, self.config.lstsq_epsilon)?;

        self.update(new_pose)?;

        Ok(())
    }

    pub fn estimated_pose(&self) -> Option<Pose2D> {
        let elapsed =
            SystemTime::now().duration_since(self.last_update?).ok()?;

        if elapsed >= self.config.max_age {
            return None;
        }

        let x = self.kf.state[STATE_X];
        let y = self.kf.state[STATE_Y];

        Some(Pose2D {
            x,
            y,
            heading: None,
        })
    }

    /// Reset the tracker with a given pose
    pub fn reset(&mut self, pose: Pose2D) {
        let config = &self.config;

        self.kf.state.copy_from(&pose_to_state(&pose));

        self.kf
            .state_noise
            .copy_from(&state_noise_from_config(config));

        self.kf
            .meas_noise
            .copy_from(&meas_noise_from_config(config));

        self.kf.covariance.scale_mut(BASE_COV_NOISE);
    }

    fn update(&mut self, pose: Pose2D) -> Result<(), UpdateError> {
        let dt = self.last_update.take().and_then(|t| {
            t.duration_since(SystemTime::now())
                .inspect_err(|_| {
                    tracing::debug!("SystemTime went backwards");
                })
                .ok()
                .map(|d| d.as_secs_f32())
        });

        let Some(dt) = dt else {
            return Err(UpdateError::NoPreviousUpdate);
        };

        #[rustfmt::skip]
        let next_state =
            move |prev_state: SMatrixView<'_, f32, 1, STATE_SIZE>| {
                let transform = SMatrix::<f32, STATE_SIZE, STATE_SIZE>::from_row_slice(
               &[
                   1.0, dt, dt * dt / 2.0, 0.0, 0.0, 0.0, //x' = x + v_x * dt + a_x * dt^2 / 2
                   0.0, 1.0, dt, 0.0, 0.0, 0.0, // v_x' = v_x + a_x * dt
                   0.0, 0.0, 1.0, 0.0, 0.0, 0.0, // a_x' = const

                   0.0, 0.0, 0.0, 1.0, dt, dt * dt / 2.0, //y = y + v_y * dt + a_y * dt^2 / 2
                   0.0, 0.0, 0.0, 0.0, 1.0, dt,  //v_y = v_y + a_y * dt
                   0.0, 0.0, 0.0, 0.0, 0.0, 1.0,  // a_y = const
               ]);

               //  uncomment to use constant velocity model (less accurate for dynamic objects)
               //  let transform = SMatrix::<f32, STATE_SIZE, STATE_SIZE>::from_row_slice(
               // &[
               //     1.0, dt, 0.0, 0.0, //x' = x + v_x * dt
               //     0.0, 1.0, 0.0, 0.0, // v_x' = v_x
               //
               //     0.0, 0.0, 1.0, dt, //y = y + v_y * dt
               //     0.0, 0.0, 0.0, 1.0,  //v_y = v_y
               // ]);

                let next_state = transform * prev_state.transpose();

                next_state.transpose()
            };

        self.kf.predict(ukf_rs::filter::PredictionConfig {
            next_state,
            mean_transform: ukf_rs::ops::linear_mean,
            state_sub: ukf_rs::ops::linear_residual,
        })?;

        let z = SMatrix::<f32, 1, MEAS_SIZE>::from_row_slice(&[pose.x, pose.y]);

        let state_to_meas = |state: SMatrixView<f32, 1, STATE_SIZE>| -> SMatrix<f32, 1, MEAS_SIZE> {
        SMatrix::from_column_slice(&[
            state[STATE_X],
            state[STATE_Y],
        ])
    };

        self.kf.update(
            &z,
            None,
            ukf_rs::filter::UpdateConfig {
                state_to_meas,
                mean_transform: ukf_rs::ops::linear_mean,
                meas_sub: ukf_rs::ops::linear_residual,
                state_sub: ukf_rs::ops::linear_residual,
            },
        )?;

        tracing::debug!(target = "tracker", "Update state = {}", self.kf.state,);

        self.last_update = Some(SystemTime::now());

        Ok(())
    }
}

fn state_noise_from_config(
    config: &TrackerConfig,
) -> SMatrix<f32, STATE_SIZE, STATE_SIZE> {
    let mut state_noise = SMatrix::<f32, STATE_SIZE, STATE_SIZE>::zeros();
    const VAR_SIZE: usize = 3;

    state_noise
        .view_range_mut(0..VAR_SIZE, 0..VAR_SIZE)
        .copy_from(&noise::discrete_white::<VAR_SIZE, 1, VAR_SIZE>(
            config.predict_epoch.as_secs_f32(),
            config.state_coords_noise,
        ));

    state_noise
        .view_range_mut(VAR_SIZE..(2 * VAR_SIZE), VAR_SIZE..(2 * VAR_SIZE))
        .copy_from(&noise::discrete_white::<VAR_SIZE, 1, VAR_SIZE>(
            config.predict_epoch.as_secs_f32(),
            config.state_coords_noise,
        ));

    state_noise
}

fn meas_noise_from_config(
    config: &TrackerConfig,
) -> SMatrix<f32, MEAS_SIZE, MEAS_SIZE> {
    //both x and y measurements have the same noise
    let diag = SVector::from_row_slice(&[
        config.meas_distance_noise,
        config.meas_distance_noise,
    ]);

    SMatrix::from_diagonal(&diag)
}

fn pose_to_state(pose: &Pose2D) -> SMatrix<f32, 1, STATE_SIZE> {
    SMatrix::<f32, 1, STATE_SIZE>::from_row_slice(&[
        pose.x, 0.0, 0.0, pose.y, 0.0, 0.0,
    ])
}

pub fn estimate_distance(rssi: isize, tx_power: f64) -> Option<f64> {
    if rssi == 0 {
        return None;
    }

    let r = rssi as f64 / tx_power;

    if r < 1.0 {
        Some(r.powi(10))
    } else {
        Some(0.89976 * r.powf(7.7095) + 0.111)
    }
}

pub fn estimate_pose(
    coords: &[Point2<f64>],
    distances: &[f64],
    lstsq_epsilon: f64,
) -> Result<Pose2D, UpdateError> {
    let ref_idx = 0;
    let num_eqs = coords.len() - 1;

    let mut a_data = Vec::with_capacity(num_eqs * 2);
    let mut b_data = Vec::with_capacity(num_eqs);

    for i in 1..coords.len() {
        let xi = coords[i].x;
        let yi = coords[i].y;
        let di = distances[i];

        let x0 = coords[ref_idx].x;
        let y0 = coords[ref_idx].y;
        let d0 = distances[ref_idx];

        a_data.push(2.0 * (xi - x0));
        a_data.push(2.0 * (yi - y0));

        b_data.push(xi * xi - x0 * x0 + yi * yi - y0 * y0 + d0 * d0 - di * di);
    }

    let a = DMatrix::from_row_slice(num_eqs, 2, &a_data);
    let b = DVector::from_row_slice(&b_data);

    match lstsq::lstsq(&a, &b, lstsq_epsilon) {
        Ok(sol) => {
            tracing::debug!(
                target = "tracker",
                "Lstsq solution: {:?}. Residuals: {:?}",
                sol.solution,
                sol.residuals,
            );

            Ok(Pose2D {
                x: sol.solution[0] as f32,
                y: sol.solution[1] as f32,
                heading: None,
            })
        }
        Err(e) => Err(UpdateError::LstsqError(e)),
    }
}

pub fn deserialize_millis<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::Deserialize;

    let millis = u64::deserialize(deserializer)?;

    Ok(Duration::from_millis(millis))
}

pub fn serialize_millis<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_u64(d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_trilateration() {
        use super::estimate_pose;
        use nalgebra::Point2;

        let devices = vec![
            Point2::new(0.0f64, 0.0),
            Point2::new(10.0, 0.0),
            Point2::new(5.0, 8.66),
        ];

        let target = Point2::new(5.0, 4.0);

        let distances: Vec<f64> = devices
            .iter()
            .map(|d| {
                ((d.x - target.x).powi(2) + (d.y - target.y).powi(2)).sqrt()
            })
            .collect();

        let estimated_pose = estimate_pose(&devices, &distances, 1e-6)
            .expect("Trilateration failed");

        assert!(
            (estimated_pose.x - target.x as f32).abs() < 1e-3,
            "X coordinate mismatch"
        );
        assert!(
            (estimated_pose.y - target.y as f32).abs() < 1e-3,
            "Y coordinate mismatch"
        );
    }
}
