use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Selecting { 
    pub first_entity: Option<Entity>,
    pub first_click: Vec3,
    pub last_click: Vec3,
    pub picking_mesh: Option<Handle<Mesh>>,
    pub picking_box: Option<Entity>
}