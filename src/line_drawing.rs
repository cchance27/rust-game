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

#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}
impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}

/// A list of points that will have a line drawn between each consecutive points
#[derive(Debug, Clone)]
pub struct LineStrip {
    pub points: Vec<Vec3>,
}
impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line.points);
        mesh
    }
}
