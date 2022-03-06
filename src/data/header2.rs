use std::ops::Range;

pub struct StudioHHeader2 {
    source_bone_transform_count: i32,
    source_bone_transform_index: i32,

    pub illumination_position_attachment_index: i32,

    fl_max_exe_deflection: f32,

    pub linear_bone_index: i32,

    pub sz_name_index: i32,

    bone_flex_driver_count: i32,
    bone_flex_driver_index: i32,

    #[allow(dead_code)]
    reserved: [i32; 56],
}

impl StudioHHeader2 {
    pub fn source_bone_transforms(&self) -> Range<i32> {
        self.source_bone_transform_index
            ..(self.source_bone_transform_index + self.source_bone_transform_count)
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
