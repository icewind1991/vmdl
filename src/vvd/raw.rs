use crate::{index_range, Vector};
use binrw::{BinRead, BinResult, ReadOptions};
use bytemuck::{cast, Pod, Zeroable};
use std::io::{Read, Seek};
use std::mem::size_of;

#[derive(Debug, Clone, BinRead)]
pub struct VvdHeader {
    pub id: i32,
    pub version: i32,
    pub checksum: [u8; 4],
    pub lod_count: i32,
    lod_vertex_count: [i32; 8],
    fixup_count: i32,
    fixup_index: i32,
    vertex_index: i32,
    #[allow(dead_code)]
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
}

#[derive(Debug, Clone, BinRead, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct VertexFileFixup {
    pub lod: i32,
    pub source_vertex_id: i32,
    pub vertex_count: i32,
}

#[derive(Debug, Clone, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct Vertex {
    pub bone_weights: BoneWeight,
    pub position: Vector,
    pub normal: Vector,
    pub texture_coordinates: [f32; 2],
}

static_assertions::const_assert_eq!(size_of::<Vertex>(), 48);
// binread_for_pod!(Vertex);

impl BinRead for Vertex {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _options: &ReadOptions,
        _args: Self::Args,
    ) -> BinResult<Self> {
        let mut bytes = unsafe {
            std::mem::MaybeUninit::<[u8; std::mem::size_of::<Self>()]>::uninit().assume_init()
        };

        reader.read(&mut bytes)?;
        Ok(cast(bytes))
    }
}

#[derive(Debug, Clone, BinRead, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct BoneWeight {
    pub weight: [f32; 3],
    pub bone: [u8; 3],
    pub bone_count: u8,
}

static_assertions::const_assert_eq!(size_of::<BoneWeight>(), 16);

#[derive(Debug, Clone, BinRead, Zeroable, Pod, Copy)]
#[repr(C)]
pub struct Tangent {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
