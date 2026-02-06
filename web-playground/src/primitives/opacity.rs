//! Opacity - visibility/transparency with CSS output

use super::bounded::bounded_f32;

bounded_f32!(Opacity, 0.0, 1.0);

impl Opacity {
    pub const FULL: Self = Self::new(1.0);
    pub const HALF: Self = Self::new(0.5);
    pub const ZERO: Self = Self::new(0.0);

    pub const fn is_visible(&self) -> bool {
        self.0 > 0.0
    }

    pub const fn is_opaque(&self) -> bool {
        self.0 >= 1.0
    }

    pub fn to_css(&self) -> String {
        if self.is_opaque() {
            return String::new(); // no CSS needed at full opacity
        }
        format!("opacity: {:.2};", self.0)
    }

    pub fn describe(&self) -> &'static str {
        match self.0 {
            x if x >= 1.0 => "fully opaque",
            x if x >= 0.7 => "mostly opaque",
            x if x >= 0.3 => "semi-transparent",
            x if x > 0.0 => "mostly transparent",
            _ => "invisible",
        }
    }

    pub const ALL: &[Self] = &[
        Self::new(1.0),
        Self::new(0.8),
        Self::new(0.6),
        Self::new(0.4),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_css() {
        assert_eq!(Opacity::FULL.to_css(), "");
        assert_eq!(Opacity::HALF.to_css(), "opacity: 0.50;");
    }

    #[test]
    fn opacity_describe() {
        assert_eq!(Opacity::FULL.describe(), "fully opaque");
        assert_eq!(Opacity::HALF.describe(), "semi-transparent");
    }
}
