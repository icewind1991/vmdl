use crate::index_range;
use binrw::BinRead;
use bitflags::bitflags;
use std::mem::size_of;
use std::ops::Range;

#[derive(Debug, Clone, BinRead)]
pub struct VtxHeader {
    pub version: i32,
    pub vertex_cache_size: i32,
    pub max_bones_per_strip: u16,
    pub max_bones_per_triangle: u16,
    pub max_bones_per_vertex: i32,
    pub checksum: [u8; 4],
    pub lod_count: i32,
    pub material_replacement_list: i32,
    body_part_count: i32,
    body_part_offset: i32,
}

static_assertions::const_assert_eq!(size_of::<VtxHeader>(), 36);

impl VtxHeader {
    pub fn body_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.body_part_offset,
            self.body_part_count,
            size_of::<BodyPartHeader>(),
        )
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct BodyPartHeader {
    model_count: i32,
    model_offset: i32,
}

static_assertions::const_assert_eq!(size_of::<BodyPartHeader>(), 8);

impl BodyPartHeader {
    pub fn model_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.model_offset,
            self.model_count,
            size_of::<ModelHeader>(),
        )
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct ModelHeader {
    lod_count: i32,
    lod_offset: i32,
}

static_assertions::const_assert_eq!(size_of::<ModelHeader>(), 8);

impl ModelHeader {
    pub fn lod_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.lod_offset, self.lod_count, size_of::<ModelLodHeader>())
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct ModelLodHeader {
    mesh_count: i32,
    mesh_offset: i32,
    pub switch_point: f32,
}

static_assertions::const_assert_eq!(size_of::<ModelLodHeader>(), 12);

impl ModelLodHeader {
    pub fn mesh_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.mesh_offset, self.mesh_count, size_of::<MeshHeader>())
    }
}

#[derive(Debug, Clone, Copy, BinRead)]
#[repr(packed)]
pub struct MeshHeader {
    strip_group_count: i32,
    strip_group_offset: i32,
    pub flags: MeshFlags,
}

static_assertions::const_assert_eq!(size_of::<MeshHeader>(), 9);

impl MeshHeader {
    pub fn strip_group_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.strip_group_offset,
            self.strip_group_count,
            size_of::<StripGroupHeader>(),
        )
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct MeshFlags: u8 {
        const IS_TEETH = 0x01;
        const IS_EYES =  0x02;
    }
}

#[derive(Debug, Clone, Copy, BinRead)]
#[repr(packed)]
pub struct StripGroupHeader {
    vertex_count: i32,
    vertex_offset: i32,
    index_count: i32,
    index_offset: i32,
    strip_count: i32,
    strip_offset: i32,
    pub flags: StripGroupFlags,
}

static_assertions::const_assert_eq!(size_of::<StripGroupHeader>(), 25);

impl StripGroupHeader {
    /// Index into the VVD file vertexes
    pub fn vertex_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.vertex_offset,
            self.vertex_count,
            size_of::<Vertex>(), // Vertex index from .VVD's vertex array
        )
    }

    pub fn index_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(self.index_offset, self.index_count, size_of::<u16>())
    }

    pub fn strip_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.strip_offset,
            self.strip_count,
            size_of::<StripHeader>(),
        )
    }
}

bitflags! {
    #[derive(BinRead)]
    pub struct StripGroupFlags: u8 {
        const IS_FLEXED =         0x01;
        const IS_HWSKINNED =      0x02;
        const IS_DELTA_FLEXED =   0x04;
        const SUPPRESS_HW_MORPH = 0x08;
    }
}

#[derive(Debug, Clone, Copy, BinRead)]
#[repr(packed)]
pub struct StripHeader {
    index_count: i32,
    index_offset: i32,
    vertex_count: i32,
    vertex_offset: i32,
    pub bone_count: u16,
    pub flags: StripFlags,
    bone_state_change_count: i32,
    bone_state_change_offset: i32,
}

static_assertions::const_assert_eq!(size_of::<StripHeader>(), 27);

bitflags! {
    #[derive(BinRead)]
    pub struct StripFlags: u8 {
        const IS_TRI_LIST =  0x01;
        const IS_TRI_STRIP = 0x02;
    }
}

impl StripHeader {
    /// Index into the VVD file vertexes
    pub fn vertex_indexes(&self) -> Range<usize> {
        self.vertex_offset as usize..(self.vertex_offset + self.vertex_count) as usize
    }

    pub fn index_indexes(&self) -> Range<usize> {
        self.index_offset as usize..(self.index_offset + self.index_count) as usize
    }

    #[allow(dead_code)]
    pub fn bone_state_change_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.bone_state_change_offset,
            self.bone_state_change_count,
            1,
        )
    }
}

#[derive(Debug, Clone, Copy, BinRead)]
#[repr(packed)]
pub struct Vertex {
    pub bone_weight_indexes: [u8; 3],
    pub bone_count: u8,
    pub original_mesh_vertex_id: u16,
    pub bone_id: [u8; 3],
}

static_assertions::const_assert_eq!(size_of::<Vertex>(), 9);
