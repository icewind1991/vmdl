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
    pub tangents: Vec<[f32; 4]>,
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
        let source_tangents = read_relative(
            data,
            header.tangent_indexes(0).ok_or(ModelError::OutOfBounds {
                data: "model_lod",
                offset: 0,
            })?,
        )?;
        let (tangents, vertices) = if !header.has_fixups() {
            (source_tangents, source_vertices)
        } else {
            let mut vertices = Vec::new();
            let mut tangents = Vec::new();
            for fixup in read_relative_iter::<'_, VertexFileFixup, _>(data, header.fixup_indexes())
            {
                let fixup = fixup?;
                let from = fixup.source_vertex_id as usize;
                let to = (fixup.source_vertex_id.saturating_add(fixup.vertex_count)) as usize;
                vertices.extend_from_slice(source_vertices.get(from..to).ok_or({
                    ModelError::OutOfBounds {
                        data: "source_vertices",
                        offset: to,
                    }
                })?);
                tangents.extend_from_slice(source_tangents.get(from..to).ok_or({
                    ModelError::OutOfBounds {
                        data: "source_tangents",
                        offset: to,
                    }
                })?);
            }
            (tangents, vertices)
        };

        debug_assert!(tangents.len() == vertices.len());

        Ok(Vvd {
            vertices,
            header,
            tangents,
        })
    }
}
