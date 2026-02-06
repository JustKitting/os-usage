//! Animation - CSS keyframe animations applied to elements
//!
//! Animations run on a separate wrapper div from static transforms
//! to avoid CSS transform property conflicts.
//!
//! Drift and bounce distances are parameterized via CSS custom properties
//! so elements can move anywhere from small wiggles to full canvas traversals.

/// Direction of drift movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriftDirection {
    Right,
    Left,
    Up,
    Down,
}

impl DriftDirection {
    fn keyframe_name(&self) -> &'static str {
        match self {
            Self::Right => "drift-right",
            Self::Left => "drift-left",
            Self::Up => "drift-up",
            Self::Down => "drift-down",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Right => "right",
            Self::Left => "left",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// Animation speed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationSpeed {
    Slow,
    Normal,
    Fast,
}

impl AnimationSpeed {
    pub fn duration(&self) -> &'static str {
        match self {
            Self::Slow => "4s",
            Self::Normal => "2s",
            Self::Fast => "0.8s",
        }
    }

    fn describe(&self) -> &'static str {
        match self {
            Self::Slow => "slowly",
            Self::Normal => "",
            Self::Fast => "quickly",
        }
    }
}

/// CSS keyframe animation applied to an element
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Animation {
    None,
    Drift { direction: DriftDirection, speed: AnimationSpeed, distance: f32 },
    Pulse { speed: AnimationSpeed },
    Fade { speed: AnimationSpeed },
    Spin { speed: AnimationSpeed },
    Bounce { speed: AnimationSpeed, height: f32 },
    Shake { speed: AnimationSpeed },
}

impl Animation {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// CSS animation property for the animation wrapper div
    pub fn to_css(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Drift { direction, speed, distance } => {
                format!(
                    "--anim-dist: {}px; animation: {} {} infinite alternate ease-in-out;",
                    distance,
                    direction.keyframe_name(),
                    speed.duration(),
                )
            }
            Self::Pulse { speed } => {
                format!("animation: pulse {} infinite alternate ease-in-out;", speed.duration())
            }
            Self::Fade { speed } => {
                format!("animation: fade {} infinite alternate ease-in-out;", speed.duration())
            }
            Self::Spin { speed } => {
                format!("animation: spin {} infinite linear;", speed.duration())
            }
            Self::Bounce { speed, height } => {
                format!(
                    "--anim-dist: {}px; animation: bounce {} infinite alternate ease-in-out;",
                    height,
                    speed.duration(),
                )
            }
            Self::Shake { speed } => {
                format!("animation: shake {} infinite linear;", speed.duration())
            }
        }
    }

    /// Ground truth description
    pub fn describe(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Drift { direction, speed, distance } => {
                let verb = if *distance > 300.0 { "sweeping" } else { "drifting" };
                let dir = direction.name();
                let spd = speed.describe();
                if spd.is_empty() {
                    format!("{verb} {dir}")
                } else {
                    format!("{verb} {dir} {spd}")
                }
            }
            Self::Pulse { speed } => {
                let spd = speed.describe();
                if spd.is_empty() { "pulsing".into() } else { format!("pulsing {spd}") }
            }
            Self::Fade { speed } => {
                let spd = speed.describe();
                if spd.is_empty() { "fading".into() } else { format!("fading {spd}") }
            }
            Self::Spin { speed } => {
                let spd = speed.describe();
                if spd.is_empty() { "spinning".into() } else { format!("spinning {spd}") }
            }
            Self::Bounce { speed, height } => {
                let verb = if *height > 100.0 { "leaping" } else { "bouncing" };
                let spd = speed.describe();
                if spd.is_empty() { verb.into() } else { format!("{verb} {spd}") }
            }
            Self::Shake { speed } => {
                let spd = speed.describe();
                if spd.is_empty() { "shaking".into() } else { format!("shaking {spd}") }
            }
        }
    }

    /// All @keyframes definitions - inject once as a <style> block
    pub fn keyframes_css() -> &'static str {
        r#"
@keyframes drift-right { 0%,100% { transform: translateX(0); } 50% { transform: translateX(var(--anim-dist)); } }
@keyframes drift-left  { 0%,100% { transform: translateX(0); } 50% { transform: translateX(calc(-1 * var(--anim-dist))); } }
@keyframes drift-up    { 0%,100% { transform: translateY(0); } 50% { transform: translateY(calc(-1 * var(--anim-dist))); } }
@keyframes drift-down  { 0%,100% { transform: translateY(0); } 50% { transform: translateY(var(--anim-dist)); } }
@keyframes pulse       { 0%,100% { transform: scale(1); } 50% { transform: scale(1.15); } }
@keyframes fade        { 0%,100% { opacity: 1; } 50% { opacity: 0.3; } }
@keyframes spin        { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
@keyframes bounce      { 0%,100% { transform: translateY(0); } 50% { transform: translateY(calc(-1 * var(--anim-dist))); } }
@keyframes shake       { 0%,100% { transform: translateX(0); } 25% { transform: translateX(-5px); } 75% { transform: translateX(5px); } }
@keyframes bg-shift {
  0%   { background-color: #0f172a; }
  8%   { background-color: #1e3a5f; }
  16%  { background-color: #2563eb; }
  24%  { background-color: #1e3a5f; }
  32%  { background-color: #0f172a; }
  40%  { background-color: #1a1a2e; }
  48%  { background-color: #4c1d95; }
  56%  { background-color: #7c3aed; }
  64%  { background-color: #4c1d95; }
  72%  { background-color: #0f172a; }
  80%  { background-color: #164e63; }
  88%  { background-color: #06b6d4; }
  94%  { background-color: #164e63; }
  100% { background-color: #0f172a; }
}
"#
    }

    /// Vocabulary for random sampling - weighted toward None so most elements are static
    pub const VOCABULARY: &[Self] = &[
        // Static
        Self::None,
        Self::None,
        Self::None,
        // Small drifts (subtle)
        Self::Drift { direction: DriftDirection::Right, speed: AnimationSpeed::Normal, distance: 40.0 },
        Self::Drift { direction: DriftDirection::Left, speed: AnimationSpeed::Normal, distance: 40.0 },
        Self::Drift { direction: DriftDirection::Up, speed: AnimationSpeed::Slow, distance: 40.0 },
        Self::Drift { direction: DriftDirection::Down, speed: AnimationSpeed::Slow, distance: 40.0 },
        // Medium drifts
        Self::Drift { direction: DriftDirection::Right, speed: AnimationSpeed::Slow, distance: 200.0 },
        Self::Drift { direction: DriftDirection::Left, speed: AnimationSpeed::Normal, distance: 150.0 },
        // Large drifts - traverse significant portion of canvas
        Self::Drift { direction: DriftDirection::Right, speed: AnimationSpeed::Slow, distance: 500.0 },
        Self::Drift { direction: DriftDirection::Down, speed: AnimationSpeed::Slow, distance: 400.0 },
        Self::Drift { direction: DriftDirection::Left, speed: AnimationSpeed::Slow, distance: 600.0 },
        // Extreme - nearly full canvas
        Self::Drift { direction: DriftDirection::Right, speed: AnimationSpeed::Slow, distance: 800.0 },
        Self::Drift { direction: DriftDirection::Up, speed: AnimationSpeed::Slow, distance: 700.0 },
        // Pulse / Fade / Spin
        Self::Pulse { speed: AnimationSpeed::Normal },
        Self::Pulse { speed: AnimationSpeed::Slow },
        Self::Fade { speed: AnimationSpeed::Normal },
        Self::Fade { speed: AnimationSpeed::Slow },
        Self::Spin { speed: AnimationSpeed::Slow },
        // Small bounces
        Self::Bounce { speed: AnimationSpeed::Normal, height: 20.0 },
        Self::Bounce { speed: AnimationSpeed::Fast, height: 20.0 },
        // Large bounces
        Self::Bounce { speed: AnimationSpeed::Slow, height: 150.0 },
        Self::Bounce { speed: AnimationSpeed::Normal, height: 300.0 },
        // Shake
        Self::Shake { speed: AnimationSpeed::Fast },
    ];
}

impl Default for Animation {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_produces_empty_css() {
        assert_eq!(Animation::None.to_css(), "");
    }

    #[test]
    fn pulse_css() {
        let anim = Animation::Pulse { speed: AnimationSpeed::Normal };
        let css = anim.to_css();
        assert!(css.contains("animation: pulse 2s"));
    }

    #[test]
    fn drift_css_with_distance() {
        let anim = Animation::Drift {
            direction: DriftDirection::Right,
            speed: AnimationSpeed::Slow,
            distance: 500.0,
        };
        let css = anim.to_css();
        assert!(css.contains("--anim-dist: 500px"));
        assert!(css.contains("drift-right"));
        assert!(css.contains("4s"));
    }

    #[test]
    fn bounce_css_with_height() {
        let anim = Animation::Bounce { speed: AnimationSpeed::Normal, height: 300.0 };
        let css = anim.to_css();
        assert!(css.contains("--anim-dist: 300px"));
        assert!(css.contains("bounce"));
    }

    #[test]
    fn describe_none_is_empty() {
        assert_eq!(Animation::None.describe(), "");
    }

    #[test]
    fn describe_large_drift_uses_sweeping() {
        let anim = Animation::Drift {
            direction: DriftDirection::Right,
            speed: AnimationSpeed::Slow,
            distance: 500.0,
        };
        assert_eq!(anim.describe(), "sweeping right slowly");
    }

    #[test]
    fn describe_small_drift_uses_drifting() {
        let anim = Animation::Drift {
            direction: DriftDirection::Left,
            speed: AnimationSpeed::Normal,
            distance: 40.0,
        };
        assert_eq!(anim.describe(), "drifting left");
    }

    #[test]
    fn describe_large_bounce_uses_leaping() {
        let anim = Animation::Bounce { speed: AnimationSpeed::Slow, height: 200.0 };
        assert_eq!(anim.describe(), "leaping slowly");
    }

    #[test]
    fn vocabulary_has_none() {
        assert!(Animation::VOCABULARY.iter().any(|a| a.is_none()));
    }

    #[test]
    fn keyframes_uses_css_custom_properties() {
        let kf = Animation::keyframes_css();
        assert!(kf.contains("var(--anim-dist)"));
    }
}
