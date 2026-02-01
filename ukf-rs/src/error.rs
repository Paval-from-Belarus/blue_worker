use nalgebra::DMatrix;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MathError {
    #[error("Provided covariance matrix is not positive defined")]
    CholeskyFailed(DMatrix<f32>),
    #[error("Failed to find inverse matrix for meas covariance")]
    NoInverseMatrix,
    #[error("Data type is not supported")]
    NotSupportedData,

    #[error("The covariance is not positively defined")]
    NotPositiveCovariance,
}
