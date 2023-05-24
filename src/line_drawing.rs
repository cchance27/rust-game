use bevy::{prelude::*, render::{render_resource::{ShaderRef, PrimitiveTopology, PolygonMode, SpecializedMeshPipelineError, RenderPipelineDescriptor, AsBindGroup}, mesh::MeshVertexBufferLayout}, pbr::{MaterialPipelineKey, MaterialPipeline}, reflect::TypeUuid};

#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
pub struct LineMaterial {
    #[uniform(0)]
    pub color: Color
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        
        Ok(())
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}


pub struct Square(pub Vec3);

impl From<Square> for Mesh {
    fn from(value: Square) -> Self {
        Mesh::from(LineStrip {
            points: vec![
                Vec3::ZERO, 
                Vec3::new(0.0, 0.0, value.0.z),
                value.0, 
                Vec3::new(value.0.x, 0.0, 0.0), 
                Vec3::ZERO ]})
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
