use crate::{Bone, FixedString, Vector};
use binrw::BinRead;
use std::mem::size_of;

pub const FILETYPE_ID: i32 = i32::from_be_bytes(*b"IDST");
pub const MDL_VERSION: i32 = 48;

#[derive(Debug, Clone, BinRead)]
pub struct StudioHeader {
    pub id: i32,
    pub version: i32,
    pub checksum: [u8; 4], // This has to be the same in the phy and vtx files to load!
    pub name: FixedString<64>,
    pub data_length: i32,

    pub eye_position: Vector, // Position of player viewpoint relative to model origin
    pub illumination_position: Vector, // Position (relative to model origin) used to calculate ambient light contribution and cubemap reflections for the entire model.
    pub hull_min: Vector,              // Corner of model hull box with the least X/Y/Z values
    pub hull_max: Vector,              // Opposite corner of model hull box
    pub view_bb_min: Vector,
    pub view_bb_max: Vector,

    pub flags: i32, // Binary flags in little-endian order.
    // ex (00000001,00000000,00000000,11000000) means flags for position 0, 30, and 31 are set.
    // Set model flags section for more information

    /*
     * After this point, the header contains many references to offsets
     * within the MDL file and the number of items at those offsets.
     *
     * Offsets are from the very beginning of the file.
     *
     * Note that indexes/counts are not always paired and ordered consistently.
     */
    // mstudiobone_t
    bone_count: i32,  // Number of data sections (of type mstudiobone_t)
    bone_offset: i32, // Offset of first data section

    // mstudiobonecontroller_t
    bone_controller_count: i32,
    bone_controller_offset: i32,

    // mstudiohitboxset_t
    hitbox_count: i32,
    hitbox_offset: i32,

    // mstudioanimdesc_t
    local_animation_count: i32,
    local_animation_offset: i32,

    // mstudioseqdesc_t
    local_seq_count: i32,
    local_seq_offset: i32,

    pub activity_list_version: i32, // ??
    pub events_indexed: i32,        // ??

    // VMT texture filenames
    // mstudiotexture_t
    texture_count: i32,
    texture_offset: i32,

    // This offset points to a series of ints.
    // Each int value, in turn, is an offset relative to the start of this header/the-file,
    // At which there is a null-terminated string.
    texture_dir_count: i32,
    texture_dir_offset: i32,

    // Each skin-family assigns a texture-id to a skin location
    pub skin_reference_count: i32,
    pub skin_r_family_count: i32,
    pub skin_reference_index: i32,

    // mstudiobodyparts_t
    body_part_count: i32,
    body_part_offset: i32,

    // Local attachment points
    // mstudioattachment_t
    attachment_count: i32,
    attachment_offset: i32,

    // Node values appear to be single bytes, while their names are null-terminated strings.
    local_node_count: i32,
    local_node_index: i32,
    local_node_name_index: i32,

    // mstudioflexdesc_t
    flex_desc_count: i32,
    flex_desc_index: i32,

    // mstudioflexcontroller_t
    flex_controller_count: i32,
    flex_controller_index: i32,

    // mstudioflexrule_t
    flex_rules_count: i32,
    flex_rules_index: i32,

    // IK probably referse to inverse kinematics
    // mstudioikchain_t
    ik_chain_count: i32,
    ik_chain_index: i32,

    // Information about any "mouth" on the model for speech animation
    // More than one sounds pretty creepy.
    // mstudiomouth_t
    mouths_count: i32,
    mouths_index: i32,

    // mstudioposeparamdesc_t
    local_pose_param_count: i32,
    local_pose_param_index: i32,

    /*
     * For anyone trying to follow along, as of this writing,
     * the next "surfaceprop_index" value is at position 0x0134 (308)
     * from the start of the file.
     */
    // Surface property value (single null-terminated string)
    pub surface_prop_index: i32,

    // Unusual: In this one index comes first, then count.
    // Key-value data is a series of strings. If you can't find
    // what you're interested in, check the associated PHY file as well.
    key_value_index: i32,
    key_value_count: i32,

    // More inverse-kinematics
    // mstudioiklock_t
    ik_lock_count: i32,
    ik_lock_index: i32,

    pub mass: f32,     // Mass of object (4-bytes)
    pub contents: i32, // ??

    // Other models can be referenced for re-used sequences and animations
    // (See also: The $includemodel QC option.)
    // mstudiomodelgroup_t
    include_model_count: i32,
    include_model_index: i32,

    pub virtual_model: i32, // Placeholder for mutable-void*
    // Note that the SDK only compiles as 32-bit, so an int and a pointer are the same size (4 bytes)

    // mstudioanimblock_t
    anim_blocks_name_index: i32,
    anim_blocks_count: i32,
    anim_blocks_index: i32,

    pub anim_block_model: i32, // Placeholder for mutable-void*

    // Points to a series of bytes?
    pub bone_table_name_index: i32,

    pub vertex_base: i32, // Placeholder for void*
    pub offset_base: i32, // Placeholder for void*

    // Used with $constantdirectionallight from the QC
    // Model should have flag #13 set if enabled
    pub directional_dot_product: u8,

    pub root_lod: u8, // Preferred rather than clamped

    // 0 means any allowed, N means Lod 0 -> (N-1)
    pub num_allowed_root_lods: u8,

    #[allow(dead_code)]
    unused0: u8, // ??
    #[allow(dead_code)]
    unused1: i32, // ??

    pub flex_controller_ui_count: i32,
    pub flex_controller_ui_index: i32,

    pub vert_anim_fixed_point_scale: f32,
    pub unused2: i32,

    pub studio_hdr2_index: i32,

    #[allow(dead_code)]
    unused3: i32,
}

impl StudioHeader {
    pub fn header2_index(&self) -> Option<usize> {
        (self.studio_hdr2_index > 0)
            .then(|| self.studio_hdr2_index)
            .and_then(|index| usize::try_from(index).ok())
    }

    pub fn bone_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.bone_offset, self.bone_count, size_of::<Bone>())
    }

    pub fn bone_controller_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.bone_controller_offset, self.bone_controller_count, 1)
    }

    pub fn hitbox_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.hitbox_offset, self.hitbox_count, 1)
    }

    pub fn local_animation_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.local_animation_offset, self.local_animation_count, 1)
    }

    pub fn local_sequence_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.local_seq_offset, self.local_seq_count, 1)
    }

    pub fn texture_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.texture_offset, self.texture_count, 1)
    }

    pub fn texture_dir_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.texture_dir_offset, self.texture_dir_count, 1)
    }

    pub fn body_part_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.body_part_offset, self.body_part_count, 1)
    }

    pub fn attachment_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.attachment_offset, self.attachment_count, 1)
    }

    pub fn local_node_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.local_node_index, self.local_node_count, 1)
    }

    pub fn local_node_name_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.local_node_name_index, self.local_node_count, 1)
    }

    pub fn flex_descriptor_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.flex_desc_index, self.flex_desc_count, 1)
    }

    pub fn flex_controller_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.flex_controller_index, self.flex_controller_count, 1)
    }

    pub fn flex_rule_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.flex_rules_index, self.flex_rules_count, 1)
    }

    pub fn ik_chain_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.ik_chain_index, self.ik_chain_count, 1)
    }

    pub fn mouth_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.mouths_index, self.mouths_count, 1)
    }

    pub fn local_pose_param_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.local_pose_param_index, self.local_pose_param_count, 1)
    }

    pub fn key_value_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.key_value_index, self.key_value_count, 1)
    }

    pub fn ik_lock_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.ik_lock_index, self.ik_lock_count, 1)
    }

    pub fn include_model_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.include_model_index, self.include_model_count, 1)
    }

    pub fn animation_block_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.anim_blocks_index, self.anim_blocks_count, 1)
    }

    pub fn animation_block_name_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.anim_blocks_name_index, self.anim_blocks_count, 1)
    }

    pub fn flex_controller_ui_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.flex_controller_ui_index,
            self.flex_controller_ui_count,
            1,
        )
    }
}

fn index_range(index: i32, count: i32, size: usize) -> impl Iterator<Item = usize> {
    (0..count as usize)
        .map(move |i| i * size)
        .map(move |i| index as usize + i)
}

static_assertions::const_assert_eq!(size_of::<StudioHeader>() - size_of::<FixedString<0>>(), 408);
