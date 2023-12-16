use bytemuck::{offset_of, Pod, Zeroable};
use gltf_json::accessor::{ComponentType, GenericComponentType, Type};
use gltf_json::buffer::{Target, View};
use gltf_json::mesh::{Mode, Primitive, Semantic};
use gltf_json::validation::Checked::Valid;
use gltf_json::{Accessor, Index, Mesh, Value};
use std::mem::size_of;
use vmdl::Model;

#[derive(Copy, Clone, Debug, Default, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl From<&vmdl::vvd::Vertex> for Vertex {
    fn from(vertex: &vmdl::vvd::Vertex) -> Self {
        Vertex {
            position: vertex.position.into(),
            uv: vertex.texture_coordinates.into(),
            normal: vertex.normal.into(),
        }
    }
}

fn push_vertices(
    buffer: &mut Vec<u8>,
    views: &mut Vec<View>,
    accessors: &mut Vec<Accessor>,
    model: &Model,
) {
    let start = buffer.len() as u32;
    let view_start = views.len() as u32;
    let vertex_count = model.vertices().len() as u32;

    let (min, max) = model.bounding_box();
    let min = <[f32; 3]>::from(min);
    let max = <[f32; 3]>::from(max);

    let vertex_data = model
        .vertices()
        .iter()
        .map(Vertex::from)
        .flat_map(bytemuck::cast::<_, [u8; size_of::<Vertex>()]>);
    buffer.extend(vertex_data);

    let vertex_buffer_view = View {
        buffer: Index::new(0),
        byte_length: buffer.len() as u32 - start,
        byte_offset: Some(start),
        byte_stride: Some(size_of::<Vertex>() as u32),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(Target::ArrayBuffer)),
    };

    views.push(vertex_buffer_view);

    let positions = Accessor {
        buffer_view: Some(Index::new(view_start)),
        byte_offset: Some(offset_of!(Vertex, position) as u32),
        count: vertex_count,
        component_type: Valid(GenericComponentType(ComponentType::F32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(Type::Vec3),
        min: Some(Value::from(Vec::from(min))),
        max: Some(Value::from(Vec::from(max))),
        name: None,
        normalized: false,
        sparse: None,
    };
    let uvs = Accessor {
        buffer_view: Some(Index::new(view_start)),
        byte_offset: Some(offset_of!(Vertex, uv) as u32),
        count: vertex_count,
        component_type: Valid(GenericComponentType(ComponentType::F32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(Type::Vec2),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    };
    let normals = Accessor {
        buffer_view: Some(Index::new(view_start)),
        byte_offset: Some(offset_of!(Vertex, normal) as u32),
        count: vertex_count,
        component_type: Valid(GenericComponentType(ComponentType::F32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(Type::Vec3),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    };

    accessors.extend([positions, uvs, normals]);
}

pub fn push_model(
    buffer: &mut Vec<u8>,
    views: &mut Vec<View>,
    accessors: &mut Vec<Accessor>,
    model: &Model,
) -> Mesh {
    let accessor_start = accessors.len() as u32;
    push_vertices(buffer, views, accessors, model);

    let primitives = model
        .meshes()
        .map(|mesh| push_primitive(buffer, views, accessors, &mesh, accessor_start))
        .collect();

    Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(model.name().into()),
        primitives,
        weights: None,
    }
}

pub fn push_primitive(
    buffer: &mut Vec<u8>,
    views: &mut Vec<View>,
    accessors: &mut Vec<Accessor>,
    mesh: &vmdl::Mesh,
    vertex_accessor_start: u32,
) -> Primitive {
    let buffer_start = buffer.len() as u32;
    let view_start = views.len() as u32;
    let accessor_start = accessors.len() as u32;

    buffer.extend(
        mesh.vertex_strip_indices()
            .flatten()
            .flat_map(|index| (index as u32).to_le_bytes()),
    );

    let byte_length = buffer.len() as u32 - buffer_start;

    let view = View {
        buffer: Index::new(0),
        byte_length,
        byte_offset: Some(buffer_start),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(Target::ElementArrayBuffer)),
    };
    views.push(view);

    let accessor = Accessor {
        buffer_view: Some(Index::new(view_start)),
        byte_offset: Some(0),
        count: byte_length / size_of::<u32>() as u32,
        component_type: Valid(GenericComponentType(ComponentType::U32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(Type::Scalar),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    };
    accessors.push(accessor);

    Primitive {
        attributes: {
            let mut map = std::collections::BTreeMap::new();
            map.insert(
                Valid(Semantic::Positions),
                Index::new(vertex_accessor_start),
            );
            map.insert(
                Valid(Semantic::TexCoords(0)),
                Index::new(vertex_accessor_start + 1),
            );
            map.insert(
                Valid(Semantic::Normals),
                Index::new(vertex_accessor_start + 2),
            );
            map
        },
        extensions: Default::default(),
        extras: Default::default(),
        indices: Some(Index::new(accessor_start)),
        material: None,
        mode: Valid(Mode::Triangles),
        targets: None,
    }
}
