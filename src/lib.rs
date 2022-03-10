mod error;
mod handle;
pub mod mdl;
mod shared;
pub mod vtx;
pub mod vvd;

use binrw::{BinRead, BinReaderExt};
pub use error::*;
pub use handle::Handle;
pub use shared::*;
use std::io::Cursor;

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
                data: "Bone",
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
