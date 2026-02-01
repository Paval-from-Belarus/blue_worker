use ukf_rs::{estimate_merwe_weights, sigma_order, SigmaMetadata};

#[test]
pub fn test_scaled_weight() {
    for n in 1..5 {
        let mut alpha = 0.99;
        let step = (1.01 - 0.99) / 100.0;
        while alpha < 1.01 {
            let metadata = SigmaMetadata {
                alpha,
                beta: 0.0,
                kappa: 3.0 - n as f32,
            };
            match n {
                1 => {
                    const S: usize = sigma_order(1);
                    let weights = estimate_merwe_weights::<1, S>(metadata);
                    assert!(
                        (weights.covariance.column_sum()[0] - 1.0).abs() < 0.1
                    );
                    assert!((weights.mean.column_sum()[0] - 1.0).abs() < 0.1);
                }
                2 => {
                    const S: usize = sigma_order(2);
                    let weights = estimate_merwe_weights::<2, S>(metadata);
                    assert!(
                        (weights.covariance.column_sum()[0] - 1.0).abs() < 0.1
                    );
                    assert!((weights.mean.column_sum()[0] - 1.0).abs() < 0.1);
                }
                3 => {
                    const S: usize = sigma_order(3);
                    let weights = estimate_merwe_weights::<3, S>(metadata);
                    assert!(
                        (weights.covariance.column_sum()[0] - 1.0).abs() < 0.1
                    );
                    assert!((weights.mean.column_sum()[0] - 1.0).abs() < 0.1);
                }
                4 => {
                    const S: usize = sigma_order(4);
                    let weights = estimate_merwe_weights::<4, S>(metadata);
                    assert!(
                        (weights.covariance.column_sum()[0] - 1.0).abs() < 0.1
                    );
                    assert!((weights.mean.column_sum()[0] - 1.0).abs() < 0.1);
                }
                _ => panic!(),
            };
            alpha += step;
        }
    }
}
