mod config;
mod snapshot;

pub use config::{PredictionConfig, UpdateConfig};
pub use snapshot::StateSnapshot;

use std::time::SystemTime;

use nalgebra::{
    Cholesky, Const, DMatrix, DMatrixView, DVector, DVectorView, Dyn, OMatrix,
    SMatrix, SMatrixView,
};
use serde::{Deserialize, Serialize};

use super::{
    estimate_merwe_sigmas, sigma_order, sigma_points, FnDistance, FnMean,
    FnMeasurement, FnState, MathError, SigmaWeights,
};

pub fn identity_transform<const N: usize>() -> OMatrix<f32, Dyn, Const<N>> {
    OMatrix::<f32, Dyn, Const<N>>::identity(N)
}

pub fn transform_dyn<T, F, const N: usize, const S: usize>(
    sigmas: &SMatrix<f32, S, N>,
    weights: &SigmaWeights<S>,
    noise_opt: Option<SMatrixView<f32, N, N>>,
    dyn_transform: DMatrixView<f32>,
    residual: &T,
    mean: &F,
) -> (SMatrix<f32, 1, N>, OMatrix<f32, Dyn, Dyn>)
where
    T: FnDistance<N>,
    F: FnMean<N, S>,
{
    assert!(dyn_transform.ncols() == N);

    let state = mean(weights.mean.as_view(), sigmas.as_view());

    let mut covariance =
        DMatrix::zeros(dyn_transform.nrows(), dyn_transform.nrows());

    if let Some(raw_noise) = noise_opt {
        let noise = {
            let diag = dyn_transform * raw_noise.diagonal();
            DMatrix::from_diagonal(&diag)
        };
        //assert noise is diagonal matrix
        covariance += noise;
    }

    let apply_dyn_transform = move |state: SMatrixView<f32, 1, N>| {
        (dyn_transform * state.transpose()).transpose()
    };

    for i in 0..S {
        let row = sigmas.row(i).clone_owned();

        let y = {
            let raw_y = residual(row.as_view(), state.as_view());
            apply_dyn_transform(raw_y.as_view())
        };

        let mut outer = y.transpose() * y;
        outer.scale_mut(weights.covariance[i]);
        covariance += outer;
    }

    (state, covariance)
}

pub fn transform<T, F, const N: usize, const S: usize>(
    sigmas: &SMatrix<f32, S, N>,
    weights: &SigmaWeights<S>,
    noise_opt: Option<SMatrixView<f32, N, N>>,
    residual: &T,
    mean: &F,
) -> (SMatrix<f32, 1, N>, SMatrix<f32, N, N>)
where
    T: FnDistance<N>,
    F: FnMean<N, S>,
{
    let state = mean(weights.mean.as_view(), sigmas.as_view());

    let mut covariance = SMatrix::<f32, N, N>::zeros();

    for i in 0..S {
        let row = sigmas.row(i).clone_owned();

        let y = residual(row.as_view(), state.as_view());

        let mut outer = y.transpose() * y;
        outer.scale_mut(weights.covariance[i]);
        covariance += outer;
    }

    if let Some(noise) = noise_opt {
        //assert noise is diagonal matrix
        covariance += noise;
    }

    (state, covariance)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter<const N: usize, const K: usize, const S: usize> {
    /// X
    pub state: SMatrix<f32, 1, N>,
    prior_state: Option<SMatrix<f32, 1, N>>,
    /// sigmas_f
    pub state_sigmas: Option<SMatrix<f32, S, N>>,
    /// Q
    pub state_noise: SMatrix<f32, N, N>,

    /// sigmas_h
    /// the only one reason why this field is not optional
    /// is memory
    pub meas_sigmas: SMatrix<f32, S, K>,
    /// R
    pub meas_noise: SMatrix<f32, K, K>,

    /// S
    pub meas_covariance: Option<DMatrix<f32>>,
    /// SI
    pub inv_meas_covariance: Option<DMatrix<f32>>,
    pub meas_residual: Option<DVector<f32>>,

    /// P
    pub covariance: SMatrix<f32, N, N>,
    prior_covariance: Option<SMatrix<f32, N, N>>,

    /// K
    pub gain: Option<OMatrix<f32, Const<N>, Dyn>>,
    pub weights: SigmaWeights<S>,
}

impl<const N: usize, const K: usize, const S: usize> Default
    for Filter<N, K, S>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, const K: usize, const S: usize> Filter<N, K, S> {
    pub fn new() -> Self {
        assert_eq!(sigma_order(N), S);

        let weights = sigma_points::estimate_merwe_weights::<N, S>(
            sigma_points::SigmaMetadata {
                alpha: 0.1,
                beta: 2.0,
                kappa: (N as f32 - 3.0),
            },
        );
        Self::with_weights(weights)
    }

    pub fn with_weights(weights: SigmaWeights<S>) -> Self {
        Self {
            state: SMatrix::zeros(),
            prior_state: None,
            state_sigmas: None,
            state_noise: SMatrix::identity(),

            meas_sigmas: SMatrix::zeros(),
            meas_noise: SMatrix::identity(),

            meas_residual: None,
            meas_covariance: None,
            inv_meas_covariance: None,

            covariance: SMatrix::identity(),
            prior_covariance: None,

            gain: None,
            weights,
        }
    }

    /// return the state at current time
    pub fn state_snapshot(
        &self,
        timestamp: SystemTime,
    ) -> StateSnapshot<N, K, S> {
        StateSnapshot {
            state: self.state.clone_owned(),
            covariance: self.covariance.clone_owned(),
            meas_noise: self.meas_noise.clone_owned(),
            timestamp,
            _marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.prior_state = None;
        self.prior_covariance = None;

        self.state_sigmas = None;

        {
            self.meas_covariance = None;
            self.inv_meas_covariance = None;
            self.meas_residual = None;
            self.gain = None;
        }
    }

    pub fn reset_with_state(
        &mut self,
        StateSnapshot {
            state,
            covariance,
            meas_noise,
            ..
        }: StateSnapshot<N, K, S>,
    ) {
        self.state = state;
        self.covariance = covariance;
        self.meas_noise = meas_noise;
    }

    pub fn predict<F, M, R>(
        &mut self,
        PredictionConfig {
            next_state,
            mean_transform,
            state_sub,
        }: PredictionConfig<F, M, R>,
    ) -> Result<
        (SMatrixView<'_, f32, 1, N>, SMatrixView<'_, f32, N, N>),
        MathError,
    >
    where
        F: FnState<N>,
        M: FnMean<N, S>,
        R: FnDistance<N>,
    {
        self.reset();

        //compute procces sigmas
        let mut sigmas = estimate_merwe_sigmas::<S, N, _>(
            self.state.as_view(),
            self.covariance.as_view(),
            self.weights.metadata,
            &state_sub,
        )?;

        for mut row in sigmas.row_iter_mut() {
            let row_copy = row.clone_owned();
            row.copy_from(&next_state(row_copy.as_view()));
        }

        // pass sigmas through the unscented transform to compute prior
        let (prior_state, prior_covariance) = transform(
            &sigmas,
            &self.weights,
            Some(self.state_noise.as_view()),
            &state_sub,
            &mean_transform,
        );

        // update sigma points to reflect the new variance of the points
        self.state_sigmas = Some(estimate_merwe_sigmas(
            prior_state.as_view(),
            prior_covariance.as_view(),
            self.weights.metadata,
            &state_sub,
        )?);

        self.prior_state = Some(prior_state);
        self.prior_covariance = Some(prior_covariance);

        Ok((
            self.prior_state.as_ref().unwrap().as_view(),
            self.prior_covariance.as_ref().unwrap().as_view(),
        ))
    }

    pub fn update<M, MT, MR, SR>(
        &mut self,
        meas: &SMatrix<f32, 1, K>,
        dyn_transform: Option<OMatrix<f32, Dyn, Const<K>>>,
        UpdateConfig {
            state_to_meas,
            mean_transform,
            meas_sub,
            state_sub,
        }: UpdateConfig<M, MT, MR, SR>,
    ) -> Result<(), MathError>
    where
        M: FnMeasurement<N, K>,
        MT: FnMean<K, S>,
        MR: FnDistance<K>,
        SR: FnDistance<N>,
    {
        let update_size =
            dyn_transform.as_ref().map(|m| m.nrows()).unwrap_or(K);
        // pass prior sigmas through h(x) to get measurement sigmas
        // the shape of sigmas_h will vary if the shape of z varies, so
        // recreate each time

        let state_sigmas = self
            .state_sigmas
            .as_ref()
            .expect("Update method should be invoked after predict");

        let (prior_state, prior_covariance) = self
            .prior_state
            .as_ref()
            .zip(self.prior_covariance.as_ref())
            .expect("Prior estimations are not provided");

        tracing::debug!("prior state=\n{prior_state}");

        state_sigmas
            .row_iter()
            .zip(self.meas_sigmas.row_iter_mut())
            .for_each(|(f_row, mut h_row)| {
                let row_copy = f_row.clone_owned();
                h_row.copy_from(&state_to_meas(row_copy.as_view()));
            });

        let transform = dyn_transform.unwrap_or(identity_transform());
        let transform_view = transform.as_view();

        // mean and covariance of prediction passed through unscented transform
        let (meas_mean, meas_covariance) = transform_dyn(
            &self.meas_sigmas,
            &self.weights,
            Some(self.meas_noise.as_view()),
            transform_view,
            &meas_sub,
            &mean_transform,
        );

        tracing::debug!("meas mean=\n{meas_mean}");

        let Some(inv_meas_covarinace) =
            meas_covariance.clone_owned().try_inverse()
        else {
            return Err(MathError::NoInverseMatrix);
        };

        // compute cross variance of the state and the measurements
        let cross_variance = {
            // let mut pxz = SMatrix::<f32, N, K>::zeros();
            let mut pxz = OMatrix::<f32, Const<N>, Dyn>::zeros(update_size);
            state_sigmas
                .row_iter()
                .zip(self.meas_sigmas.row_iter())
                .zip(self.weights.covariance.iter())
                .for_each(|((f_row, h_row), weight)| {
                    let dx = state_sub(
                        f_row.clone_owned().as_view(),
                        prior_state.as_view(),
                    );

                    let dz = {
                        let raw_dz = meas_sub(
                            h_row.clone_owned().as_view(),
                            meas_mean.as_view(),
                        );
                        (transform_view * raw_dz.transpose()).transpose()
                    };

                    pxz += *weight * (dx.transpose() * dz);
                });
            pxz
        };
        let gain = cross_variance * inv_meas_covarinace.clone();
        // cross_variance.mul_to(&inv_meas_covarinace, &mut self.gain);

        let y = {
            let raw_y = meas_sub(meas.as_view(), meas_mean.as_view());
            transform_view * raw_y.transpose()
        };

        //todo: consider to add custom method add
        let state_diff = (gain.clone() * y.clone()).transpose();

        tracing::debug!("state diff=\n{state_diff}");
        let covariance_diff =
            gain.clone() * (meas_covariance.clone() * gain.transpose());

        let covariance = *prior_covariance - covariance_diff;

        if Cholesky::new(covariance).is_none() {
            return Err(MathError::NotPositiveCovariance);
        }

        self.state = prior_state + state_diff;
        self.covariance = *prior_covariance - covariance_diff;

        {
            self.gain = Some(gain);
            self.meas_covariance = Some(meas_covariance);
            self.inv_meas_covariance = Some(inv_meas_covarinace);
            self.meas_residual = Some(y);
        }

        Ok(())
    }

    /// Calculate the mahalanobis distance of update
    pub fn mahalanobis(&self) -> Option<f32> {
        let y = self.meas_residual.as_ref()?;
        let inv_meas_covariance = self.inv_meas_covariance.as_ref()?;
        Some(mahalanobis(y.as_view(), inv_meas_covariance.as_view()))
    }
}

/// Mahalanobis distance of measurement. E.g. 3 means measurement
/// was 3 standard deviations away from the predicted value.
pub fn mahalanobis(
    meas_residual: DVectorView<f32>,
    inv_meas_covariance: DMatrixView<f32>,
) -> f32 {
    let dot = meas_residual.transpose() * inv_meas_covariance;

    let dist2 = (dot * meas_residual)[0].max(0.0);

    dist2.sqrt()
}
