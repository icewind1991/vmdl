use crate::{Quaternion, RadianEuler, Vector};
use binrw::BinRead;
use bitflags::bitflags;

#[derive(Debug, Clone, BinRead)]
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
    #[derive(BinRead)]
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
