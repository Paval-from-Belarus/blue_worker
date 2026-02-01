#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SigmaMetadata {
    pub alpha: f32,
    pub beta: f32,
    pub kappa: f32,
}

impl Default for SigmaMetadata {
    fn default() -> Self {
        Self {
            alpha: 0.1,
            beta: 2.0,
            kappa: -1.0,
        }
    }
}
