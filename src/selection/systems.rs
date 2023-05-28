use bevy::{prelude::*, window::PrimaryWindow};
use bevy_polyline::prelude::*;
use bevy_rapier3d::prelude::RapierContext;

use crate::{camera::components::PlayerCamera, line_drawing::Square, screen_ray_to_entity};

use super::{components::*, resources::*};

pub fn handle_mouse_input_selection(
    mut commands: Commands,
    mouse_btn: Res<Input<MouseButton>>,
    keyboard_btn: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut selecting: ResMut<Selecting>,
    windows: Query<&Window, With<PrimaryWindow>>,
    previous_sel_entities: Query<(Entity, &SelectedUnit)>,
    selectable: Query<(Entity, &GlobalTransform), With<Selectable>>,
    camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
) {
    // Guarded early returns if we don't have a cursor or camera
    let Ok(window) = windows.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok((camera, camera_location)) = camera.get_single() else { return; };
    let Some(hit_entity) = screen_ray_to_entity(camera, &rapier_context, camera_location, cursor_position) else { return; };

    // TODO: The selection box is currently tied to the world x/y not camera x/y so boxes aren't good.

    // We just started selecting something with left mouse button
    // Find what we're pointing at and set it as being the start of our selection.
    if mouse_btn.just_pressed(MouseButton::Left) {
        if selectable.contains(hit_entity.0) {
            // We started on something non-selectable
            commands.insert_resource(Selecting {
                first_entity: Some(hit_entity.0),
                first_click: hit_entity.1,
                last_click: hit_entity.1,
                ..default()
            });
        } else {
            commands.insert_resource(Selecting {
                first_entity: None,
                first_click: hit_entity.1,
                last_click: hit_entity.1,
                ..default()
            });
        }
        return;
    }

    // We're likely dragging, check if we've moved if we have update our selecting state.
    if mouse_btn.pressed(MouseButton::Left) {
        if selecting.first_click.distance(hit_entity.1) > 0.0 {
            selecting.last_click = hit_entity.1;
        }

        return;
    }

    // We've released the mouse so we're done selecting one way or another.
    if mouse_btn.just_released(MouseButton::Left) {
        //  If we're holding shift and we just released, we don't clear previous selections
        if !keyboard_btn.pressed(KeyCode::LShift) && !keyboard_btn.pressed(KeyCode::RShift) {
            previous_sel_entities
                .iter()
                .for_each(|(entity, selection_icon)| {
                    if let Some(mut entity) = commands.get_entity(entity) {
                        // Despawn our selection component, and despawn it's mesh
                        entity.remove::<SelectedUnit>();
                        commands.entity(selection_icon.0).despawn_recursive();
                    }
                })
        }

        // Is the first item we were on the same as the last item we were over (single click)
        if selecting.first_entity.is_some() && selecting.first_entity.unwrap() == hit_entity.0 {
            // 1 Item so just select it, if it exists still
            if let Some(mut entity_to_select) = commands.get_entity(hit_entity.0) {
                entity_to_select.insert(PendingSelection);
            }
        } else {
            // MultiSect all in the box
            selectable
                .iter()
                .filter(|(_e, gt)| {
                    let tf = gt.translation();
                    let within_x = (selecting.first_click.x < tf.x
                        && selecting.last_click.x > tf.x)
                        || (selecting.last_click.x < tf.x && selecting.first_click.x > tf.x);
                    let within_z = (selecting.first_click.z < tf.z
                        && selecting.last_click.z > tf.z)
                        || (selecting.last_click.z < tf.z && selecting.first_click.z > tf.z);
                    within_x && within_z
                })
                .for_each(|valid| {
                    if let Some(mut entity_to_select) = commands.get_entity(valid.0) {
                        entity_to_select.insert(PendingSelection);
                    }
                });
        }

        // Drop our picking box entity, and then reset our selecting resource
        if let Some(pickingbox) = selecting.picking_box {
            commands.get_entity(pickingbox).unwrap().despawn_recursive();
        }

        commands.insert_resource(Selecting::default());
    }
}

pub fn draw_selection_indicator(
    mut commands: Commands,
    query: Query<Entity, With<PendingSelection>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.0, 0.0, 0.4),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mesh = meshes.add(Mesh::from(shape::Torus {
        radius: 1.0,
        ring_radius: 0.1,
        subdivisions_segments: 40,
        subdivisions_sides: 6,
    }));

    for e in query.iter() {
        let selection_icon_entity = commands
            .spawn(PbrBundle {
                mesh: mesh.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: material.clone(),
                ..default()
            })
            .id();

        let mut ec = commands.get_entity(e).unwrap();
        ec.remove::<PendingSelection>();
        ec.add_child(selection_icon_entity);
        ec.insert(SelectedUnit(selection_icon_entity));
    }
}

pub fn draw_selection_box(
    mut commands: Commands,
    mut selecting: ResMut<Selecting>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    if selecting.first_click == selecting.last_click {
        return;
    }
    // TODO: Replace pin to ground ith a projected test to draw shorter lines up and down hills.

    let mut local_pos = selecting.last_click - selecting.first_click;
    local_pos.y = 0.0;

    let y_offset = 0.02;
    let poly = Square(local_pos).into();

    if let Some(picking_mesh) = &selecting.picking_mesh {
        // We've already created a picking_box so just update it's mesh to the new size
        let x = polylines.get_mut(picking_mesh).unwrap();
        *x = poly;
    } else {
        // This is the first time we've moved, so we need to generate the selection box object
        let mesh = polylines.add(Square(local_pos).into());
        let pickbox = commands
            .spawn(PolylineBundle {
                polyline: mesh.clone(),
                material: polyline_materials.add(PolylineMaterial {
                    color: Color::ORANGE_RED,
                    width: 100.0,
                    perspective: true,
                    ..default()
                }),
                transform: Transform::from_translation(selecting.first_click + Vec3::Y * y_offset),
                ..default()
            })
            .insert(Name::new("line"))
            .id();
        selecting.picking_mesh = Some(mesh);
        selecting.picking_box = Some(pickbox)
    }
}
