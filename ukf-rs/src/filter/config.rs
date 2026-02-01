pub struct PredictionConfig<F, M, R> {
    pub next_state: F,
    pub mean_transform: M,
    pub state_sub: R,
}

pub struct UpdateConfig<M, MT, MR, SR> {
    pub state_to_meas: M,
    /// transformation for sigma points
    pub mean_transform: MT,
    pub meas_sub: MR,
    pub state_sub: SR,
}
