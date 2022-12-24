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
        let source_vertices = read_relative(data, header.vertex_indexes(0).unwrap())?;
        let vertices = if !header.has_fixups() {
            source_vertices
        } else {
            let mut vertices = Vec::new();
            for fixup in read_relative_iter::<'_, VertexFileFixup, _>(data, header.fixup_indexes())
            {
                let fixup = fixup?;
                vertices.extend_from_slice(
                    &source_vertices[fixup.source_vertex_id as usize
                        ..(fixup.source_vertex_id + fixup.vertex_count) as usize],
                );
            }
            vertices
        };
        Ok(Vvd { vertices, header })
    }
}
