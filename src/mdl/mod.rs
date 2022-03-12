mod raw;

pub use raw::header::*;
pub use raw::header2::*;

use crate::mdl::raw::{BodyPartHeader, Bone, MeshHeader, ModelHeader};
use crate::{read_indexes, FixedString, ModelError};
use binrw::BinReaderExt;
use std::io::Cursor;

type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct Mdl {
    pub header: StudioHeader,
    pub bones: Vec<Bone>,
    pub body_parts: Vec<BodyPart>,
}

impl Mdl {
    pub fn read(data: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let header: StudioHeader = reader.read_le()?;
        let bones = read_indexes(header.bone_indexes(), data).collect::<Result<_>>()?;
        Ok(Mdl {
            bones,
            body_parts: header
                .body_part_indexes()
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
    pub name_index: i32,
    pub models: Vec<Model>,
}

impl BodyPart {
    pub fn read(data: &[u8], header: BodyPartHeader) -> Result<Self> {
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
            name_index: header.name_index,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    pub name: FixedString<64>,
    pub ty: i32,
    pub bounding_radius: f32,
    pub meshes: Vec<Mesh>,
}

impl Model {
    pub fn read(data: &[u8], header: ModelHeader) -> Result<Self> {
        Ok(Model {
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
            name: header.name,
            ty: header.ty,
            bounding_radius: header.bounding_radius,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertex_offset: i32,
}

impl Mesh {
    pub fn read(_data: &[u8], header: MeshHeader) -> Result<Self> {
        Ok(Mesh {
            vertex_offset: header.vertex_index,
        })
    }
}
