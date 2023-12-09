use crate::{index_range, FixedString};
use crate::{Quaternion, RadianEuler, Vector};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

pub mod header;
pub mod header2;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Bone {
    pub sz_name_index: i32,
    pub parent: i32,               // parent bone
    pub bone_controller: [i32; 6], // bone controller index, -1 == none

    pub pos: Vector,
    pub quaternion: Quaternion,
    pub rot: RadianEuler,
    pub pos_scale: Vector,
    pub rot_scale: Vector,

    pub pose_to_bone: [[f32; 3]; 4], // 3x4 matrix
    pub q_alignment: Quaternion,
    pub flags: BoneFlags,
    pub proc_type: i32,
    pub proc_index: i32,       // procedural rule
    pub physics_bone: i32,     // index into physically simulated bone
    pub surface_prop_idx: i32, // index into string table for property name
    pub contents: i32,         // See BSPFlags.h for the contents flags

    #[allow(dead_code)]
    reserved: [i32; 8], // remove as appropriate
}

bitflags! {
    #[derive(Zeroable, Pod)]
    #[repr(C)]
    pub struct BoneFlags: u32 {
        const BONE_PHYSICALLY_SIMULATED = 	0x00000001;
        const BONE_PHYSICS_PROCEDURAL = 	0x00000002;
        const BONE_ALWAYS_PROCEDURAL = 		0x00000004;
        const BONE_SCREEN_ALIGN_SPHERE = 	0x00000008;
        const BONE_SCREEN_ALIGN_CYLINDER = 	0x00000010;

        const BONE_USED_BY_HITBOX =			0x00000100;
        const BONE_USED_BY_ATTACHMENT =		0x00000200;

        const BONE_USED_BY_VERTEX_LOD0 =	0x00000400;
        const BONE_USED_BY_VERTEX_LOD1 =	0x00000800;
        const BONE_USED_BY_VERTEX_LOD2 =	0x00001000;
        const BONE_USED_BY_VERTEX_LOD3 =	0x00002000;
        const BONE_USED_BY_VERTEX_LOD4 =	0x00004000;
        const BONE_USED_BY_VERTEX_LOD5 =	0x00008000;
        const BONE_USED_BY_VERTEX_LOD6 =	0x00010000;
        const BONE_USED_BY_VERTEX_LOD7 =	0x00020000;
        const BONE_USED_BY_BONE_MERGE =		0x00040000;

        const BONE_TYPE_MASK =				0x00F00000;
        const BONE_FIXED_ALIGNMENT =		0x00100000;

        const BONE_HAS_SAVEFRAME_POS =		0x00200000;
        const BONE_HAS_SAVEFRAME_ROT =		0x00400000;
    }
}

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
    material: i32,
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
