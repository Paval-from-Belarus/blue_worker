#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Pose2D {
    pub x: f32,
    pub y: f32,
    pub heading: Option<f32>,
}
