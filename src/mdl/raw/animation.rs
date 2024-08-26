use crate::vvd::Vertex;
use crate::{ModelError, ReadRelative, ReadableRelative};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct PoseParameterDescriptionHeader {
    name_index: i32,
    flags: i32,
    start: f32,
    end: f32,
    loop_range: f32,
}

static_assertions::const_assert_eq!(size_of::<PoseParameterDescriptionHeader>(), 20);

#[derive(Clone, Debug)]
pub struct PoseParameterDescription {
    pub name: String,
    pub flags: i32,
    pub start: f32,
    pub end: f32,
    pub loop_range: f32,
}

impl ReadRelative for PoseParameterDescription {
    type Header = PoseParameterDescriptionHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self, ModelError> {
        let name_bytes = data
            .get(header.name_index as usize..)
            .ok_or(ModelError::OutOfBounds {
                data: "pose name",
                offset: header.name_index as usize,
            })?;
        Ok(PoseParameterDescription {
            name: String::read(name_bytes, ())?,
            flags: header.flags,
            start: header.start,
            end: header.end,
            loop_range: header.loop_range,
        })
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AnimationDescriptionHeader {
    base_ptr: i32,
    name_offset: i32,
    fps: f32,
    flags: i32,

    frame_count: i32,

    movement_count: i32,
    movement_offset: i32,

    _padding: [i32; 6],

    animation_block: i32,
    animation_index: i32, // non-zero when anim data isn't in sections

    ik_rule_count: i32,
    ik_rule_offset: i32,
    animation_block_ik_rule_index: i32,

    local_hierarchy_count: i32,
    local_hierarchy_offset: i32,

    section_offset: i32,
    section_frames: i32,

    zero_frame_span: i16,
    zero_frame_count: i16,
    zero_frame_offset: i32,

    zero_frame_stall_time: f32,
}

static_assertions::const_assert_eq!(size_of::<AnimationDescriptionHeader>(), 100);

#[derive(Clone, Debug)]
pub struct AnimationDescription {
    pub name: String,
    pub fps: f32,
    pub frame_count: i32,
    pub animation_block: i32,
    pub animation: i32,
}

impl ReadRelative for AnimationDescription {
    type Header = AnimationDescriptionHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self, ModelError> {
        let name_bytes =
            data.get(header.name_offset as usize..)
                .ok_or(ModelError::OutOfBounds {
                    data: "animation name",
                    offset: header.name_offset as usize,
                })?;
        Ok(AnimationDescription {
            name: String::read(name_bytes, ())?,
            fps: header.fps,
            frame_count: header.frame_count,
            animation_block: header.animation_block,
            animation: header.animation_index,
        })
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AnimationBlock {
    start: i32,
    end: i32,
}

impl ReadableRelative for AnimationBlock {}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct AnimationHeader {
    bone: u8,
    flags: AnimationFlags,
    next_offset: u16,
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct AnimationFlags(u8);

bitflags! {
    impl AnimationFlags: u8 {
        /// Vector48
        const STUDIO_ANIM_RAWPOS = 	0x00000001;
        /// Quaternion48
        const STUDIO_ANIM_RAWROT = 	0x00000002;
        /// mstudioanim_valueptr_t
        const STUDIO_ANIM_ANIMPOS = 0x00000004;
        /// mstudioanim_valueptr_t
        const STUDIO_ANIM_ANIMROT = 0x00000008;
        const STUDIO_ANIM_DELTA = 	0x00000010;
        /// Quaternion64
        const STUDIO_ANIM_RAWROT2 = 0x00000020;
    }
}
