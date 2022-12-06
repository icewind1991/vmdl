mod raw;

use std::mem::size_of;
pub use raw::header::*;
pub use raw::header2::*;

use crate::mdl::raw::{BodyPartHeader, Bone, MeshHeader, ModelHeader};
use crate::{read_indexes, read_relative, FixedString, ModelError, ReadRelative, Readable};
use crate::vvd::Vertex;

type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct Mdl {
    pub header: StudioHeader,
    pub bones: Vec<Bone>,
    pub body_parts: Vec<BodyPart>,
}

impl Mdl {
    pub fn read(data: &[u8]) -> Result<Self> {
        let header = <StudioHeader as Readable>::read(data)?;
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
                    let header = <BodyPartHeader as Readable>::read(data)?;
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

impl ReadRelative for BodyPart {
    type Header = BodyPartHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(BodyPart {
            models: read_relative(data, header.model_indexes())?,
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
    /// Base offset of the model's vertices
    pub vertex_offset: i32,
}

impl ReadRelative for Model {
    type Header = ModelHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(Model {
            meshes: read_relative(data, header.mesh_indexes())?,
            name: header.name.try_into()?,
            ty: header.ty,
            bounding_radius: header.bounding_radius,
            vertex_offset: header.vertex_index  / (size_of::<Vertex>() as i32),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertex_offset: i32,
}

impl ReadRelative for Mesh {
    type Header = MeshHeader;

    fn read(_data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(Mesh {
            vertex_offset: header.vertex_index,
        })
    }
}
