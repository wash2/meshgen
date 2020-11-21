extern crate more_asserts;

use std::ops::{Add, Mul, Sub};

use half::f16;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Position2D32 {
    pub x: f32,
    pub y: f32,
}

impl From<Position3D32> for Position2D32 {
    fn from(pos3d: Position3D32) -> Self {
        Position2D32 {x: pos3d.x, y: pos3d.z}
    }
}

impl Add for Position2D32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: (self.x as f64 + other.x as f64).min(f32::MAX.into()).max(f32::MIN.into()) as f32,
            y: (self.y as f64 + other.y as f64).min(f32::MAX.into()).max(f32::MIN.into()) as f32,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Position3D32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Add for Position3D32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: (self.x as f64 + other.x as f64).min(f32::MAX.into()).max(f32::MIN.into()) as f32,
            y: (self.y as f64 + other.y as f64).min(f32::MAX.into()).max(f32::MIN.into()) as f32,
            z: (self.z as f64 + other.z as f64).min(f32::MAX.into()).max(f32::MIN.into()) as f32,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Normal16 {
    pub x: f16,
    pub y: f16,
    pub z: f16,
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Normal32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Tangent32 {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Debug)]
#[repr(C)]
pub struct TangentU8 {
    pub w: u8,
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct TexCoord32 {
    pub u: f32,
    pub v: f32
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
#[repr(C)]
pub struct Vertex {
    pub pos: Position3D32,
    pub norm: Normal32,
    pub tangent: Tangent32,
    pub uv: TexCoord32,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[repr(C)]
pub struct Triangle {
    pub v1: i32,
    pub v2: i32,
    pub v3: i32
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[repr(C)]
pub struct Quad {
    pub v1: i32,
    pub v2: i32,
    pub v3: i32,
    pub v4: i32,
    pub v5: i32,
    pub v6: i32,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[repr(C)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}


impl Add for Color32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            r: (self.r as i16 + other.r as i16).min(255).max(0) as u8,
            g: (self.g as i16 + other.g as i16).min(255).max(0) as u8,
            b: (self.b as i16 + other.b as i16).min(255).max(0) as u8,
            a: (self.a as i16 + other.a as i16).min(255).max(0) as u8,
        }
    }
}

impl Sub for Color32 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: (self.r as i16 - other.r as i16).min(255).max(0) as u8,
            g: (self.g as i16 - other.g as i16).min(255).max(0) as u8,
            b: (self.b as i16 - other.b as i16).min(255).max(0) as u8,
            a: (self.a as i16 - other.a as i16).min(255).max(0) as u8,
        }
    }
}

impl Mul<f64> for Color32 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            r: (self.r as f64 * rhs).min(255.0).max(0.0).round() as u8,
            g: (self.g as f64 * rhs).min(255.0).max(0.0).round() as u8,
            b: (self.b as f64 * rhs).min(255.0).max(0.0).round() as u8,
            a: (self.a as f64 * rhs).min(255.0).max(0.0).round() as u8,
        }
    }
}

pub trait Lerp {
    fn lerp(self, other: Self, t: f64) -> Self;
    fn lerp_bounded(self, other: Self, t: f64) -> Self;
}

impl<T> Lerp for T where T: Add<T, Output = T> + Sub<T, Output = T> + Mul<f64, Output = T> + Copy + PartialOrd {
    fn lerp(self, other: Self, t: f64) -> T {
        if self > other {
            other + (self - other) * (1_f64 - t)
        }
        else {
            self + (other - self) * t
        }
    }

    fn lerp_bounded(self, other: Self, t: f64) -> T {
        let t = t.min(1_f64).max(0_f64);
        self.lerp(other,t)
    }
}

#[cfg(test)]
mod unity_tests {
    use super::{Color32, Lerp};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_add_color() {
        let black = Color32{r: 0, g: 0, b: 0, a: 255};
        let white = Color32{r: 255, g: 255, b: 255, a: 255};
        let green = Color32{r: 0, g: 255, b: 0, a: 255};

        assert_eq!(white + white, white);
        assert_eq!(white + black, white);
        assert_eq!(green + black, green);
        assert_eq!(black + black, black);
    }

    #[test]
    fn test_mul_color() {
        let black = Color32{r: 0, g: 0, b: 0, a: 255};
        let white = Color32{r: 255, g: 255, b: 255, a: 255};
        let green = Color32{r: 0, g: 255, b: 0, a: 255};

        assert_eq!(white * 1.0, white);
        assert_eq!(black * 1.0, black);
        assert_eq!(green * 1.0, green);
    }

    #[test]
    fn test_lerp_color() {
        let black = Color32{r: 0, g: 0, b: 0, a: 255};
        let white = Color32{r: 255, g: 255, b: 255, a: 255};
        assert_eq!(white.lerp(black, 0.0), white);
        assert_eq!(white.lerp(black, 1.0), black);
        assert_eq!(white.lerp_bounded(black, -1.0), white);
        assert_eq!(white.lerp_bounded(black, 20.0), black);
        assert_eq!(white.lerp_bounded(black, 0.5), Color32{r: 128, g: 128, b: 128, a: 255});
        assert_eq!(white.lerp_bounded(black, 0.999), Color32{r: 0, g: 0, b: 0, a: 255});

    }
}
