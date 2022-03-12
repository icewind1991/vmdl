mod error;
mod handle;
pub mod mdl;
mod shared;
pub mod vtx;
pub mod vvd;

use crate::mdl::Mdl;
use crate::vtx::Vtx;
use crate::vvd::{Vertex, Vvd};
use binrw::{BinRead, BinReaderExt};
pub use error::*;
pub use handle::Handle;
pub use shared::*;
use std::any::type_name;
use std::io::Cursor;

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

    pub fn vertex_strips(&self) -> impl Iterator<Item = impl Iterator<Item = &'_ Vertex> + '_> {
        self.vertex_strip_indices()
            .map(|strip| strip.map(|index| &self.vvd.vertices[index]))
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vvd.vertices
    }

    pub fn vertex_strip_indices(&self) -> impl Iterator<Item = impl Iterator<Item = usize> + '_> {
        let mdl_meshes = self
            .mdl
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| model.meshes.iter());

        let vtx_meshes = self
            .vtx
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| model.lods.iter().next())
            .flat_map(|lod| lod.meshes.iter());

        vtx_meshes
            .zip(mdl_meshes)
            .flat_map(|(vtx_mesh, mdl_mesh)| {
                vtx_mesh
                    .strip_groups
                    .iter()
                    .map(move |strip_group| (strip_group, mdl_mesh))
            })
            .flat_map(|(strip_group, mdl_mesh)| {
                let group_indices = &strip_group.indices;
                let vertices = &strip_group.vertices;
                let mesh_vertex_offset = mdl_mesh.vertex_offset as usize;
                strip_group.strips.iter().cloned().map(move |strip| {
                    strip
                        .indices()
                        .flat_map(|i| i)
                        .map(move |index| group_indices[index] as usize)
                        .map(move |index| {
                            vertices[index].original_mesh_vertex_id as usize + mesh_vertex_offset
                        })
                })
            })
    }
}

fn read_indexes<'a, I: Iterator<Item = usize> + 'static, T: BinRead>(
    indexes: I,
    data: &'a [u8],
) -> impl Iterator<Item = Result<T, ModelError>> + 'a
where
    T::Args: Default,
{
    indexes
        .map(|index| {
            data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                data: type_name::<T>(),
                offset: index,
            })
        })
        .map(|data| {
            data.and_then(|data| {
                let mut cursor = Cursor::new(data);
                cursor.read_le().map_err(ModelError::from)
            })
        })
}

fn index_range(index: i32, count: i32, size: usize) -> impl Iterator<Item = usize> {
    (0..count as usize)
        .map(move |i| i * size)
        .map(move |i| index as usize + i)
}
