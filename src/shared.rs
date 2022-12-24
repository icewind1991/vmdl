use crate::{ModelError, StringError};
use arrayvec::ArrayString;
use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vector> for Vector3<f32> {
    fn from(v: Vector) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vector3<f32>> for Vector {
    fn from(v: Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl Vector {
    pub fn iter(&self) -> impl Iterator<Item = f32> {
        [self.x, self.y, self.z].into_iter()
    }
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

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct RadianEuler {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Fixed length, null-terminated string
#[derive(Debug, Clone)]
pub struct FixedString<const LEN: usize>(ArrayString<LEN>);

impl<const LEN: usize> TryFrom<[u8; LEN]> for FixedString<LEN> {
    type Error = ModelError;

    fn try_from(name_buf: [u8; LEN]) -> Result<Self, Self::Error> {
        use std::str;

        let zero_pos = name_buf
            .iter()
            .position(|c| *c == 0)
            .ok_or(StringError::NotNullTerminated)?;
        let name = &name_buf[..zero_pos];
        Ok(FixedString(
            ArrayString::from(str::from_utf8(name).map_err(StringError::NonUTF8)?).unwrap(),
        ))
    }
}

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
