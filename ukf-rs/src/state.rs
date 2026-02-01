use std::{marker::PhantomData, time::SystemTime};

use nalgebra::SMatrix;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterState<const N: usize, const K: usize, const S: usize> {
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
    for FilterState<N, K, S>
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}
