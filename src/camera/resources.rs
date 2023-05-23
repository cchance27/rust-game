use bevy::prelude::*;

#[derive(Resource)]
pub struct CameraSettings {
    pub rotation: f32,
    pub pixels_per_line: f32,
    pub wheel_sensitivity: f32,
}
