use nalgebra::{SMatrix, SMatrixView};

pub fn linear_residual<const N: usize>(
    x: SMatrixView<f32, 1, N>,
    y: SMatrixView<f32, 1, N>,
) -> SMatrix<f32, 1, N> {
    x - y
}

pub fn linear_mean<const N: usize, const S: usize>(
    weights: SMatrixView<f32, 1, S>,
    sigmas: SMatrixView<f32, S, N>,
) -> SMatrix<f32, 1, N> {
    weights * sigmas
}

pub fn identity_state<const N: usize>(
    view: SMatrixView<f32, 1, N>,
) -> SMatrix<f32, 1, N> {
    view.clone_owned()
}

pub fn zeros_measurement<const N: usize, const K: usize>(
    _state: SMatrixView<f32, 1, N>,
) -> SMatrix<f32, 1, K> {
    SMatrix::zeros()
}
