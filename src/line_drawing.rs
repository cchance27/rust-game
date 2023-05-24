use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use bevy_polyline::prelude::*;

pub struct Square(pub Vec3);

impl From<Square> for Polyline {
    fn from(value: Square) -> Self {
        Polyline {
            vertices: vec![
                Vec3::ZERO, 
                Vec3::new(0.0, 0.0, value.0.z),
                value.0, 
                Vec3::new(value.0.x, 0.0, 0.0), 
                Vec3::ZERO ]
        }
    }
}