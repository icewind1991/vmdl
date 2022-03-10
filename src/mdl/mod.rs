mod bone;
mod header;
mod header2;

pub use bone::*;
pub use header::*;
pub use header2::*;

use crate::{read_indexes, ModelError};
use binrw::BinReaderExt;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct Mdl {
    pub header: StudioHeader,
    pub bones: Vec<Bone>,
}

impl Mdl {
    pub fn read(data: &[u8]) -> Result<Self, ModelError> {
        let mut reader = Cursor::new(data);
        let header: StudioHeader = reader.read_le()?;
        let bones = read_indexes(header.bone_indexes(), data).collect::<Result<_, _>>()?;
        Ok(Mdl { header, bones })
    }
}
