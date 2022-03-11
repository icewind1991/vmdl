mod error;
mod handle;
pub mod mdl;
mod shared;
pub mod vtx;
pub mod vvd;

use crate::mdl::Mdl;
use crate::vtx::Vtx;
use crate::vvd::Vvd;
use binrw::{BinRead, BinReaderExt};
pub use error::*;
pub use handle::Handle;
pub use shared::*;
use std::any::type_name;
use std::io::Cursor;

pub struct Model {
    mdl: Mdl,
    vtx: Vtx,
    vvd: Vvd,
}

impl Model {
    pub fn from_parts(mdl: Mdl, vtx: Vtx, vvd: Vvd) -> Self {
        Model { mdl, vtx, vvd }
    }

    pub fn vertex_strips(&self) -> impl Iterator<Item = impl Iterator<Item = Vector> + '_> {
        self.vtx
            .body_parts
            .iter()
            .flat_map(|part| part.models.iter())
            .flat_map(|model| model.lods.iter().next())
            .flat_map(|lod| lod.meshes.iter())
            .flat_map(|mesh| mesh.strip_groups.iter())
            .map(|strip_group| {
                strip_group
                    .indices
                    .iter()
                    .map(|index| self.vvd.vertices[(*index) as usize].position)
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
