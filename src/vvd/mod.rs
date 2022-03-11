mod raw;

use crate::vvd::raw::VvdHeader;
use crate::ModelError;
use binrw::BinReaderExt;
pub use raw::{BoneWeight, Tangent, Vertex};
use std::io::Cursor;

type Result<T> = std::result::Result<T, ModelError>;

#[derive(Debug, Clone)]
pub struct Vvd {
    pub header: VvdHeader,
    pub vertices: Vec<Vertex>,
}

impl Vvd {
    pub fn read(data: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let header: VvdHeader = reader.read_le()?;
        Ok(Vvd {
            vertices: header
                .vertex_indexes(0)
                .unwrap()
                .map(|index| {
                    let data = data.get(index..).ok_or_else(|| ModelError::OutOfBounds {
                        data: "Vertex",
                        offset: index,
                    })?;
                    let mut reader = Cursor::new(data);
                    Ok(reader.read_le()?)
                })
                .collect::<Result<_>>()?,
            header,
        })
    }
}
