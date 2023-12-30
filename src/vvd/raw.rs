use crate::{index_range, ReadableRelative, Vector};
use bytemuck::{Pod, Zeroable};
use std::cmp::min;
use std::mem::size_of;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct VvdHeader {
    pub id: i32,
    pub version: i32,
    pub checksum: [u8; 4],
    pub lod_count: i32,
    lod_vertex_count: [i32; 8],
    fixup_count: i32,
    fixup_index: i32,
    vertex_index: i32,
    tangent_index: i32,
}

impl VvdHeader {
    pub fn fixup_indexes(&self) -> impl Iterator<Item = usize> {
        index_range(
            self.fixup_index,
            self.fixup_count,
            size_of::<VertexFileFixup>(),
        )
    }

    pub fn has_fixups(&self) -> bool {
        self.fixup_count > 0
    }

    pub fn vertex_indexes(&self, lod: i32) -> Option<impl Iterator<Item = usize>> {
        if lod < self.lod_count {
            Some(index_range(
                self.vertex_index,
                self.lod_vertex_count[lod as usize],
                size_of::<Vertex>(),
            ))
        } else {
            None
        }
    }

    pub fn tangent_indexes(&self, lod: i32) -> Option<impl Iterator<Item = usize>> {
        if lod < self.lod_count {
            Some(index_range(
                self.tangent_index,
                self.lod_vertex_count[lod as usize],
                size_of::<[f32; 4]>(),
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct VertexFileFixup {
    pub lod: i32,
    pub source_vertex_id: i32,
    pub vertex_count: i32,
}

impl ReadableRelative for VertexFileFixup {}

#[derive(Debug, Clone, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct Vertex {
    pub bone_weights: BoneWeights,
    pub position: Vector,
    pub normal: Vector,
    pub texture_coordinates: [f32; 2],
}

impl ReadableRelative for Vertex {}

static_assertions::const_assert_eq!(size_of::<Vertex>(), 48);

#[derive(Debug, Clone, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct BoneWeights {
    weight: [f32; 3],
    bone: [u8; 3],
    bone_count: u8,
}

impl BoneWeights {
    pub fn weights(&self) -> impl Iterator<Item = BoneWeight> + '_ {
        (0..min(self.bone_count as usize, 2)).map(|i| BoneWeight {
            weight: self.weight[i],
            bone_id: self.bone[i],
        })
    }
}

pub struct BoneWeight {
    pub bone_id: u8,
    pub weight: f32,
}

static_assertions::const_assert_eq!(size_of::<BoneWeights>(), 16);

#[derive(Debug, Clone, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct Tangent {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
