use crate::Vertex;
use vmdl::Model;

pub fn model_to_vertices(model: &Model) -> Vec<Vertex> {
    model
        .meshes()
        .flat_map(|mesh| mesh.vertices())
        .map(|vertex| Vertex {
            position: vertex.position.into(),
            uv: vertex.texture_coordinates.into(),
            normal: vertex.normal.into(),
        })
        .collect()
}
