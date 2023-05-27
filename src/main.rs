#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use std::f32::consts::PI;

use bevy::{
    pbr::{DirectionalLightShadowMap},
    prelude::{*}, ecs::system::EntityCommands,
};
use bevy_turborand::rng::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::RigidBodyBuilder};
use bevy_editor_pls::prelude::*;
use bevy_mod_gizmos::{prelude::Gizmos, GizmoConfig, GizmoPlugin};

mod line_drawing;
mod camera;
mod selection;

use camera::CameraPlugin;
use selection::{SelectionPlugin, components::Selectable};

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
        .insert_resource(Ground { size: 100, subdivisions: 1, ..default() })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 0.1 })
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
        .run();
}

#[derive(Resource, Default)]
pub struct Ground {
    size: i32,
    subdivisions: u32,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    entity: Option<Entity>
}

fn spawn_ground(
    mut commands: Commands, 
    mut ground: ResMut<Ground>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let mesh = meshes.add(Mesh::from(shape::Plane {
        size: ground.size as f32, 
        subdivisions: ground.subdivisions
    })); 

    let material =  materials.add(Color::GREEN.into());

    let ground_id = commands.spawn(PbrBundle { mesh: mesh.clone(), material: material.clone(), ..default() })
        .insert(Collider::cuboid((ground.size/2) as f32, 0.1, (ground.size/2) as f32))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.1, 0.0)))
        .insert(Name::new("Ground")).id();

    ground.mesh = mesh;
    ground.material = material; 
    ground.entity = Some(ground_id);

    //TODO: Perform noise or some form of generation to make the ground actually interesting.
}

fn draw_gizmos(
    mut units: Query<(&Transform, &UnitSize, &UnitView), With<Player>>,
    mut gizmos: Gizmos, 
    time: Res<Time>,
    rapier_context: Res<RapierContext>
) {
    let sensitivity = 5;

    for (unit_transform, unit_size, unit_view) in units.iter_mut() {
        let origin = unit_transform.translation + (unit_transform.forward() * unit_size.collider/2.0);

        let view_results = scan_fov(&rapier_context, &unit_transform, unit_view.fov, origin, sensitivity, unit_view.distance);
        view_results.iter().for_each(|result| {
            match result.entity {
                Some(_) => gizmos.ray(origin, result.hit_location,Color::GREEN),
                None => gizmos.ray(origin, result.hit_location,Color::RED),
            }
        });
    }
}

fn scan_fov(rapier_context: &RapierContext, unit_transform: &Transform, fov: i16, origin: Vec3, sensitivity: usize, distance: f32) -> Vec<FovScanResult> {
    let mut results: Vec<FovScanResult> = Vec::new();

    let filter = QueryFilter::default(); 
    let solid = true;
    let half_fov = fov/2;
    let (yaw, _, _) = unit_transform.rotation.to_euler(EulerRot::YXZ);

    for angle in (-half_fov..half_fov).step_by(sensitivity) {
        let direction =  direction_from_angle(angle, yaw);
        if let Some((entity, toi)) = rapier_context.cast_ray(origin, direction, distance, solid, filter) {
            results.push(FovScanResult {angle, hit_location: direction * toi, entity: Some(entity)});
        } else {
            results.push(FovScanResult {angle,  hit_location: direction * distance, entity: None });
        }
    }

    results
}

#[derive(Debug, Default)]
struct FovScanResult {
    angle: i16, 
    hit_location: Vec3, 
    entity: Option<Entity>
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
    ground: Res<Ground>
) { 
    let rand = Rng::new();
    let min = -ground.size/2;
    let max = ground.size/2;

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        ..default()
    });

    for i in 0..1_000 {
        let x = rand.i32(min..max) as f32;
        let y =rand.i32(min..max) as f32;
        let id = create_cube(&mut commands, Color::WHITE, &mut meshes, &mut materials, 0.5, Vec3::new(x, 0.0, y));
        commands.entity(id)
            .insert(Enemy)
            .insert(UnitMovement { turn_speed: 0.5, move_speed: 5.0 })
            .insert(UnitSize { collider: 0.502, model: 0.5 })
            .insert(UnitView { fov: 90, distance: 10.0 });
    }

    let player_id = create_cube(&mut commands, Color::RED, &mut meshes, &mut materials, 0.5, Vec3::new(0.0, 0.0, 1.0));
    commands.entity(player_id)
        .insert(Player)
        .insert(UnitMovement { turn_speed: 0.5, move_speed: 5.0 })
        .insert(UnitSize { collider: 0.502, model: 0.5 })
        .insert(UnitView { fov: 90, distance: 10.0 });
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
pub struct UnitMovement {
    turn_speed: f32,
    move_speed: f32    
}

#[derive(Component)]
pub struct UnitSize {
    collider: f32,
    model: f32    
}

#[derive(Component)]
pub struct UnitView {
    fov: i16, 
    distance: f32
}
fn create_cube(commands: &mut Commands, color: Color, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, size: f32, location: Vec3) -> Entity {
    let collider_size = (size / 2.0) + 0.0001;
    let offset_location = location + Vec3::Y * (size / 2.0);
    commands
      .spawn(RigidBody::Fixed)
      .insert(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube {
                    size: size
                })),
                material: materials.add(color.into()),
                ..default()
            })
      .insert(Collider::cuboid(collider_size, collider_size, collider_size))
      .insert(ColliderMassProperties::Density(2.0))
      .insert(Restitution::coefficient(0.7))
      .insert(TransformBundle::from(Transform::from_translation(offset_location)))
      .insert(Selectable)
      .id()
}

fn screen_ray_to_entity(camera: &Camera, rapier_context: &RapierContext, camera_location: &GlobalTransform, cursor_position: Vec2) -> Option<(Entity, Vec3)> {
    if let Some(check_ray) = camera.viewport_to_world(camera_location, cursor_position) {
        let max_toi = 5000.0; // Hard value probably shouldn't be
        let solid = true;
        let filter = QueryFilter::default(); //TODO: Don't filter because we want to hit anything, we might want to filter out invisible stuff like sensors.

        if let Some((entity, toi)) = rapier_context.cast_ray(
            check_ray.origin, check_ray.direction, max_toi, solid, filter
        ) {
            let hit_point = check_ray.origin + check_ray.direction * toi;
            return Some((entity, hit_point));
        }    
    }

    None
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
