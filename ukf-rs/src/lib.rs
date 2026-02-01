mod error;
pub mod noise;
mod sigma_points;
mod state;
mod unscented;

use nalgebra::{Const, Dyn, OMatrix, SMatrix, SMatrixView};

pub use error::*;
pub use sigma_points::{
    estimate_merwe_sigmas, estimate_merwe_weights, sigma_order, SigmaMetadata,
    SigmaWeights,
};
pub use state::*;

pub use unscented::{
    linear_mean, linear_residual, KallmanFilter, PredictionConfig, UpdateConfig,
};

pub trait FnState<const N: usize>:
    Fn(SMatrixView<'_, f32, 1, N>) -> SMatrix<f32, 1, N>
{
}

impl<T, const N: usize> FnState<N> for T where
    T: Fn(SMatrixView<'_, f32, 1, N>) -> SMatrix<f32, 1, N>
{
}

pub trait FnMeasurement<const N: usize, const K: usize>:
    Fn(SMatrixView<'_, f32, 1, N>) -> SMatrix<f32, 1, K>
{
}

impl<T, const N: usize, const K: usize> FnMeasurement<N, K> for T where
    T: Fn(SMatrixView<'_, f32, 1, N>) -> SMatrix<f32, 1, K>
{
}

pub trait FnDistance<const N: usize>:
    Fn(SMatrixView<'_, f32, 1, N>, SMatrixView<'_, f32, 1, N>) -> SMatrix<f32, 1, N>
{
}

impl<T, const N: usize> FnDistance<N> for T where
    T: Fn(
        SMatrixView<'_, f32, 1, N>,
        SMatrixView<'_, f32, 1, N>,
    ) -> SMatrix<f32, 1, N>
{
}

pub trait FnMean<const N: usize, const S: usize>:
    Fn(SMatrixView<'_, f32, 1, S>, SMatrixView<'_, f32, S, N>) -> SMatrix<f32, 1, N>
{
}

impl<T, const N: usize, const S: usize> FnMean<N, S> for T where
    T: Fn(
        SMatrixView<'_, f32, 1, S>,
        SMatrixView<'_, f32, S, N>,
    ) -> SMatrix<f32, 1, N>
{
}

//specially for dynamically sized measurements
pub trait FnMeanDyn<const N: usize, const S: usize>:
    Fn(
    SMatrixView<'_, f32, 1, S>,
    SMatrixView<'_, f32, S, N>,
) -> OMatrix<f32, Const<1>, Dyn>
{
}

impl<T, const N: usize, const S: usize> FnMeanDyn<N, S> for T where
    T: Fn(
        SMatrixView<'_, f32, 1, S>,
        SMatrixView<'_, f32, S, N>,
    ) -> OMatrix<f32, Const<1>, Dyn>
{
}
