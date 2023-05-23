
use bevy::{
    pbr::{DirectionalLightShadowMap},
    prelude::*, input::mouse::{MouseButtonInput, MouseMotion, MouseWheel}, ecs::system::EntityCommands,
};

use bevy_rapier3d::{prelude::*};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
//use bevy_prototype_debug_lines::*;
use bevy_editor_pls::prelude::*;
//use bevy_mod_picking::prelude::*;

mod line_drawing;
mod camera;
use camera::CameraPlugin;
use line_drawing::*;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 2048 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
        })
        .add_plugins(DefaultPlugins)
        //.add_plugins(DefaultPickingPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin::default())
        .add_plugin(CameraPlugin)
        .add_plugin(MaterialPlugin::<LineMaterial>::default())
        .add_startup_system(spawn_world)
        .add_system(print_mouse_events_system)
        .add_system(selected_highlight)
        //.add_system(make_scenes_pickable)
        //.add_system(spawn_pickingbox)
        .run();
}

#[derive(Component)]
struct Player;

fn spawn_world(
    mut command: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) { 
    command
    .spawn(Collider::cuboid(100.0, 0.1, 100.0))
    .insert(RigidBody::Fixed)
    .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)))
    .insert(Name::new("Floor"));

    create_ball(&mut command, &mut meshes, &mut materials, 0.5, Transform::from_xyz(0.0, 4.0, 0.0));
    create_ball(&mut command, &mut meshes, &mut materials, 0.5, Transform::from_xyz(2.0, 2.0, 2.0));
}

fn create_ball(command: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, radius: f32, transform: Transform)  {
    command
      .spawn(RigidBody::Dynamic)
      .insert(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: radius - 0.001,
                    sectors: 10, 
                    stacks: 10
                })),
                material: materials.add(Color::RED.into()),
                ..default()
            })
      .insert(Collider::ball(radius))
      .insert(ColliderMassProperties::Density(2.0))
      .insert(Restitution::coefficient(0.7))
      .insert(TransformBundle::from(transform))
      .insert(SelectableItem)
      .insert(Name::new("Ball"));
}

#[derive(Component)]
pub struct PickingBox {
    pub start: Vec3,
    pub mesh: Handle<Mesh>,
}

fn print_mouse_events_system(
    mut commands: Commands,
    mouse_btn: Res<Input<MouseButton>>,
    windows: Query<&Window>, 
    camera: Query<(&Camera, &GlobalTransform), With<camera::components::PlayerCamera>>, 
    mut player: Query<Entity, With<SelectedItem>>,
    rapier_context: Res<RapierContext>
) {
    if mouse_btn.just_pressed(MouseButton::Left) {
        let y = windows.single().cursor_position().unwrap();
        let (c, g) = camera.single();
        if let Some(r) = c.viewport_to_world(g, y) {
            let max_toi = Real::MAX;
            let solid = true;
            let filter = QueryFilter::default();
            
            // entity is first collider hit, distance to hit from cam r.dir * toi
            if let Some((entity, toi)) = rapier_context.cast_ray(
                r.origin, r.direction, max_toi, solid, filter
            ) {
                let hit_point = r.origin + r.direction * toi;
                commands.get_entity(entity).unwrap().log_components();
                println!("Entity {:?} hit at point {}", entity, hit_point);
                commands.get_entity(entity).unwrap().insert(SelectingItem);
            }
        }
    }
}

#[derive(Component)]
struct SelectingItem;
#[derive(Component)]
struct SelectedItem;
#[derive(Component)]
struct SelectableItem;


fn selected_highlight(
    mut commands: Commands,
    query: Query<(Entity), (With<SelectingItem>, Without<SelectedItem>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for e in query.iter() {
        let mut m: StandardMaterial = Color::DARK_GREEN.into();
        m.alpha_mode = AlphaMode::Mask(0.5);

        let new_entity = commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.5,
                rings: 2,
                depth: 0.5,
                ..default()
            })),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            material: materials.add(m.clone()),
            ..default()
        }).id();

        let mut ec = commands.get_entity(e).unwrap();
        ec.insert(SelectedItem);
        ec.add_child(new_entity);
    }
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
