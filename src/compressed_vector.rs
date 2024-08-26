use crate::{Quaternion, ReadableRelative, Vector};
use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Vector4};
use half::f16;

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vector48 {
    x: u16,
    y: u16,
    z: u16,
}

impl ReadableRelative for Vector48 {}

impl Vector48 {
    pub fn x(&self) -> f32 {
        f16::from_bits(self.x).into()
    }
    pub fn y(&self) -> f32 {
        f16::from_bits(self.y).into()
    }
    pub fn z(&self) -> f32 {
        f16::from_bits(self.z).into()
    }
}

impl From<Vector48> for Vector {
    fn from(value: Vector48) -> Self {
        Vector {
            x: value.x(),
            y: value.y(),
            z: value.z(),
        }
    }
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct Quaternion48 {
    x: u16,
    y: u16,
    z: u16,
}

impl ReadableRelative for Quaternion48 {}

fn calc_w(x: f32, y: f32, z: f32, w_neg: bool) -> f32 {
    let w_sign = if w_neg { -1.0 } else { 1.0 };
    f32::sqrt(1.0 - ((x * x) - (y * y) - (z * z))) * w_sign
}

impl Quaternion48 {
    const Z_MASK: u16 = 0x7F_FF;
    const W_NEG_MASK: u16 = 0x80_00;

    pub fn x(&self) -> f32 {
        ((self.x as f32) - 32768.0) / 32768.0
    }
    pub fn y(&self) -> f32 {
        ((self.y as f32) - 32768.0) / 32768.0
    }
    pub fn z(&self) -> f32 {
        (((self.z & Self::Z_MASK) as f32) - 16384.0) / 16384.0
    }
    pub fn w(&self) -> f32 {
        calc_w(
            self.x(),
            self.y(),
            self.z(),
            self.z & Self::W_NEG_MASK == Self::W_NEG_MASK,
        )
    }
}

impl From<Quaternion48> for Quaternion {
    fn from(value: Quaternion48) -> Self {
        let normalized = Vector4::new(value.x(), value.y(), value.z(), value.w()).normalize();
        Quaternion {
            x: normalized.x,
            y: normalized.y,
            z: normalized.z,
            w: normalized.w,
        }
    }
}

#[derive(Zeroable, Pod, Copy, Clone, Debug)]
#[repr(C)]
pub struct Quaternion64(u64);

impl ReadableRelative for Quaternion64 {}

impl Quaternion64 {
    const MASK_21_BIT: u64 = 0b111111111111111111111;
    const W_NEG_MASK: u64 = 0x80_00_00_00_00_00_00_00;

    fn val(&self, offset: i32) -> f32 {
        let raw = (self.0 >> offset) & Self::MASK_21_BIT;
        (raw as f32 - 1048576.0) / 1048576.5
    }

    pub fn x(&self) -> f32 {
        self.val(0)
    }
    pub fn y(&self) -> f32 {
        self.val(21)
    }
    pub fn z(&self) -> f32 {
        self.val(42)
    }
    pub fn w(&self) -> f32 {
        calc_w(
            self.x(),
            self.y(),
            self.z(),
            self.0 & Self::W_NEG_MASK == Self::W_NEG_MASK,
        )
    }
}

impl From<Quaternion64> for Quaternion {
    fn from(value: Quaternion64) -> Self {
        let normalized = Vector4::new(value.x(), value.y(), value.z(), value.w()).normalize();
        Quaternion {
            x: normalized.x,
            y: normalized.y,
            z: normalized.z,
            w: normalized.w,
        }
    }
}
