mod raw;

use crate::{read_relative, ModelError, ReadRelative};
use binrw::BinReaderExt;
use itertools::Either;
use raw::*;
pub use raw::{MeshFlags, StripFlags, StripGroupFlags, Vertex};
use std::io::Cursor;
use std::ops::Range;

pub const MDL_VERSION: i32 = 7;

type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct Vtx {
    pub header: VtxHeader,
    pub body_parts: Vec<BodyPart>,
}

impl Vtx {
    pub fn read(data: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let header: VtxHeader = reader.read_le()?;
        Ok(Vtx {
            body_parts: read_relative(data, header.body_indexes())?,
            header,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BodyPart {
    pub models: Vec<Model>,
}

impl ReadRelative for BodyPart {
    type Header = BodyPartHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(BodyPart {
            models: read_relative(data, header.model_indexes())?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub lods: Vec<ModelLod>,
}

impl ReadRelative for Model {
    type Header = ModelHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(Model {
            lods: read_relative(data, header.lod_indexes())?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ModelLod {
    pub meshes: Vec<Mesh>,
    pub switch_point: f32,
}

impl ReadRelative for ModelLod {
    type Header = ModelLodHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(ModelLod {
            meshes: read_relative(data, header.mesh_indexes())?,
            switch_point: header.switch_point,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub strip_groups: Vec<StripGroup>,
    pub flags: MeshFlags,
}

impl ReadRelative for Mesh {
    type Header = MeshHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(Mesh {
            strip_groups: read_relative(data, header.strip_group_indexes())?,
            flags: header.flags,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StripGroup {
    // todo topologies
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
    pub strips: Vec<Strip>,
    pub flags: StripGroupFlags,
}

impl ReadRelative for StripGroup {
    type Header = StripGroupHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(StripGroup {
            vertices: read_relative(data, header.vertex_indexes())?,
            strips: read_relative(data, header.strip_indexes())?,
            indices: read_relative(data, header.index_indexes())?,
            flags: header.flags,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Strip {
    // todo bone state changes
    vertices: Range<usize>,
    pub flags: StripFlags,
    indices: Range<usize>,
}

impl ReadRelative for Strip {
    type Header = StripHeader;

    fn read(_data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(Strip {
            vertices: header.vertex_indexes(),
            indices: header.index_indexes(),
            flags: header.flags,
        })
    }
}

impl Strip {
    pub fn vertices(&self) -> impl Iterator<Item = usize> + 'static {
        self.vertices.clone()
    }

    pub fn indices(&self) -> impl Iterator<Item = usize> + 'static {
        if self.flags.contains(StripFlags::IS_TRI_STRIP) {
            let offset = self.indices.start;
            Either::Left((0..self.indices.len()).flat_map(move |i| {
                let cw = i & 1;
                let idx = offset + i;
                [idx, idx + 1 - cw, idx + 2 - cw].into_iter()
            }))
        } else {
            Either::Right(self.indices.clone())
        }
    }
}
