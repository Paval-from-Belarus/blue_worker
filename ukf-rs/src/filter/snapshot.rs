use std::{marker::PhantomData, time::SystemTime};

use nalgebra::SMatrix;
use serde::{Deserialize, Serialize};

/// A snapshot representing the state of an Unscented Kalman Filter (UKF) at a specific moment in time.
///
/// This struct captures the complete state of the UKF filter, including the estimated state vector,
/// covariance matrix, and measurement noise. It can be used for checkpointing, state persistence,
/// or analyzing the filter's evolution over time.
///
/// # Type Parameters
///
/// * `N` - The dimension of the state vector
/// * `K` - The dimension of the measurement vector
/// * `S` - The number of sigma points used in the UKF (typically 2*N + 1)
///
/// # Fields
///
/// * `state` - The estimated state vector (X) at this moment
/// * `covariance` - The state covariance matrix (P) representing uncertainty in the state estimate
/// * `meas_noise` - The measurement noise covariance matrix (R)
/// * `timestamp` - The time at which this snapshot was taken
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSnapshot<const N: usize, const K: usize, const S: usize> {
    /// X
    pub state: SMatrix<f32, 1, N>,
    /// P
    pub covariance: SMatrix<f32, N, N>,
    /// R
    pub meas_noise: SMatrix<f32, K, K>,

    pub timestamp: SystemTime,
    pub(crate) _marker: PhantomData<[f32; S]>,
}

impl<const N: usize, const K: usize, const S: usize> PartialOrd
    for StateSnapshot<N, K, S>
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}
