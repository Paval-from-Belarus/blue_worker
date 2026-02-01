use nalgebra::{Cholesky, DMatrix, SMatrix, SMatrixView, SVector};

use crate::{
    sigma_order, sigma_points::SigmaPointsType, FnDistance, MathError,
    SigmaMetadata,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SigmaWeights<const S: usize> {
    pub covariance: SMatrix<f32, 1, S>,
    pub mean: SMatrix<f32, 1, S>,

    /// number of sigma-points for each state
    pub sigma_order: usize,
    /// metadata by which weights were built
    pub metadata: SigmaMetadata,
    /// generation strategy
    pub(crate) points_type: SigmaPointsType,
}

pub fn estimate_merwe_weights<const N: usize, const S: usize>(
    metadata: SigmaMetadata,
) -> SigmaWeights<S> {
    assert!(S == sigma_order(N));

    let SigmaMetadata { alpha, beta, kappa } = metadata;

    let n = N as f32;

    let lambda = (alpha * alpha) * (n + kappa) - n;
    let c = 0.5 / (n + lambda);
    let mut w_m = vec![c; S];
    let mut w_c = vec![c; S];
    w_c[0] = lambda / (n + lambda) + (1.0 - alpha * alpha + beta);
    w_m[0] = lambda / (n + lambda);

    SigmaWeights {
        covariance: SVector::<f32, S>::from_vec(w_c).transpose(),
        mean: SVector::<f32, S>::from_vec(w_m).transpose(),
        sigma_order: S,
        metadata,
        points_type: SigmaPointsType::Merwe,
    }
}

pub fn estimate_merwe_sigmas<const S: usize, const N: usize, D>(
    state: SMatrixView<f32, 1, N>,
    covariance: SMatrixView<f32, N, N>,
    metadata: SigmaMetadata,
    distance_of: &D,
) -> Result<SMatrix<f32, S, N>, MathError>
where
    D: FnDistance<N>,
{
    let SigmaMetadata { alpha, kappa, .. } = metadata;

    let mean_estimations = {
        let state_size = N as f32;
        let lambda = alpha.powi(2) * (state_size + kappa) - state_size;
        let matrix = covariance.scale(lambda + state_size);
        let matrix = matrix.symmetric_part() + SMatrix::identity() * 0.0001;
        let Some(cholesky) = Cholesky::new(matrix) else {
            let dmatrix = DMatrix::from_iterator(
                matrix.nrows(),
                matrix.ncols(),
                matrix.iter().copied(),
            );

            return Err(MathError::CholeskyFailed(dmatrix));
        };
        cholesky.l()
    };

    let mut sigmas = SMatrix::<f32, S, N>::zeros();
    sigmas.row_mut(0).copy_from(&state);

    for i in 0..state.len() {
        let mean_estimation = mean_estimations.column(i).transpose();

        sigmas
            .row_mut(i + 1)
            .copy_from(&distance_of(state, (-1.0 * mean_estimation).as_view()));

        sigmas
            .row_mut(N + i + 1)
            .copy_from(&distance_of(state, (1.0 * mean_estimation).as_view()));
    }

    Ok(sigmas)
}
