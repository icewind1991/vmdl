mod error;
mod handle;
pub mod mdl;
mod shared;
pub mod vtx;
pub mod vvd;

use crate::mdl::Mdl;
use crate::vtx::Vtx;
use crate::vvd::{Vertex, Vvd};
use bytemuck::{pod_read_unaligned, Pod};
pub use error::*;
pub use handle::Handle;
pub use shared::*;
use std::any::type_name;
use std::mem::size_of;

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
        let mesh_vertex_offsets = self
            .mdl
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| {
                model
                    .meshes
                    .iter()
                    .map(move |mesh| (mesh.vertex_offset + model.vertex_offset) as usize)
            });

        let vtx_meshes = self
            .vtx
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| model.lods.iter().next())
            .flat_map(|lod| lod.meshes.iter());

        vtx_meshes
            .zip(mesh_vertex_offsets)
            .flat_map(|(vtx_mesh, vertex_offset)| {
                vtx_mesh
                    .strip_groups
                    .iter()
                    .map(move |strip_group| (strip_group, vertex_offset))
            })
            .flat_map(|(strip_group, mesh_vertex_offset)| {
                let group_indices = &strip_group.indices;
                let vertices = &strip_group.vertices;
                strip_group.strips.iter().map(move |strip| {
                    strip
                        .indices()
                        .map(move |index| group_indices[index] as usize)
                        .map(move |index| {
                            vertices[index].original_mesh_vertex_id as usize + mesh_vertex_offset
                        })
                })
            })
    }
}

fn read_indexes<'a, I: Iterator<Item = usize> + 'static, T: Readable>(
    indexes: I,
    data: &'a [u8],
) -> impl Iterator<Item = Result<T, ModelError>> + 'a {
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
        let data = &data[0..size_of::<Self>()];
        Ok(pod_read_unaligned(data))
    }
}

trait ReadRelative: Sized {
    type Header: Readable;

    fn read(data: &[u8], header: Self::Header) -> Result<Self, ModelError>;
}

impl<T: Readable> ReadRelative for T {
    type Header = T;

    fn read(_data: &[u8], header: Self::Header) -> Result<Self, ModelError> {
        Ok(header)
    }
}
