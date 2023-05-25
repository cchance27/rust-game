#![allow(unused_imports)]
#![allow(unused_variables)]
use bevy::{
    pbr::{DirectionalLightShadowMap},
    prelude::{*}, ecs::system::EntityCommands,
};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_turborand::rng::*;
use bevy_rapier3d::{prelude::*};
use bevy_editor_pls::prelude::*;
use bevy_mod_gizmos::{prelude::Gizmos, GizmoConfig, GizmoPlugin};

mod line_drawing;
mod camera;
mod selection;

use camera::CameraPlugin;
use selection::{SelectionPlugin, components::Selectable};

fn main() {
    App::new()
        //.insert_resource(Msaa::default())
        //.insert_resource(DirectionalLightShadowMap { size: 2048 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
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
        .add_system(draw_gizmos)
        .run();
}

fn draw_gizmos(
    balls: Query<&GlobalTransform, With<Player>>,
    mut gizmos: Gizmos, 
    time: Res<Time>
) {
    for ball in balls.iter() {
        gizmos.ray(
            ball.translation(),Vec3::new(1.0, 0.25, 0.0),Color::BLUE,
        );
    }
}

#[derive(Component)]
struct Source;

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) { 
    commands
    .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 200.0, 
                subdivisions: 1
            })),
            material: materials.add(Color::GREEN.into()),
            ..default()})
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(RigidBody::Fixed)
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -0.1, 0.0)))
        .insert(Name::new("Floor"));
    
    let rand = Rng::new();
    for i in 0..1_000 {
        let x = rand.i32(-99..=99) as f32;
        let y =rand.i32(-99..=99) as f32;
        let _ = create_ball(&mut commands, Color::WHITE, &mut meshes, &mut materials, 0.5, Transform::from_xyz(x, 0.0, y));
    }

    let player_id = create_ball(&mut commands, Color::RED, &mut meshes, &mut materials, 0.5, Transform::from_xyz(0.1, 0.0, 0.1));
    commands.entity(player_id).insert(Player);
}

#[derive(Component)]
struct Player;

fn create_ball(commands: &mut Commands, color: Color, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, radius: f32, transform: Transform) -> Entity {
    commands
      .spawn(RigidBody::Dynamic)
      .insert(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: radius - 0.001,
                    sectors: 5, 
                    stacks: 5
                })),
                material: materials.add(color.into()),
                ..default()
            })
      .insert(Collider::ball(radius))
      .insert(ColliderMassProperties::Density(2.0))
      .insert(Restitution::coefficient(0.7))
      .insert(TransformBundle::from(transform))
      .insert(Selectable)
      .insert(Name::new("Ball")).id()
}

fn screen_ray_to_entity(camera: &Camera, rapier_context: &RapierContext, camera_location: &GlobalTransform, cursor_position: Vec2) -> Option<(Entity, Vec3)> {
    if let Some(check_ray) = camera.viewport_to_world(camera_location, cursor_position) {
        let max_toi = 5000.0; // Hard value probably shouldn't be
        let solid = true;
        let filter = QueryFilter::default(); //TODO: Don't filter because we want to hit anything, we might want to filter out invisible stuff like sensors.

        // Single selector because we hit a specific item            
        if let Some((entity, toi)) = rapier_context.cast_ray(
            check_ray.origin, check_ray.direction, max_toi, solid, filter
        ) {
            let hit_point = check_ray.origin + check_ray.direction * toi;
            //warn!("Entity {:?} hit at click ray {}", entity, hit_point);
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
