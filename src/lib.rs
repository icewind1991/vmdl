mod data;
mod error;
mod handle;

use binrw::{BinRead, BinReaderExt};
pub use data::*;
pub use error::*;
pub use handle::Handle;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct Mdl {
    pub header: StudioHeader,
    pub bones: Vec<Bone>,
}

impl Mdl {
    pub fn read(data: &[u8]) -> Result<Self, MdlError> {
        let mut reader = Cursor::new(data);
        let header: StudioHeader = reader.read_le()?;
        let bones = read_indexes(header.bone_indexes(), data).collect::<Result<_, _>>()?;
        Ok(Mdl { header, bones })
    }
}

fn read_indexes<'a, I: Iterator<Item = usize> + 'static, T: BinRead>(
    indexes: I,
    data: &'a [u8],
) -> impl Iterator<Item = Result<T, MdlError>> + 'a
where
    T::Args: Default,
{
    indexes
        .map(|index| data.get(index..).ok_or(MdlError::OutOfBounds))
        .map(|data| {
            data.and_then(|data| {
                let mut cursor = Cursor::new(data);
                cursor.read_le().map_err(MdlError::from)
            })
        })
}
