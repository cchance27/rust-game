use super::{components::PlayerCamera, resources::CameraSettings};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use bevy_mod_picking::prelude::*;

pub fn spawn_camera(mut command: Commands) {
    command.spawn(
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 0.0, 0.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection::default()),
            ..default()
        })
        .insert(PlayerCamera)
        .insert(RaycastPickCamera::default())
        .insert(Name::new("PlayerCamera3d"));
}

pub fn camera_movement(
    keyboard: Res<Input<KeyCode>>,
    cam_settings: Res<CameraSettings>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        if keyboard.pressed(KeyCode::Q) {
            transform.rotate_around(Vec3::Y, Quat::from_rotation_y(cam_settings.rotation));
        } else if keyboard.pressed(KeyCode::E) {
            transform.rotate_around(Vec3::Y, Quat::from_rotation_y(-cam_settings.rotation));
        }

        let mut change = 0.0;
        for event in mouse_wheel.iter() {
            // scale the event magnitude per pixel or per line
            let scroll_amount = match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y / cam_settings.pixels_per_line,
            };
            change += scroll_amount;
        }

        if change != 0.0 {
            change *= cam_settings.wheel_sensitivity;
            let change = transform.local_z() * change;
            transform.translation -= change;
        }
    }
}
