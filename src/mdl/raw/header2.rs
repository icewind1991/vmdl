use crate::mdl::SourceBoneTransformHeader;
use crate::{index_range, ReadableRelative};
use bytemuck::{Pod, Zeroable};
use std::mem::size_of;
use std::ops::Range;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct StudioHeader2 {
    source_bone_transform_count: i32,
    source_bone_transform_index: i32,

    pub illumination_position_attachment_index: i32,

    fl_max_exe_deflection: f32,

    pub linear_bone_index: i32,

    pub sz_name_index: i32,

    bone_flex_driver_count: i32,
    bone_flex_driver_index: i32,

    _reserved1: [i32; 32],
    _reserved2: [i32; 24],
}

impl ReadableRelative for StudioHeader2 {}

impl StudioHeader2 {
    pub fn source_bone_transforms(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.source_bone_transform_index,
            self.source_bone_transform_count,
            size_of::<SourceBoneTransformHeader>(),
        )
    }

    pub fn bone_flex_drivers(&self) -> Range<i32> {
        self.bone_flex_driver_index..(self.bone_flex_driver_index + self.bone_flex_driver_count)
    }

    pub fn max_eye_deflection(&self) -> f32 {
        if self.fl_max_exe_deflection == 0.0 {
            (30.0f32).cos()
        } else {
            self.fl_max_exe_deflection
        }
    }
}
