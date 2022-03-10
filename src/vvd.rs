use binrw::BinRead;

pub const MDL_VERSION: i32 = 7;

#[derive(Debug, Clone, BinRead)]
pub struct Header {
    pub id: i32,
    pub version: i32,
    pub checksum: [u8; 4],
    pub lod_count: i32,
    pub lod_vertex_count: [i32; 8],
    pub fixup_count: i32,
    pub fixup_index: i32,
    pub vertex_index: i32,
    pub tangent_index: i32,
}
