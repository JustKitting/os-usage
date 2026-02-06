//! Scale - CSS transform scale factor

use super::bounded::bounded_f32;

bounded_f32!(Scale, 0.25, 3.0);

impl Scale {
    pub const NORMAL: Self = Self::new(1.0);
    pub const HALF: Self = Self::new(0.5);
    pub const DOUBLE: Self = Self::new(2.0);

    pub fn to_css(&self) -> String {
        if (self.0 - 1.0).abs() < 0.01 {
            return String::new();
        }
        format!("scale({:.2})", self.0)
    }

    pub fn describe(&self) -> &'static str {
        match self.0 {
            x if x < 0.6 => "very small",
            x if x < 0.9 => "small",
            x if x <= 1.1 => "normal size",
            x if x <= 1.5 => "enlarged",
            _ => "very large",
        }
    }

    /// Predefined scale values for sampling
    pub const VOCABULARY: &[Self] = &[
        Self(0.5),
        Self(0.75),
        Self(1.0),
        Self(1.25),
        Self(1.5),
        Self(2.0),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_css() {
        assert_eq!(Scale::NORMAL.to_css(), "");
        assert_eq!(Scale::DOUBLE.to_css(), "scale(2.00)");
        assert_eq!(Scale::HALF.to_css(), "scale(0.50)");
    }

    #[test]
    fn scale_describe() {
        assert_eq!(Scale::NORMAL.describe(), "normal size");
        assert_eq!(Scale::DOUBLE.describe(), "very large");
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn scale_rejects_zero() {
        let _ = Scale::new(0.0);
    }
}
