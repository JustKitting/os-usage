//! Position - element placement with CSS output
//!
//! Positions are absolute within the 1024x1024 viewport.

/// Position in pixels, absolute within the viewport
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };
    pub const CENTER: Self = Self { x: 512.0, y: 512.0 };
    pub const VIEWPORT: f32 = 1024.0;

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create from percentage of viewport (0.0 - 1.0)
    pub fn from_fraction(fx: f32, fy: f32) -> Self {
        Self {
            x: fx * Self::VIEWPORT,
            y: fy * Self::VIEWPORT,
        }
    }

    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Clamp to viewport bounds, accounting for element size
    pub fn clamp_to_viewport(&self, elem_width: f32, elem_height: f32) -> Self {
        Self {
            x: self.x.clamp(0.0, Self::VIEWPORT - elem_width),
            y: self.y.clamp(0.0, Self::VIEWPORT - elem_height),
        }
    }

    pub fn to_css(&self) -> String {
        format!("left: {}px; top: {}px;", self.x, self.y)
    }

    pub fn describe(&self) -> &'static str {
        let third = Self::VIEWPORT / 3.0;
        let col = if self.x < third { 0 } else if self.x < third * 2.0 { 1 } else { 2 };
        let row = if self.y < third { 0 } else if self.y < third * 2.0 { 1 } else { 2 };

        match (row, col) {
            (0, 0) => "top-left",
            (0, 1) => "top-center",
            (0, 2) => "top-right",
            (1, 0) => "center-left",
            (1, 1) => "center",
            (1, 2) => "center-right",
            (2, 0) => "bottom-left",
            (2, 1) => "bottom-center",
            (2, 2) => "bottom-right",
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_css() {
        let p = Position::new(100.0, 200.0);
        assert_eq!(p.to_css(), "left: 100px; top: 200px;");
    }

    #[test]
    fn position_from_fraction() {
        let p = Position::from_fraction(0.5, 0.5);
        assert_eq!(p.x, 512.0);
        assert_eq!(p.y, 512.0);
    }

    #[test]
    fn position_clamp() {
        let p = Position::new(1000.0, 1000.0);
        let clamped = p.clamp_to_viewport(100.0, 100.0);
        assert_eq!(clamped.x, 924.0);
        assert_eq!(clamped.y, 924.0);
    }

    #[test]
    fn position_describe() {
        assert_eq!(Position::CENTER.describe(), "center");
        assert_eq!(Position::new(50.0, 50.0).describe(), "top-left");
        assert_eq!(Position::new(512.0, 50.0).describe(), "top-center");
        assert_eq!(Position::new(900.0, 512.0).describe(), "center-right");
        assert_eq!(Position::new(200.0, 800.0).describe(), "bottom-left");
        assert_eq!(Position::new(512.0, 900.0).describe(), "bottom-center");
    }
}
