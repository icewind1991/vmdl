mod raw;

pub use raw::header::*;
pub use raw::header2::*;
pub use raw::*;
use std::mem::size_of;

use crate::vvd::Vertex;
use crate::{
    read_indexes, read_relative, read_relative_iter, FixedString, ModelError, ReadRelative,
    Readable,
};

type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct Mdl {
    pub name: FixedString<64>,
    pub header: StudioHeader,
    pub bones: Vec<Bone>,
    pub body_parts: Vec<BodyPart>,
    pub textures: Vec<TextureInfo>,
    pub texture_paths: Vec<String>,
    pub skin_table: Vec<u16>,
}

impl Mdl {
    pub fn read(data: &[u8]) -> Result<Self> {
        let header = <StudioHeader as Readable>::read(data)?;
        let name = header.name.try_into()?;
        let mut textures = read_relative_iter(data, header.texture_indexes())
            .collect::<Result<Vec<TextureInfo>>>()?;
        let texture_dirs_indexes =
            read_relative_iter(data, header.texture_dir_indexes()).collect::<Result<Vec<u32>>>()?;
        let texture_paths = read_relative_iter::<String, _>(
            data,
            texture_dirs_indexes.into_iter().map(|index| index as usize),
        )
        .map(|path| path.map(|path| path.replace('\\', "/")))
        .collect::<Result<Vec<_>>>()?;
        for texture in textures.iter_mut() {
            texture.search_paths = texture_paths.clone();
        }

        let skin_table = read_relative::<u16, _>(data, header.skin_reference_indexes())?;

        let bones = read_indexes(header.bone_indexes(), data).collect::<Result<_>>()?;
        Ok(Mdl {
            name,
            bones,
            body_parts: header
                .body_part_indexes()
                .map(|index| {
                    let data = data.get(index..).ok_or(ModelError::OutOfBounds {
                        data: "BodyPart",
                        offset: index,
                    })?;
                    let header = <BodyPartHeader as Readable>::read(data)?;
                    BodyPart::read(data, header)
                })
                .collect::<Result<_>>()?,
            textures,
            texture_paths,
            skin_table,
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
            vertex_offset: header.vertex_index / (size_of::<Vertex>() as i32),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub material: i32,
    pub vertex_offset: i32,
}

impl ReadRelative for Mesh {
    type Header = MeshHeader;

    fn read(_data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(Mesh {
            material: header.material,
            vertex_offset: header.vertex_index,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub name: String,
    pub name_index: i32,
    pub search_paths: Vec<String>,
}

impl ReadRelative for TextureInfo {
    type Header = MeshTexture;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(TextureInfo {
            name: String::read(&data[header.name_index as usize..], ())?.replace('\\', "/"),
            name_index: header.name_index,
            search_paths: Vec::new(),
        })
    }
}
