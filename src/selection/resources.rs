use bevy::prelude::*;
use bevy_polyline::prelude::*;

#[derive(Resource, Default)]
pub struct Selecting {
    pub first_entity: Option<Entity>,
    pub first_click: Vec3,
    pub last_click: Vec3,
    pub picking_mesh: Option<Handle<Polyline>>,
    pub picking_box: Option<Entity>,
}
