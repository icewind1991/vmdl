mod raw;

use crate::vvd::raw::{VertexFileFixup, VvdHeader};
use crate::{read_relative, read_relative_iter, ModelError, Readable};
pub use raw::{BoneWeight, Tangent, Vertex};

type Result<T> = std::result::Result<T, ModelError>;

/// The vvd file contains the raw vertex data that will be indexed into based on the vtx data
#[derive(Debug, Clone)]
pub struct Vvd {
    pub header: VvdHeader,
    pub vertices: Vec<Vertex>,
}

impl Vvd {
    pub fn read(data: &[u8]) -> Result<Self> {
        let header = <VvdHeader as Readable>::read(data)?;
        let source_vertices = read_relative(
            data,
            header.vertex_indexes(0).ok_or(ModelError::OutOfBounds {
                data: "model_lod",
                offset: 0,
            })?,
        )?;
        let vertices = if !header.has_fixups() {
            source_vertices
        } else {
            let mut vertices = Vec::new();
            for fixup in read_relative_iter::<'_, VertexFileFixup, _>(data, header.fixup_indexes())
            {
                let fixup = fixup?;
                let from = fixup.source_vertex_id as usize;
                let to = (fixup.source_vertex_id.saturating_add(fixup.vertex_count)) as usize;
                vertices.extend_from_slice(&source_vertices.get(from..to).ok_or_else(|| {
                    ModelError::OutOfBounds {
                        data: "source_vertices",
                        offset: to as usize,
                    }
                })?);
            }
            vertices
        };
        Ok(Vvd { vertices, header })
    }
}
