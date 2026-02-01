mod metadata;
mod weights;

pub use metadata::SigmaMetadata;
pub use weights::{
    estimate_merwe_sigmas, estimate_merwe_weights, SigmaWeights,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigmaPointsType {
    #[serde(other)]
    Merwe,
}

pub const fn sigma_order(state_size: usize) -> usize {
    2 * state_size + 1
}
