mod raw;

use crate::ModelError;
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
            body_parts: header
                .body_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "BodyPart",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    let header = reader.read_le()?;
                    BodyPart::read(data, header)
                })
                .collect::<Result<_>>()?,
            header,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BodyPart {
    pub models: Vec<Model>,
}

impl BodyPart {
    fn read(data: &[u8], header: BodyPartHeader) -> Result<Self> {
        Ok(BodyPart {
            models: header
                .model_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "Model",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    let header = reader.read_le()?;
                    Model::read(data, header)
                })
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub lods: Vec<ModelLod>,
}

impl Model {
    fn read(data: &[u8], header: ModelHeader) -> Result<Self> {
        Ok(Model {
            lods: header
                .lod_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "ModelLod",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    let header = reader.read_le()?;
                    ModelLod::read(data, header)
                })
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ModelLod {
    pub meshes: Vec<Mesh>,
    pub switch_point: f32,
}

impl ModelLod {
    fn read(data: &[u8], header: ModelLodHeader) -> Result<Self> {
        Ok(ModelLod {
            meshes: header
                .mesh_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "Mesh",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    let header = reader.read_le()?;
                    Mesh::read(data, header)
                })
                .collect::<Result<_>>()?,
            switch_point: header.switch_point,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub strip_groups: Vec<StripGroup>,
    pub flags: MeshFlags,
}

impl Mesh {
    fn read(data: &[u8], header: MeshHeader) -> Result<Self> {
        Ok(Mesh {
            strip_groups: header
                .strip_group_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "StripGroup",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    let header = reader.read_le()?;
                    StripGroup::read(data, header)
                })
                .collect::<Result<_>>()?,
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

impl StripGroup {
    fn read(data: &[u8], header: StripGroupHeader) -> Result<Self> {
        Ok(StripGroup {
            vertices: header
                .vertex_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "Vertex",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    reader.read_le().map_err(ModelError::from)
                })
                .collect::<Result<_>>()?,
            strips: header
                .strip_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "Strip",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    let header = reader.read_le()?;
                    Strip::read(data, header)
                })
                .collect::<Result<_>>()?,
            indices: header
                .index_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "VertexIndex",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    Ok(reader.read_le()?)
                })
                .collect::<Result<_>>()?,
            flags: dbg!(header).flags,
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

impl Strip {
    fn read(_data: &[u8], header: StripHeader) -> Result<Self> {
        Ok(Strip {
            vertices: header.vertex_indexes(),
            indices: header.index_indexes(),
            flags: dbg!(header).flags,
        })
    }

    pub fn vertices(&self) -> impl Iterator<Item = usize> + 'static {
        self.vertices.clone()
    }

    pub fn indices(&self) -> impl Iterator<Item = [usize; 3]> + 'static {
        if self.flags.contains(StripFlags::IS_TRI_STRIP) {
            let offset = self.indices.start;
            Either::Left((0..self.indices.len()).map(move |i| {
                let cw = i & 1;
                let idx = offset + i;
                [idx, idx + 1 - cw, idx + 2 - cw]
            }))
        } else {
            Either::Right(self.indices.clone().step_by(3).map(|i| [i, i + 1, i + 2]))
        }
    }
}
