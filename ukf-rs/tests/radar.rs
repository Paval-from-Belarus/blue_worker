use nalgebra::{SMatrix, SMatrixView, Vector2, Vector4};

use ukf_rs::filter::{Filter, PredictionConfig, UpdateConfig};
use ukf_rs::ops::{linear_mean, linear_residual};
use ukf_rs::{estimate_merwe_weights, noise, sigma_order, SigmaMetadata};

use plotters::{
    chart::ChartBuilder,
    evcxr::evcxr_figure_with_saving,
    prelude::PathElement,
    series::LineSeries,
    style::{Color, IntoFont, BLACK, BLUE, RED, WHITE},
};
use rand::Rng;

#[test]
fn test_radar() {
    const S: usize = sigma_order(4);
    const TICK_TIME: f32 = 1.0;

    #[derive(Debug)]
    struct Radar {
        pos: Vector2<f32>,
        range_std: f32,
        elevation_angle_std: f32,
    }

    impl Radar {
        fn reading_of(&self, ac_pos: Vector2<f32>) -> [f32; 2] {
            let diff = ac_pos - self.pos;
            let range = diff.norm();
            let angle = f32::atan2(diff.y, diff.x);
            [range, angle]
        }

        fn noisy_reading(&self, ac_pos: Vector2<f32>) -> [f32; 2] {
            let [mut range, mut angle] = self.reading_of(ac_pos);
            range += rand::rng().random::<f32>() * self.range_std;
            angle += rand::rng().random::<f32>() * self.elevation_angle_std;
            [range, angle]
        }
    }

    #[derive(Debug)]
    struct Airplane {
        pos: Vector2<f32>,
        vel: Vector2<f32>,
        vel_std: f32,
    }

    impl Airplane {
        fn update(&mut self, dt: f32) {
            let mut dx = self.vel * dt;
            dx.add_scalar_mut(rand::rng().random::<f32>() * self.vel_std * dt);
            self.pos += dx;
        }
    }

    let radar = Radar {
        pos: Vector2::new(0.0, 0.0),
        range_std: 5.,
        elevation_angle_std: 0.5f32.to_radians(),
    };

    let mut airplane = Airplane {
        pos: Vector2::new(0.0, 1000.0),
        vel: Vector2::new(100., 0.),
        vel_std: 0.02,
    };

    let weights = estimate_merwe_weights::<4, S>(SigmaMetadata {
        alpha: 0.1,
        beta: 2.0,
        kappa: -1.0,
    });
    let mut filter = Filter::<4, 2, S>::with_weights(weights);
    filter.state.copy_from_slice(&[0., 90., 1100., 0.]);

    filter
        .state_noise
        .view_range_mut(0..2, 0..2)
        .copy_from(&noise::discrete_white::<2, 1, 2>(TICK_TIME, 0.1));

    filter
        .state_noise
        .view_range_mut(2..4, 2..4)
        .copy_from(&noise::discrete_white::<2, 1, 2>(TICK_TIME, 0.1));

    filter.covariance = SMatrix::from_diagonal(&Vector4::new(
        300.0f32.powi(2),
        2.0f32.powi(2),
        150.0f32.powi(2),
        3.0f32.powi(2),
    ));

    eprintln!("Prior P: {}", filter.covariance);
    eprintln!("Prior P: {}", filter.covariance);

    let hf_noise = SMatrix::from_diagonal(&Vector2::new(
        radar.range_std,
        radar.elevation_angle_std,
    ));
    filter.meas_noise = hf_noise * hf_noise;

    let mut tick = 0.0;
    let mut filter_series = Vec::new();
    let mut origin_series = Vec::new();
    while tick <= 360.0 {
        if tick >= 60. {
            airplane.vel.y = 300. / 60.;
        }

        airplane.update(TICK_TIME);
        // eprintln!("AC = {:?}", airplane);
        let z_value = radar.noisy_reading(airplane.pos);

        let z = SMatrix::<f32, 1, 2>::from_row_slice(&z_value);

        filter
            .predict(PredictionConfig {
                state_sub: linear_residual,
                mean_transform: linear_mean,
                next_state: move |state: SMatrixView<f32, 1, 4>| {
                    let f = SMatrix::<f32, 4, 4>::from_row_slice(&[
                        1., TICK_TIME, 0., 0., 0., 1., 0., 0., 0., 0., 1.,
                        TICK_TIME, 0., 0., 0., 1.,
                    ]);
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
                    state_to_meas: move |state: SMatrixView<f32, 1, 4>| {
                        let dx = state[0] - 0.0;
                        let dy = state[2] - 0.0;

                        let slant_range = (dx.powi(2) + dy.powi(2)).sqrt();
                        let elevation_angle = f32::atan2(dy, dx);

                        SMatrix::from_row_slice(&[slant_range, elevation_angle])
                    },
                },
            )
            .unwrap();
        filter_series.push((tick, filter.state[2]));
        origin_series.push((tick, airplane.pos.y));
        tick += TICK_TIME;
    }

    eprintln!("Actual: {}", airplane.pos.y);
    eprintln!("Filter: {}", filter.state[2]);

    eprintln!("State P: {}", filter.covariance);
    eprintln!("State S: {}", filter.meas_covariance.unwrap());

    let _ = evcxr_figure_with_saving("chart.svg", (640, 480), move |root| {
        let min_y = f32::min(
            filter_series
                .iter()
                .map(|(_, y)| *y)
                .reduce(f32::min)
                .unwrap(),
            origin_series
                .iter()
                .map(|(_, y)| *y)
                .reduce(f32::min)
                .unwrap(),
        );
        let max_y = f32::max(
            filter_series
                .iter()
                .map(|(_, y)| *y)
                .reduce(f32::max)
                .unwrap(),
            origin_series
                .iter()
                .map(|(_, y)| *y)
                .reduce(f32::max)
                .unwrap(),
        );
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("y=x^2", ("Arial", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0f32..360f32, min_y..max_y)?;

        chart.configure_mesh().draw()?;

        // let s = (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x));
        chart
            .draw_series(LineSeries::new(filter_series, &RED))
            .unwrap()
            .label("Filter")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .draw_series(LineSeries::new(origin_series, &BLUE))
            .unwrap()
            .label("Origin")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));
        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
        Ok(())
    });
}
