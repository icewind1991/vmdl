mod error;
mod handle;
pub mod mdl;
mod shared;
pub mod vtx;
pub mod vvd;

pub use crate::mdl::Mdl;
use crate::mdl::TextureInfo;
pub use crate::vtx::Vtx;
use crate::vvd::Vertex;
pub use crate::vvd::Vvd;
use bytemuck::{pod_read_unaligned, Pod};
pub use error::*;
pub use handle::Handle;
pub use shared::*;
use std::any::type_name;
use std::fs;
use std::mem::size_of;
use std::path::Path;

pub struct Model {
    #[allow(dead_code)]
    mdl: Mdl,
    vtx: Vtx,
    vvd: Vvd,
}

impl Model {
    pub fn from_parts(mdl: Mdl, vtx: Vtx, vvd: Vvd) -> Self {
        Model { mdl, vtx, vvd }
    }

    /// Load the model from path
    ///
    /// Requires a path to the `.mdl` file and the `.dx90.vtx` and `.vvd` files for the model to be in the same directory.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ModelError> {
        let path = path.as_ref();
        let data = fs::read(path)?;
        let mdl = Mdl::read(&data)?;
        let data = fs::read(path.with_extension("dx90.vtx"))?;
        let vtx = Vtx::read(&data)?;
        let data = fs::read(path.with_extension("vvd"))?;
        let vvd = Vvd::read(&data)?;

        Ok(Model::from_parts(mdl, vtx, vvd))
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vvd.vertices
    }

    pub fn tangents(&self) -> &[[f32; 4]] {
        &self.vvd.tangents
    }

    pub fn texture_directories(&self) -> &[String] {
        &self.mdl.texture_paths
    }

    pub fn textures(&self) -> &[TextureInfo] {
        &self.mdl.textures
    }

    pub fn skin_tables(&self) -> impl Iterator<Item = SkinTable> {
        self.mdl
            .skin_table
            .chunks(self.mdl.header.skin_reference_count as usize)
            .map(|chunk| SkinTable {
                table: chunk,
                textures: &self.mdl.textures,
            })
    }

    pub fn meshes(&self) -> impl Iterator<Item = Mesh> {
        let mdl_meshes = self
            .mdl
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| {
                model
                    .meshes
                    .iter()
                    .map(|mesh| (mesh, model.name.as_str(), model.vertex_offset as usize))
            });

        let vtx_meshes = self
            .vtx
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| model.lods.first())
            .flat_map(|lod| lod.meshes.iter());

        mdl_meshes
            .zip(vtx_meshes)
            .map(|((mdl, model_name, model_vertex_offset), vtx)| Mesh {
                model_vertex_offset,
                model_name,
                vertices: self.vertices(),
                tangents: self.tangents(),
                mdl,
                vtx,
            })
    }

    /// Calculate bounding coordinates of the model
    pub fn bounding_box(&self) -> (Vector, Vector) {
        let mut min = Vector::from([f32::MAX, f32::MAX, f32::MAX]);
        let mut max = Vector::from([f32::MIN, f32::MIN, f32::MIN]);

        for point in self.vertices() {
            let p = point.position;
            min.x = f32::min(min.x, p.x);
            min.y = f32::min(min.y, p.y);
            min.z = f32::min(min.z, p.z);

            max.x = f32::max(max.x, p.x);
            max.y = f32::max(max.y, p.y);
            max.z = f32::max(max.z, p.z);
        }
        (min, max)
    }

    pub fn name(&self) -> &str {
        self.mdl.name.as_str()
    }
}

pub struct SkinTable<'a> {
    textures: &'a [TextureInfo],
    table: &'a [u16],
}

impl<'a> SkinTable<'a> {
    pub fn texture(&self, index: i32) -> Option<&'a str> {
        let texture_index = self.table.get(index as usize)?;
        self.textures
            .get(*texture_index as usize)
            .map(|info| info.name.as_str())
    }

    pub fn texture_index(&self, index: i32) -> Option<usize> {
        let texture_index = self.table.get(index as usize)?;
        Some(*texture_index as usize)
    }
}

pub struct Mesh<'a> {
    pub model_name: &'a str,
    model_vertex_offset: usize,
    vertices: &'a [Vertex],
    tangents: &'a [[f32; 4]],
    mdl: &'a mdl::Mesh,
    vtx: &'a vtx::Mesh,
}

impl<'a> Mesh<'a> {
    /// Vertex indices into the model's vertex list
    pub fn vertex_strip_indices(&self) -> impl Iterator<Item = impl Iterator<Item = usize> + 'a> {
        let mdl_offset = self.mdl.vertex_offset as usize + self.model_vertex_offset;
        self.vtx.strip_groups.iter().flat_map(move |strip_group| {
            let group_indices = &strip_group.indices;
            let vertices = &strip_group.vertices;
            strip_group.strips.iter().map(move |strip| {
                strip
                    .indices()
                    .map(move |index| group_indices[index] as usize)
                    .map(move |index| vertices[index].original_mesh_vertex_id as usize + mdl_offset)
            })
        })
    }

    pub fn material_index(&self) -> i32 {
        self.mdl.material
    }

    pub fn vertices(&self) -> impl Iterator<Item = &'a Vertex> + 'a {
        self.vertex_strip_indices()
            .flat_map(|strip| strip.map(|index| &self.vertices[index]))
    }

    pub fn tangents(&self) -> impl Iterator<Item = [f32; 4]> + '_ {
        self.vertex_strip_indices()
            .flat_map(|strip| strip.map(|index| self.tangents[index]))
    }
}

fn read_indexes<I: Iterator<Item = usize> + 'static, T: Readable>(
    indexes: I,
    data: &[u8],
) -> impl Iterator<Item = Result<T, ModelError>> + '_ {
    indexes
        .map(|index| {
            data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                data: type_name::<T>(),
                offset: index,
            })
        })
        .map(|data| data.and_then(|data| T::read(data)))
}

fn index_range(index: i32, count: i32, size: usize) -> impl Iterator<Item = usize> {
    (0..count as usize)
        .map(move |i| i * size)
        .map(move |i| index as usize + i)
}

fn read_relative_iter<'a, T: ReadRelative, I: 'a + Iterator<Item = usize>>(
    data: &'a [u8],
    indexes: I,
) -> impl Iterator<Item = Result<T, ModelError>> + 'a {
    indexes.map(|index| {
        let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
            data: type_name::<T>(),
            offset: index,
        })?;
        let header = <T::Header as Readable>::read(data)?;
        T::read(data, header)
    })
}

fn read_relative<T: ReadRelative, I: Iterator<Item = usize>>(
    data: &[u8],
    indexes: I,
) -> Result<Vec<T>, ModelError> {
    read_relative_iter(data, indexes).collect()
}

trait Readable: Sized {
    fn read(data: &[u8]) -> Result<Self, ModelError>;
}

impl<T: Pod> Readable for T {
    fn read(data: &[u8]) -> Result<Self, ModelError> {
        let data = data
            .get(0..size_of::<Self>())
            .ok_or(ModelError::Eof(size_of::<Self>()))?;
        Ok(pod_read_unaligned(data))
    }
}

trait ReadRelative: Sized {
    type Header: Readable;

    fn read(data: &[u8], header: Self::Header) -> Result<Self, ModelError>;
}

trait ReadableRelative: Readable {}

impl ReadableRelative for u8 {}
impl ReadableRelative for u16 {}
impl ReadableRelative for u32 {}
impl ReadableRelative for i8 {}
impl ReadableRelative for i16 {}
impl ReadableRelative for i32 {}
impl ReadableRelative for f32 {}
impl<T: ReadableRelative + Pod> ReadableRelative for [T; 1] {}
impl<T: ReadableRelative + Pod> ReadableRelative for [T; 2] {}
impl<T: ReadableRelative + Pod> ReadableRelative for [T; 3] {}
impl<T: ReadableRelative + Pod> ReadableRelative for [T; 4] {}

impl<T: ReadableRelative> ReadRelative for T {
    type Header = T;

    fn read(_data: &[u8], header: Self::Header) -> Result<Self, ModelError> {
        Ok(header)
    }
}

impl ReadRelative for String {
    type Header = ();

    fn read(data: &[u8], _header: Self::Header) -> Result<Self, ModelError> {
        let bytes = data.iter().copied().take_while(|byte| *byte != 0).collect();
        String::from_utf8(bytes).map_err(ModelError::from)
    }
}
