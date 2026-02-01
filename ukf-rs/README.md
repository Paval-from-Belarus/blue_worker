# ukf-rs

A Rust implementation of the Unscented Kalman Filter (UKF) for state estimation in nonlinear systems.

## Overview

This crate implements the basic functionality of the Unscented Kalman Filter, a powerful algorithm for estimating the state of nonlinear dynamic systems.
The implementation is inspired by the [filterpy](https://github.com/rlabbe/filterpy) Python library.

## Features

- **Unscented Kalman Filter**: Core UKF implementation for nonlinear state estimation
- **Merwe Sigma Points**: Uses the scaled unscented transform with Merwe's sigma point generation
- **State Snapshots**: Checkpoint and restore filter state at any point in time
- **Serialization Support**: Built-in support for serializing and deserializing filter states via `serde`

## Current Limitations

- **std Required**: Currently only compatible with `std` environments (uses `std::time::SystemTime` for checkpoints)
- **Sigma Points**: Only Merwe (scaled) sigma points are implemented; other sigma point methods (Julier, simplex) are not yet available

## Implementation Details

The implementation follows the standard UKF algorithm:
1. Generate sigma points around the current state estimate
2. Propagate sigma points through the nonlinear process model
3. Compute predicted mean and covariance
4. Propagate sigma points through the measurement model
5. Update state estimate based on measurements

## Upcoming Changes

The following features are planned for future releases:

- [ ] **no_std Support**: Make the crate compatible with `no_std` environments for embedded systems
- [ ] **Particle Filter**: Implement Monte Carlo-based particle filtering for highly nonlinear systems

## Inspiration

This implementation is inspired by the filterpy Python library, providing similar functionality in Rust with strong type safety and performance benefits.

## License

MIT License - see the [LICENSE](LICENSE) file for details.
