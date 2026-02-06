//! PlacedElement - a snippet with transforms applied on the canvas

use crate::pool::DesignSnippet;
use crate::primitives::{Angle, Animation, Opacity, Position, Scale};

/// A snippet placed on the canvas with transforms
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedElement {
    pub snippet: DesignSnippet,
    pub position: Position,
    pub scale: Scale,
    pub angle: Angle,
    pub opacity: Opacity,
    pub animation: Animation,
}

impl PlacedElement {
    pub fn new(snippet: DesignSnippet, position: Position) -> Self {
        Self {
            snippet,
            position,
            scale: Scale::NORMAL,
            angle: Angle::ZERO,
            opacity: Opacity::FULL,
            animation: Animation::None,
        }
    }

    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_angle(mut self, angle: Angle) -> Self {
        self.angle = angle;
        self
    }

    pub fn with_opacity(mut self, opacity: Opacity) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        self
    }

    /// CSS style for the outer wrapper div (position + static transforms)
    pub fn wrapper_style(&self) -> String {
        let mut parts = vec![
            "position: absolute".to_string(),
            self.position.to_css(),
            "transform-origin: center center".to_string(),
        ];

        // Build transform chain
        let mut transforms = Vec::new();
        let scale_css = self.scale.to_css();
        let angle_css = self.angle.to_css();
        if !scale_css.is_empty() {
            transforms.push(scale_css);
        }
        if !angle_css.is_empty() {
            transforms.push(angle_css);
        }
        if !transforms.is_empty() {
            parts.push(format!("transform: {}", transforms.join(" ")));
        }

        // Opacity
        let opacity_css = self.opacity.to_css();
        if !opacity_css.is_empty() {
            parts.push(opacity_css.trim_end_matches(';').to_string());
        }

        parts.join("; ") + ";"
    }

    /// CSS style for the animation wrapper div (between outer and snippet)
    pub fn animation_style(&self) -> String {
        self.animation.to_css()
    }

    /// Ground truth description for training labels
    pub fn describe(&self) -> String {
        let mut desc = self.snippet.describe();

        let mut modifiers: Vec<String> = Vec::new();

        if self.scale.value() != 1.0 {
            modifiers.push(self.scale.describe().to_string());
        }
        if self.angle.degrees().abs() > 0.01 {
            modifiers.push(self.angle.describe().to_string());
        }
        if self.opacity.value() < 1.0 {
            modifiers.push(self.opacity.describe().to_string());
        }
        let anim_desc = self.animation.describe();
        if !anim_desc.is_empty() {
            modifiers.push(anim_desc);
        }

        if !modifiers.is_empty() {
            desc.push_str(&format!(", {}", modifiers.join(", ")));
        }

        desc.push_str(&format!(", at {}", self.position.describe()));
        desc
    }

    /// Bounding box estimate (for collision detection and ground truth)
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let w = self.snippet.approx_width * self.scale.value();
        let h = self.snippet.approx_height * self.scale.value();
        (self.position.x, self.position.y, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::ElementKind;
    use crate::primitives::animation::AnimationSpeed;

    fn test_snippet() -> DesignSnippet {
        DesignSnippet::new(
            "test-btn",
            ElementKind::Button,
            "test button",
            "<button>Test</button>",
            "<button>Test</button>",
            100.0,
            40.0,
        )
    }

    #[test]
    fn wrapper_style_default() {
        let placed = PlacedElement::new(test_snippet(), Position::new(100.0, 200.0));
        let style = placed.wrapper_style();
        assert!(style.contains("left: 100px"));
        assert!(style.contains("top: 200px"));
        assert!(!style.contains("transform:"));
    }

    #[test]
    fn wrapper_style_with_transforms() {
        let placed = PlacedElement::new(test_snippet(), Position::new(50.0, 50.0))
            .with_scale(Scale::DOUBLE)
            .with_angle(Angle::new(45.0));
        let style = placed.wrapper_style();
        assert!(style.contains("scale(2.00)"));
        assert!(style.contains("rotate(45deg)"));
    }

    #[test]
    fn animation_style_none() {
        let placed = PlacedElement::new(test_snippet(), Position::center());
        assert_eq!(placed.animation_style(), "");
    }

    #[test]
    fn animation_style_pulse() {
        let placed = PlacedElement::new(test_snippet(), Position::center())
            .with_animation(Animation::Pulse { speed: AnimationSpeed::Normal });
        let style = placed.animation_style();
        assert!(style.contains("animation: pulse 2s"));
    }

    #[test]
    fn describe_includes_animation() {
        let placed = PlacedElement::new(test_snippet(), Position::center())
            .with_animation(Animation::Bounce { speed: AnimationSpeed::Fast, height: 20.0 });
        let desc = placed.describe();
        assert!(desc.contains("bouncing quickly"));
    }

    #[test]
    fn describe_no_animation_when_none() {
        let placed = PlacedElement::new(test_snippet(), Position::center());
        let desc = placed.describe();
        assert!(!desc.contains("pulsing"));
        assert!(!desc.contains("bouncing"));
    }
}
