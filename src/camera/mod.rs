use bevy::prelude::*;

pub mod components;
mod resources;
mod systems;

use resources::*;
use systems::*;
//use components::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings {
            rotation: 0.02,
            pixels_per_line: 52.0,
            wheel_sensitivity: 0.2,
        })
        .add_startup_system(spawn_camera)
        .add_system(pan_orbit_camera);
    }
}
