
use bevy::{
    pbr::{DirectionalLightShadowMap},
    prelude::{*}, window::PrimaryWindow,
};
use bevy_rapier3d::{prelude::*};
use bevy_editor_pls::prelude::*;
mod line_drawing;
mod camera;
use camera::CameraPlugin;
use line_drawing::*;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(Selecting::default())
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
        .add_system(mouse_item_selection)
        .add_system(add_highlight_to_selected)
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
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 100.0, 
                subdivisions: 1
            })),
            material: materials.add(Color::GREEN.into()),
            ..default()})
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
      .insert(Name::new("Ball"));
}

#[derive(Resource, Default)]
struct Selecting { 
    first_entity: Option<Entity>,
    first_click: Vec3,
    picking_mesh: Option<Handle<Mesh>>,
    picking_box: Option<Entity>
}
#[derive(Component)]
struct PendingSelection;
#[derive(Component)]
struct SelectedUnit;

fn mouse_item_selection(
    mut commands: Commands,
    mut selecting: ResMut<Selecting>,
    mouse_btn: Res<Input<MouseButton>>,
    keyboard_btn: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    windows: Query<&Window, With<PrimaryWindow>>, 
    camera: Query<(&Camera, &GlobalTransform), With<camera::components::PlayerCamera>>, 
    existing_selections: Query<Entity, With<SelectedUnit>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    // Guarded early returns if we don't have a cursor or camera
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok((camera, camera_location)) = camera.get_single() else { return; };

    // We just started selecting something with left mouse button
    // Find what we're pointing at and set it as being the start of our selection.
    if mouse_btn.just_pressed(MouseButton::Left) {
        let Some(hit_entity) = screen_ray_to_entity(&camera, &rapier_context, &camera_location, cursor_position) else { return; };
        commands.insert_resource(Selecting { first_entity: Some(hit_entity.0), first_click: hit_entity.1, picking_box: None, picking_mesh: None });
        return;
    }

    if mouse_btn.pressed(MouseButton::Left) {
        let Some(hit_entity) = screen_ray_to_entity(&camera, &rapier_context, &camera_location, cursor_position) else { return; };
        let local_position = hit_entity.1 - selecting.first_click;
        // Has our mouse moved from it's previous spot?
        if selecting.first_click.distance(hit_entity.1) > 0.0 {
            // We've moved our mouse from the start point 
            if let Some(picking_mesh) = &selecting.picking_mesh {
                // We've already created a picking_box so just update it's mesh to the new size
                let x = meshes.get_mut(picking_mesh).unwrap();
                *x = Mesh::from(LineStrip {
                    points: vec![
                        Vec3::ZERO, 
                        Vec3::new(0.0, 0.0, local_position.z),
                        local_position, 
                        Vec3::new(local_position.x, 0.0, 0.0), 
                        Vec3::ZERO ]})
            } else {
                // This is the first time we've moved, so we need to generate the selection box object
                let mesh = meshes.add(Mesh::from(LineStrip { 
                    points: vec![
                        Vec3::ZERO, 
                        Vec3::new(0.0, 0.0, local_position.z),
                        local_position, 
                        Vec3::new(local_position.x, 0.0, 0.0), 
                        Vec3::ZERO ]}));
                let pickbox = commands.spawn(MaterialMeshBundle  {
                    mesh: mesh.clone(),
                    material: materials.add(LineMaterial { color: Color::WHITE }),
                    transform: Transform::from_translation(selecting.first_click + Vec3::Y * 0.02),
                    ..default()
                }).insert(Name::new("line")).id();
                selecting.picking_mesh = Some(mesh);
                selecting.picking_box = Some(pickbox)
            }
        }

        // Early return, as we haven't moved or we handled the movement already
        return;
    }

    if mouse_btn.just_released(MouseButton::Left) {
        //  If we're holding shift and we just released, we don't clear previous selections
        if !keyboard_btn.pressed(KeyCode::LShift) && !keyboard_btn.pressed(KeyCode::RShift) {
            existing_selections.iter().for_each(|old_selection| {
                if let Some(entity) = commands.get_entity(old_selection) { entity.despawn_recursive() }
            }) 
        }
        
        // Check where we were pointing where we let go of the click or drag
        if let Some(hit_entity) = screen_ray_to_entity(&camera, &rapier_context, &camera_location, cursor_position) {
            // Is the first item we were on the same as the last item we were over (single click)
            if selecting.first_entity.is_some() && selecting.first_entity.unwrap() == hit_entity.0 { 
                // 1 Item so just select it, if it exists still                
                if let Some(mut entity_to_select) = commands.get_entity(hit_entity.0) {
                    entity_to_select.insert(PendingSelection);
                }
            } else {
                // Oh shit we dragged somewhere and let go we've got a box we need to grab the internal entities.
            }

            // Drop our picking box entity, and then reset our selecting resource
            if let Some(pickingbox) = selecting.picking_box {
                commands.get_entity(pickingbox).unwrap().despawn_recursive();
            }
            commands.insert_resource(Selecting::default());
        }  
    }
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

fn add_highlight_to_selected(
    mut commands: Commands,
    query: Query<Entity, With<PendingSelection>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for e in query.iter() {
        let material: StandardMaterial = StandardMaterial {
            base_color: Color::rgba(0.0, 0.0, 0.0, 0.4),
            //base_color_texture: Some(texture_handle.clone()),
            alpha_mode: AlphaMode::Blend,
            //unlit: true,
            ..default()
        };

        let new_entity = commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Torus {
                    radius: 0.8,
                    ring_radius: 0.2,
                    subdivisions_segments: 40,
                    subdivisions_sides: 6
                })),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: materials.add(material),
                ..default()
            }, 
            SelectedUnit
        )).id();

        let mut ec = commands.get_entity(e).unwrap();
        ec.remove::<PendingSelection>();
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
