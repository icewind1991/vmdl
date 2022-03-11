use crate::{BinRead, StringError};
use arrayvec::ArrayString;
use binrw::{BinResult, ReadOptions};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, BinRead)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vector> for [f32; 3] {
    fn from(vector: Vector) -> Self {
        [vector.x, vector.y, vector.z]
    }
}

impl From<[f32; 3]> for Vector {
    fn from(vector: [f32; 3]) -> Self {
        Vector {
            x: vector[0],
            y: vector[1],
            z: vector[2],
        }
    }
}

impl From<&Vector> for [f32; 3] {
    fn from(vector: &Vector) -> Self {
        [vector.x, vector.y, vector.z]
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, BinRead)]
pub struct RadianEuler {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Fixed length, null-terminated string
#[derive(Debug, Clone)]
pub struct FixedString<const LEN: usize>(ArrayString<LEN>);

impl<const N: usize> AsRef<str> for FixedString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> FixedString<N> {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<const LEN: usize> Display for FixedString<LEN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<const LEN: usize> BinRead for FixedString<LEN> {
    type Args = ();

    fn read_options<R: binrw::io::Read + binrw::io::Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        use std::str;

        let name_buf = <[u8; LEN]>::read_options(reader, options, args)?;

        let zero_pos =
            name_buf
                .iter()
                .position(|c| *c == 0)
                .ok_or_else(|| binrw::Error::Custom {
                    pos: reader.stream_position().unwrap(),
                    err: Box::new(StringError::NotNullTerminated),
                })?;
        let name = &name_buf[..zero_pos];
        Ok(FixedString(
            ArrayString::from(
                str::from_utf8(name)
                    .map_err(StringError::NonUTF8)
                    .map_err(|e| binrw::Error::Custom {
                        pos: reader.stream_position().unwrap(),
                        err: Box::new(e),
                    })?,
            )
            .unwrap(),
        ))
    }
}