//! Noise matrix generation utilities.
//!
//! This module contains utility functions to generate noise covariance matrices
//! for use in Kalman filtering, specifically for creating discrete white noise
//! process covariance matrices.
//!
//! The primary function [`discrete_white`] generates properly ordered noise matrices
//! based on the state space dimensions and derivative order.
use nalgebra::SMatrix;

/// N - the count of state variable
/// K - the count of derivations per block (i.e. x, x', x''). By another words, block size
/// S - output matrix size (N * K)
pub fn discrete_white<const N: usize, const K: usize, const S: usize>(
    dt: f32,
    base_var: f32,
) -> SMatrix<f32, S, S> {
    assert!(S == N * K);
    let noise_matrix = match N {
        2 => {
            SMatrix::<f32, N, N>::from_column_slice(&[
                //first column
                0.25 * dt.powi(4),
                0.5 * dt.powi(3),
                //second column
                0.5 * dt.powi(3),
                dt.powi(2),
            ])
        }
        3 => {
            SMatrix::<f32, N, N>::from_column_slice(&[
                //first column
                0.25 * dt.powi(4),
                0.5 * dt.powi(3),
                0.5 * dt.powi(2),
                //second column
                0.5 * dt.powi(3),
                dt.powi(2),
                dt,
                //third column
                0.5 * dt.powi(2),
                dt,
                1.0,
            ])
        }
        4 => {
            SMatrix::<f32, N, N>::from_column_slice(&[
                //first column
                dt.powi(6) / 36.0,
                dt.powi(5) / 12.0,
                dt.powi(4) / 6.0,
                dt.powi(3) / 6.0,
                //second column
                dt.powi(5) / 12.0,
                dt.powi(4) / 4.0,
                dt.powi(3) / 2.0,
                dt.powi(2) / 2.0,
                //third column
                dt.powi(4) / 6.0,
                dt.powi(3) / 2.0,
                dt.powi(2),
                dt,
                //fourth column
                dt.powi(3) / 6.0,
                dt.powi(2) / 2.0,
                dt,
                1.0,
            ])
        }
        _ => {
            unreachable!("Invalid dimmensions");
        }
    };
    order_by_derivative::<N, K, S>(noise_matrix) * base_var
}

/// Given a matrix Q, ordered assuming state space
///        `[x y z x' y' z' x'' y'' z''...]`
///
///    return a reordered matrix assuming an ordering of
///       `[ x x' x'' y y' y'' z z' y'']`
///
///    This works for any covariance matrix or state transition function
fn order_by_derivative<const N: usize, const K: usize, const S: usize>(
    raw_matrix: SMatrix<f32, N, N>,
) -> SMatrix<f32, S, S> {
    assert!(S == N * K);

    let mut matrix = SMatrix::<f32, S, S>::zeros();
    for (index, value) in raw_matrix.iter().enumerate() {
        let mut eye = SMatrix::<f32, K, K>::identity();
        eye.scale_mut(*value);

        let first_row = index / N * K;
        let first_column = index % N * K;

        let mut view = matrix.view_range_mut(
            first_row..first_row + K,
            first_column..first_column + K,
        );
        view.copy_from(&eye);
    }
    matrix
}
