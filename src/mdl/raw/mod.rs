use crate::{index_range, FixedString, Transform3x4};
use crate::{ModelError, ReadRelative, Vector};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

mod animation;
mod bones;
pub mod header;
pub mod header2;

pub use animation::*;
pub use bones::*;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct BodyPartHeader {
    pub name_index: i32,
    model_count: i32,
    pub base: i32,
    model_index: i32,
}

impl BodyPartHeader {
    pub fn model_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.model_index,
            self.model_count,
            size_of::<ModelHeader>() - size_of::<FixedString<0>>(),
        )
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct ModelHeader {
    pub name: [u8; 64],
    pub ty: i32,
    pub bounding_radius: f32,
    mesh_count: i32,
    mesh_index: i32,
    vertex_count: i32,
    pub vertex_index: i32,
    tangent_index: i32,
    attachment_count: i32,
    attachment_index: i32,
    eyeball_count: i32,
    eyeball_index: i32,
    pub vertex_data: ModelVertexData,
    padding: [i32; 8],
}

static_assertions::const_assert_eq!(size_of::<ModelHeader>(), 148);

impl ModelHeader {
    pub fn mesh_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.mesh_index, self.mesh_count, size_of::<MeshHeader>())
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct ModelVertexData {
    // these are pointers?
    vertex_data: i32,
    tangent_data: i32,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct MeshHeader {
    pub material: i32,
    model_index: i32,
    vertex_count: i32,
    pub vertex_index: i32,
    flex_count: i32,
    flex_index: i32,
    material_type: i32,
    material_param: i32,
    mesh_id: i32,
    center: Vector,
    vertex_data: MeshVertexData,
    padding: [i32; 8],
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct MeshVertexData {
    // these are pointers?
    model_vertex_data: i32,
    lod_vertex_count: [i32; 8],
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct MeshTexture {
    pub name_index: i32, // relative offset to this struct
    pub flags: i32,
    pub used: i32,
    _padding: i32,
    pub material_ptr: i32,
    pub client_material_ptr: i32,
    _padding2: [i32; 10],
}

static_assertions::const_assert_eq!(size_of::<MeshTexture>(), 16 * 4);

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct StudioAttachmentHeader {
    pub name_index: i32,
    pub flags: AttachmentFlags,
    pub local_bone: i32,
    pub local: Transform3x4,
    pub padding: [i32; 8],
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct AttachmentFlags(i32);

bitflags! {
    impl AttachmentFlags: i32 {
        /// Vector48
        const ATTACHMENT_WORLD_ALIGN = 0x10000;
    }
}

static_assertions::const_assert_eq!(size_of::<StudioAttachmentHeader>(), 23 * 4);

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct HitBoxSetHeader {
    pub name_index: i32,
    pub hitbox_count: i32,
    pub hitbox_offset: i32,
}

impl HitBoxSetHeader {
    pub fn hitbox_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.hitbox_offset, self.hitbox_count, size_of::<BoundingBoxHeader>())
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
#[allow(dead_code)]
pub struct BoundingBoxHeader {
    pub bone: i32,
    pub group: i32,
    pub bounding_box_min: Vector,
    pub bounding_box_max: Vector,
    pub name_index: i32,
    padding: [i32; 8],
}