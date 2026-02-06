//! Angle - rotation with CSS transform output

use std::f32::consts::PI;

/// Angle in degrees, normalized to [-180, 180]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Angle(f32);

impl Angle {
    pub const ZERO: Self = Self(0.0);

    pub fn new(degrees: f32) -> Self {
        Self(Self::normalize(degrees))
    }

    pub fn from_radians(radians: f32) -> Self {
        Self::new(radians * 180.0 / PI)
    }

    pub const fn degrees(&self) -> f32 {
        self.0
    }

    pub fn radians(&self) -> f32 {
        self.0 * PI / 180.0
    }

    fn normalize(degrees: f32) -> f32 {
        let mut d = degrees % 360.0;
        if d > 180.0 {
            d -= 360.0;
        } else if d < -180.0 {
            d += 360.0;
        }
        d
    }

    pub fn rotate(&self, by: Self) -> Self {
        Self::new(self.0 + by.0)
    }

    pub fn to_css(&self) -> String {
        if self.0.abs() < 0.01 {
            return String::new();
        }
        format!("rotate({}deg)", self.0)
    }

    pub fn describe(&self) -> &'static str {
        match self.0.abs() as u32 {
            0 => "no rotation",
            1..=15 => "slightly rotated",
            16..=45 => "moderately rotated",
            46..=90 => "heavily rotated",
            _ => "extremely rotated",
        }
    }

    /// Predefined rotation values for sampling
    pub const VOCABULARY: &[Self] = &[
        Self(0.0),
        Self(5.0),
        Self(10.0),
        Self(15.0),
        Self(30.0),
        Self(45.0),
        Self(90.0),
        Self(180.0),
        Self(-5.0),
        Self(-10.0),
        Self(-15.0),
        Self(-30.0),
        Self(-45.0),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_normalization() {
        let a = Angle::new(360.0);
        assert!(a.degrees().abs() < 0.01);

        let b = Angle::new(-270.0);
        assert!((b.degrees() - 90.0).abs() < 0.01);
    }

    #[test]
    fn angle_css() {
        assert_eq!(Angle::ZERO.to_css(), "");
        assert_eq!(Angle::new(45.0).to_css(), "rotate(45deg)");
    }

    #[test]
    fn angle_rotation() {
        let a = Angle::new(45.0);
        let b = a.rotate(Angle::new(45.0));
        assert!((b.degrees() - 90.0).abs() < 0.01);
    }
}
