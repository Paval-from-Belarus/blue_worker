use nalgebra::{Const, Dyn, OMatrix, SMatrix, SMatrixView};
use rand::Rng;

use ukf_rs::filter::{KallmanFilter, PredictionConfig, UpdateConfig};
use ukf_rs::ops::{linear_mean, linear_residual};
use ukf_rs::{estimate_merwe_weights, sigma_order, SigmaMetadata};

#[test]
pub fn test_linear_1d() {
    const S: usize = sigma_order(2);

    let dt = 0.1;
    let weights = estimate_merwe_weights::<2, S>(SigmaMetadata {
        alpha: 0.1,
        beta: 2.0,
        kappa: -1.0,
    });
    let mut filter = KallmanFilter::<2, 1, S>::with_weights(weights);

    filter.state.copy_from_slice(&[1.0, 2.0]);
    filter.covariance.copy_from_slice(&[1.0, 1.1, 1.1, 3.0]);

    filter.state_noise.copy_from_slice(&[0.0, 0.0, 0.0, 0.001]);
    filter.meas_noise.scale_mut(0.05);

    let mut is_first = true;

    let mut gains = Vec::<_>::new();
    let mut states = Vec::<_>::new();

    for _ in 0..20 {
        let z_value = if is_first {
            2.0
        } else {
            rand::rng().random::<f32>()
        };

        let z = SMatrix::<f32, 1, 1>::from_fn(|_, _| z_value);
        filter
            .predict(PredictionConfig {
                state_sub: linear_residual,
                mean_transform: linear_mean,
                next_state: move |state: SMatrixView<f32, 1, 2>| {
                    let f =
                        SMatrix::<f32, 2, 2>::from_vec(vec![1.0, 0.0, dt, 1.0]);
                    let next_state = f * state.transpose();
                    next_state.transpose()
                },
            })
            .unwrap();

        filter
            .update(
                &z,
                None,
                UpdateConfig {
                    meas_sub: linear_residual,
                    state_sub: linear_residual,
                    mean_transform: linear_mean,
                    state_to_meas: |state: SMatrixView<f32, 1, 2>| {
                        let x: f32 = state[0];
                        SMatrix::<f32, 1, 1>::from_fn(|_, _| x)
                    },
                },
            )
            .unwrap();
        if !is_first {
            gains.push(filter.gain.clone().unwrap().transpose());
            states.push(filter.state);
        } else {
            is_first = false
        }
    }

    fn cmp_matrix(matrix: &OMatrix<f32, Dyn, Const<2>>, vec: Vec<f32>) -> bool {
        let l = SMatrix::<f32, 1, 2>::from_vec(vec);
        matrix.relative_eq(&l, 0.001, 1.0)
    }

    let mut gain_iter = gains.iter();
    assert!(cmp_matrix(gain_iter.next().unwrap(), vec![0.596, 1.641]));
    assert!(cmp_matrix(gain_iter.next().unwrap(), vec![0.536, 1.838]));
    assert!(cmp_matrix(gain_iter.next().unwrap(), vec![0.515, 1.664]));
    assert!(cmp_matrix(gain_iter.next().unwrap(), vec![0.487, 1.379]));
    assert!(cmp_matrix(gain_iter.next().unwrap(), vec![0.453, 1.113]));
    assert!(cmp_matrix(gain_iter.next().unwrap(), vec![0.418, 0.899]));
}
