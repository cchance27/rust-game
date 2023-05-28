#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use itertools::Itertools;
use std::f32::consts::PI;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::system::EntityCommands,
    input::mouse::MouseMotion,
    pbr::DirectionalLightShadowMap,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_editor_pls::prelude::*;
use bevy_mod_gizmos::{prelude::Gizmos, GizmoConfig, GizmoPlugin};
use bevy_rapier3d::{prelude::*, rapier::prelude::RigidBodyBuilder};
use bevy_turborand::rng::*;

mod camera;
mod line_drawing;
mod selection;

use camera::{components::PlayerCamera, CameraPlugin};
use selection::{
    components::{Selectable, SelectedUnit},
    SelectionPlugin,
};

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(MouseLocation::default())
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
        .insert_resource(Ground {
            size: 100,
            subdivisions: 1,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.1,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(GizmoPlugin)
        //.add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin::default())
        .add_plugin(CameraPlugin)
        .add_plugin(SelectionPlugin)
        .add_startup_system(spawn_world)
        .add_startup_system(spawn_ground)
        .add_system(draw_gizmos)
        .add_system(mouse_click_set_movement_target)
        .add_system(move_to_location)
        .add_system(track_mouse_location)
        .run();
}

#[derive(Resource, Default)]
struct MouseLocation(Option<Vec3>);

fn track_mouse_location(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse: EventReader<MouseMotion>,
    camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    rapier_context: Res<RapierContext>,
) {
    if mouse.iter().len() > 0 {
        let Ok(window) = windows.get_single() else { return; };
        let Some(cursor_position) = window.cursor_position() else { return; };
        let Ok((camera, camera_location)) = camera.get_single() else { return; };
        let Some((entity, loc)) = screen_ray_to_entity(camera, &rapier_context, camera_location, cursor_position) else { return; };

        commands.insert_resource(MouseLocation(Some(loc)));
    }
}

fn mouse_click_set_movement_target(
    mut commands: Commands,
    mouse_btn: Res<Input<MouseButton>>,
    mouse_loc: Res<MouseLocation>,
    selected_units: Query<Entity, With<SelectedUnit>>,
) {
    if mouse_btn.just_released(MouseButton::Right) && mouse_loc.0.is_some() {
        let Some(loc) = mouse_loc.0 else { return; };

        for unit in selected_units.iter() {
            commands.entity(unit).insert(WalkToLocation(loc));
        }
    }
}

fn move_to_location(
    mut commands: Commands,
    mut player_with_move: Query<(Entity, &mut Transform, &WalkToLocation)>,
    time: Res<Time>,
) {
    let speed = 2.0;
    for (entity, mut transform, target) in player_with_move.iter_mut() {
        let direction = target.0 - transform.translation;

        transform.translation += direction * speed * time.delta_seconds();

        if transform.translation.distance(target.0) <= 0.001 {
            commands.entity(entity).remove::<WalkToLocation>();
        }
    }
}

#[derive(Resource, Default)]
pub struct Ground {
    size: i32,
    subdivisions: u32,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    entity: Option<Entity>,
}

fn spawn_ground(
    mut commands: Commands,
    mut ground: ResMut<Ground>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane {
        size: ground.size as f32,
        subdivisions: ground.subdivisions,
    }));

    let material = materials.add(Color::GREEN.into());

    let ground_id = commands
        .spawn(PbrBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            ..default()
        })
        .insert(Collider::cuboid(
            (ground.size / 2) as f32,
            0.1,
            (ground.size / 2) as f32,
        ))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.1, 0.0)))
        .insert(Name::new("Ground"))
        .id();

    ground.mesh = mesh;
    ground.material = material;
    ground.entity = Some(ground_id);

    //TODO: Perform noise or some form of generation to make the ground actually interesting.
}

fn draw_gizmos(
    commands: Commands,
    units: Query<(&Transform, &UnitSize, &UnitView), With<Enemy>>,
    mut gizmos: Gizmos,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
) {
    let sensitivity = 10;

    for (unit_transform, unit_size, unit_view) in units.iter() {
        let origin =
            unit_transform.translation + (unit_transform.forward() * unit_size.collider / 2.0);

        let view_results = scan_fov(
            &rapier_context,
            unit_transform,
            unit_view.fov,
            origin,
            sensitivity,
            unit_view.distance,
        );
        view_results.iter().for_each(|result| match result.entity {
            Some(e) => {
                gizmos.ray(origin, result.hit_location, Color::GREEN);
            }
            None => gizmos.ray(origin, result.hit_location, Color::RED),
        });

        // Mark the ones we can see.
        // view_results.iter().filter_map(|x| x.entity).unique().for_each(|visible| {
        //    commands.entity(visible).insert(VisibleToPlayer);
        // });
    }
}

#[derive(Component)]
struct VisibleToPlayer;

fn scan_fov(
    rapier_context: &RapierContext,
    unit_transform: &Transform,
    fov: i16,
    origin: Vec3,
    sensitivity: usize,
    distance: f32,
) -> Vec<FovScanResult> {
    let mut results: Vec<FovScanResult> = Vec::new();

    let filter = QueryFilter::default();
    let solid = true;
    let half_fov = fov / 2;
    let (yaw, _, _) = unit_transform.rotation.to_euler(EulerRot::YXZ);

    for angle in (-half_fov..half_fov).step_by(sensitivity) {
        let direction = direction_from_angle(angle, yaw);
        if let Some((entity, toi)) =
            rapier_context.cast_ray(origin, direction, distance, solid, filter)
        {
            results.push(FovScanResult {
                angle,
                hit_location: direction * toi,
                entity: Some(entity),
            });
        } else {
            results.push(FovScanResult {
                angle,
                hit_location: direction * distance,
                entity: None,
            });
        }
    }

    results
}

#[derive(Debug, Default)]
struct FovScanResult {
    angle: i16,
    hit_location: Vec3,
    entity: Option<Entity>,
}

fn direction_from_angle(angle: impl Into<f32> + Copy, yaw_radians: f32) -> Vec3 {
    let x: f32 = angle.into().to_radians() + yaw_radians;
    let z: f32 = angle.into().to_radians() + yaw_radians;
    -Vec3::new(x.sin(), 0.0, z.cos())
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground: Res<Ground>,
) {
    let rand = Rng::new();
    let max = ground.size / 2 - 2;

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        ..default()
    });

    const SPAWN_AMT: usize = 1000;
    let mut spawned: Vec<Vec3> = vec![];

    for i in 0..SPAWN_AMT {
        let r = rand.i32(0..360) as f32;

        let mut valid_new_location: Option<Vec3> = None;

        for _ in 0..100 {
            let test_new_location =
                Vec3::new(rand.i32(-max..max) as f32, 0.0, rand.i32(-max..max) as f32);
            if spawned
                .iter()
                .all(|exist| test_new_location.distance(*exist) > 0.5 * 3.0)
            {
                spawned.push(test_new_location);
                valid_new_location = Some(test_new_location);
                break;
            };
        }

        if let Some(valid_new_location) = valid_new_location {
            let id = create_cube(
                &mut commands,
                Color::WHITE,
                &mut meshes,
                &mut materials,
                0.5,
                valid_new_location,
                r,
            );

            commands
                .entity(id)
                .insert(Enemy)
                .insert(UnitMovement {
                    turn_speed: 0.5,
                    move_speed: 5.0,
                })
                .insert(UnitSize {
                    collider: 0.502,
                    model: 0.5,
                })
                .insert(UnitView {
                    fov: 90,
                    distance: 5.0,
                });
        } else {
            error!("No valid Location for enemy spawn found.");
        }
    }

    let player_id = create_cube(
        &mut commands,
        Color::RED,
        &mut meshes,
        &mut materials,
        0.5,
        Vec3::new(0.0, 0.0, 1.0),
        0.0,
    );
    commands
        .entity(player_id)
        .insert(Player)
        .insert(UnitMovement {
            turn_speed: 0.5,
            move_speed: 5.0,
        })
        .insert(UnitSize {
            collider: 0.502,
            model: 0.5,
        })
        .insert(UnitView {
            fov: 120,
            distance: 10.0,
        });
}

fn create_cube(
    commands: &mut Commands,
    color: Color,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    size: f32,
    location: Vec3,
    rotation: f32,
) -> Entity {
    let collider_size = (size / 2.0) + 0.0001;
    let offset_location = location + Vec3::Y * (size / 2.0);
    commands
        .spawn(RigidBody::Fixed)
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size })),
            material: materials.add(color.into()),
            ..default()
        })
        .insert(Collider::cuboid(
            collider_size,
            collider_size,
            collider_size,
        ))
        .insert(ColliderMassProperties::Density(2.0))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(
            Transform::from_translation(offset_location)
                .with_rotation(Quat::from_rotation_y(rotation)),
        ))
        .insert(Selectable)
        .id()
}

fn screen_ray_to_entity(
    camera: &Camera,
    rapier_context: &RapierContext,
    camera_location: &GlobalTransform,
    cursor_position: Vec2,
) -> Option<(Entity, Vec3)> {
    if let Some(check_ray) = camera.viewport_to_world(camera_location, cursor_position) {
        let max_toi = 5000.0; // Hard value probably shouldn't be
        let solid = true;
        let filter = QueryFilter::default(); //TODO: Don't filter because we want to hit anything, we might want to filter out invisible stuff like sensors.

        if let Some((entity, toi)) = rapier_context.cast_ray(
            check_ray.origin,
            check_ray.direction,
            max_toi,
            solid,
            filter,
        ) {
            let hit_point = check_ray.origin + check_ray.direction * toi;
            return Some((entity, hit_point));
        }
    }

    None
}

#[derive(Component)]
struct WalkToLocation(Vec3);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
pub struct UnitMovement {
    turn_speed: f32,
    move_speed: f32,
}

#[derive(Component)]
pub struct UnitSize {
    collider: f32,
    model: f32,
}

#[derive(Component)]
pub struct UnitView {
    fov: i16,
    distance: f32,
}

// #[derive(Component)]
// struct MakeThisPickable;
//
// fn set_pickible_recursive(
// commands: &mut Commands,
// entity: &Entity,
// mesh_query: &Query<(Entity, &Parent), With<Handle<Mesh>>>,
// children_query: &Query<&Children>,
// ) {
// for (mesh_entity, mesh_parent) in mesh_query.iter() {
// if mesh_parent.get() == *entity {
// commands
// .entity(mesh_entity)
// .insert(PickableBundle::default())
// .insert(RaycastPickTarget::default())
// .insert(OnPointer::<DragStart>::target_remove::<Pickable>())
// .insert(OnPointer::<Drag>::run_callback(mouse_drop));
// }
// }
//
// if let Ok(children) = children_query.get(*entity) {
// for child in children.iter() {
// set_pickible_recursive(commands, child, mesh_query, children_query);
// }
// }
// }
//
// fn make_scenes_pickable(
// mut commands: Commands,
// mut unpickable_query: Query<(Entity, &Name), With<MakeThisPickable>>,
// mesh_query: Query<(Entity, &Parent), With<Handle<Mesh>>>,
// children_query: Query<&Children>,
// ) {
// for (entity, name) in unpickable_query.iter_mut() {
// info!(" [MODELS] Setting Pickable on {name}");
// set_pickible_recursive(&mut commands, &entity, &mesh_query, &children_query);
// commands.entity(entity).remove::<MakeThisPickable>();
// }
// }
