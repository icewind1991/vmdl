mod raw;

pub use raw::header::*;
pub use raw::header2::*;
pub use raw::*;
use std::mem::size_of;
use bytemuck::{Pod, Zeroable};

use crate::vvd::Vertex;
use crate::{read_relative, read_relative_iter, read_single, FixedString, ModelError, ReadRelative, Readable, Transform3x4};

type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct Mdl {
    pub name: FixedString<64>,
    pub header: StudioHeader,
    pub header2: Option<StudioHeader2>,
    pub bones: Vec<Bone>,
    pub body_parts: Vec<BodyPart>,
    pub textures: Vec<TextureInfo>,
    pub texture_paths: Vec<String>,
    pub skin_table: Vec<u16>,
    pub surface_prop: String,
    pub key_values: Option<String>,
    pub local_animations: Vec<AnimationDescription>,
    pub pose_parameters: Vec<PoseParameterDescription>,
    pub attachments: Vec<StudioAttachment>,
}

impl Mdl {
    pub fn read(data: &[u8]) -> Result<Self> {
        let header = <StudioHeader as Readable>::read(data)?;
        let header2 = header
            .header2_index()
            .map(|index| read_single::<StudioHeader2, _>(data, index))
            .transpose()?;
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
        let bones = read_relative(data, header.bone_indexes())?;

        // are these used?
        let _source_bone_transforms: Vec<SourceBoneTransform> = read_relative(data, header2.unwrap().source_bone_transforms())?;

        let surface_prop = read_single(data, header.surface_prop_index)?;
        let key_values = (header.key_value_size > 0)
            .then(|| read_single(data, header.key_value_index))
            .transpose()?;
        let local_animations = read_relative(data, header.local_animation_indexes())?;
        let pose_parameters = read_relative(data, header.local_pose_param_indexes())?;
        let attachments = read_relative(data, header.attachment_indexes())?;

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
            header2,
            surface_prop,
            key_values,
            pose_parameters,
            local_animations,
            attachments,
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

#[derive(Debug, Clone)]
pub struct StudioAttachment {
    pub name: String,
    pub flags: AttachmentFlags,
    pub local_bone: i32,
    pub local: Transform3x4,
}

impl ReadRelative for StudioAttachment {
    type Header = StudioAttachmentHeader;

    fn read(data: &[u8], header: Self::Header) -> Result<Self> {
        Ok(StudioAttachment {
            name: String::read(&data[header.name_index as usize..], ())?.replace('\\', "/"),
            flags: header.flags,
            local: header.local,
            local_bone: header.local_bone,
        })
    }
}
