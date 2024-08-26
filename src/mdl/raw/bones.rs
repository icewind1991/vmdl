use crate::{
    read_single, ModelError, Quaternion, RadianEuler, ReadRelative, Readable, Transform3x4, Vector,
};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use num_enum::TryFromPrimitive;
use std::mem::size_of;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct BoneHeader {
    pub sz_name_index: i32,
    pub parent: i32,               // parent bone
    pub bone_controller: [i32; 6], // bone controller index, -1 == none

    pub pos: Vector,
    pub quaternion: Quaternion,
    pub rot: RadianEuler,
    pub pos_scale: Vector,
    pub rot_scale: Vector,

    pub pose_to_bone: Transform3x4,
    pub q_alignment: Quaternion,
    pub flags: BoneFlags,
    pub proc_type: i32,
    pub proc_index: i32,       // procedural rule
    pub physics_bone: i32,     // index into physically simulated bone
    pub surface_prop_idx: i32, // index into string table for property name
    pub contents: ContentFlags,

    #[allow(dead_code)]
    reserved: [i32; 8], // remove as appropriate
}

static_assertions::const_assert_eq!(size_of::<BoneHeader>(), 216);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Bone {
    pub name: String,
    pub parent: i32,               // parent bone
    pub bone_controller: [i32; 6], // bone controller index, -1 == none

    pub pos: Vector,
    pub quaternion: Quaternion,
    pub rot: RadianEuler,
    pub pos_scale: Vector,
    pub rot_scale: Vector,

    pub pose_to_bone: Transform3x4,
    pub q_alignment: Quaternion,
    pub flags: BoneFlags,
    pub procedural_rules: Option<ProceduralBone>,
    pub physics_bone: i32, // index into physically simulated bone
    pub surface_prop: String,
    pub contents: ContentFlags,
}

impl ReadRelative for Bone {
    type Header = BoneHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self, ModelError> {
        let prop_type = ProceduralBoneType::try_from(header.proc_type).ok();
        let proc_bytes = (header.proc_index != 0)
            .then(|| {
                data.get(header.proc_index as usize..)
                    .ok_or(ModelError::OutOfBounds {
                        data: "bone surface property",
                        offset: header.proc_index as usize,
                    })
            })
            .transpose()?;
        let procedural_rules = prop_type
            .zip(proc_bytes)
            .map(|(ty, bytes)| {
                Result::<_, ModelError>::Ok(match ty {
                    ProceduralBoneType::AxisInterp => {
                        ProceduralBone::AxisInterp(AxisInterpBone::read(bytes)?)
                    }
                    ProceduralBoneType::QuaternionInterp => {
                        ProceduralBone::QuaternionInterp(QuaternionInterpBone::read(bytes)?)
                    }
                    ProceduralBoneType::AiMatBone => {
                        ProceduralBone::AiMatBone(AiMatBone::read(bytes)?)
                    }
                    ProceduralBoneType::AiMatAttach => {
                        ProceduralBone::AiMatAttach(AiMatBone::read(bytes)?)
                    }
                    ProceduralBoneType::Jiggle => ProceduralBone::Jiggle(JiggleBone::read(bytes)?),
                })
            })
            .transpose()?;

        Ok(Bone {
            name: read_single(data, header.sz_name_index)?,
            parent: header.parent,
            bone_controller: header.bone_controller,
            pos: header.pos,
            quaternion: header.quaternion,
            rot: header.rot,
            pos_scale: header.pos_scale,
            rot_scale: header.rot_scale,
            pose_to_bone: header.pose_to_bone,
            q_alignment: header.q_alignment,
            flags: header.flags,
            procedural_rules,
            physics_bone: header.physics_bone,
            surface_prop: read_single(data, header.surface_prop_idx)?,
            contents: header.contents,
        })
    }
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct BoneFlags(u32);

bitflags! {
    impl BoneFlags: u32 {
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

#[derive(Debug, Clone)]
pub enum ProceduralBone {
    AxisInterp(AxisInterpBone),
    QuaternionInterp(QuaternionInterpBone),
    AiMatBone(AiMatBone),
    AiMatAttach(AiMatBone),
    Jiggle(JiggleBone),
}

#[derive(TryFromPrimitive, Copy, Clone)]
#[repr(i32)]
pub enum ProceduralBoneType {
    AxisInterp = 1,
    QuaternionInterp = 2,
    AiMatBone = 3,
    AiMatAttach = 4,
    Jiggle = 5,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AxisInterpBone {
    pub control: i32,
    pub axis: i32,
    pub position: [Vector; 6],       // X+, X-, Y+, Y-, Z+, Z-
    pub quaternion: [Quaternion; 6], // X+, X-, Y+, Y-, Z+, Z-
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct QuaternionInterpBone {
    /// 1 / radian angle of trigger influence
    pub inverse_tolerance: f32,
    /// angle to match
    pub trigger: Quaternion,
    /// new position
    pub position: Vector,
    /// new angle
    pub quaternion: Quaternion,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AiMatBone {
    pub parent: i32,
    pub aim: i32,
    pub aim_vector: Vector,
    pub up_vector: Vector,
    pub base_position: Vector,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct JiggleBone {
    pub flags: JiggleBoneFlags,
    pub length: f32,
    pub tip_mass: f32,

    pub yaw_stiffness: f32,
    pub yaw_damping: f32,
    pub pitch_stiffness: f32,
    pub pitch_damping: f32,
    pub along_stiffness: f32,
    pub along_damping: f32,

    pub angle_limit: f32,

    pub min_yaw: f32,
    pub max_yaw: f32,
    pub yaw_friction: f32,
    pub yaw_bound: f32,

    pub min_pitch: f32,
    pub max_pitch: f32,
    pub pitch_friction: f32,
    pub pitch_bounce: f32,

    pub base_mass: f32,
    pub base_stiffness: f32,
    pub base_damping: f32,
    pub base_min_left: f32,
    pub base_max_left: f32,
    pub base_left_friction: f32,
    pub base_min_up: f32,
    pub base_max_up: f32,
    pub base_up_friction: f32,
    pub base_min_forward: f32,
    pub base_max_forward: f32,
    pub base_forward_friction: f32,

    pub boing_impact_speed: f32,
    pub boing_impact_angle: f32,
    pub boing_damping_rate: f32,
    pub boing_frequency: f32,
    pub boing_amplitute: f32,
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct JiggleBoneFlags(u32);

bitflags! {
    impl JiggleBoneFlags: u32 {
        const JIGGLE_IS_FLEXIBLE = 	         0x01;
        const JIGGLE_IS_RIGID =              0x02;
        const JIGGLE_HAS_YAW_CONSTRAINT = 	 0x04;
        const JIGGLE_HAS_PITCH_CONSTRAINT =  0x08;
        const JIGGLE_HAS_ANGLE_CONSTRAINT =  0x10;
        const JIGGLE_HAS_LENGTH_CONSTRAINT = 0x20;
        const JIGGLE_HAS_BASE_SPRING =       0x40;
        /// simple squash and stretch sinusoid "boing"
        const JIGGLE_IS_BOING =              0x80;
    }
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct ContentFlags(u32);

bitflags! {
    impl ContentFlags: u32 {
        const CONTENTS_SOLID = 	              0x01;
        const CONTENTS_WINDOW =               0x02;
        const CONTENTS_AUX = 	              0x04;
        const CONTENTS_GRATE =                0x08;
        const CONTENTS_SLIME =                0x10;
        const CONTENTS_WATER =                0x20;
        const CONTENTS_BLOCKLOS =             0x40;
        const CONTENTS_OPAQUE =               0x80;

        const CONTENTS_TESTFOGVOLUME =        0x100;

        const CONTENTS_TEAM1 =                0x800;
        const CONTENTS_TEAM2 =                0x1000;

        const CONTENTS_IGNORE_NODRAW_OPAQUE = 0x2000;

        const CONTENTS_MOVABLE =              0x4000;
        const CONTENTS_AREAPORTAL =           0x8000;

        const CONTENTS_PLAYERCLIP =           0x10000;
        const CONTENTS_MONSTERCLIP =          0x20000;

        const CONTENTS_CURRENT_0 =            0x40000;
        const CONTENTS_CURRENT_90 =           0x80000;
        const CONTENTS_CURRENT_180 =          0x100000;
        const CONTENTS_CURRENT_270 =          0x200000;
        const CONTENTS_CURRENT_UP =           0x400000;
        const CONTENTS_CURRENT_DOWN =         0x800000;

        const CONTENTS_ORIGIN =               0x1000000;

        const CONTENTS_MONSTER =              0x2000000;
        const CONTENTS_DEBRIS =               0x4000000;
        const CONTENTS_DETAIL =               0x8000000;
        const CONTENTS_TRANSLUCENT =          0x10000000;
        const CONTENTS_LADDER =               0x20000000;
        const CONTENTS_HITBOX =               0x40000000;
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SourceBoneTransformHeader {
    pub sz_name_index: i32,

    pub pre_transform: Transform3x4,
    pub post_transform: Transform3x4,
}
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SourceBoneTransform {
    pub name: String,
    pub pre_transform: Transform3x4,
    pub post_transform: Transform3x4,
}

impl ReadRelative for SourceBoneTransform {
    type Header = SourceBoneTransformHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self, ModelError> {
        Ok(SourceBoneTransform {
            name: read_single(data, header.sz_name_index)?,
            pre_transform: header.pre_transform,
            post_transform: header.post_transform,
        })
    }
}
