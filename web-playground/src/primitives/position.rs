//! Position - element placement with CSS output
//!
//! Positions are absolute within the dynamically-sized viewport.

use std::cell::Cell;

/// Current viewport size in pixels (width, height). Read from `window.__vpW`
/// and `window.__vpH` (set by autoFit JS) with a fallback estimate from window
/// dimensions.
pub fn viewport_size() -> (f32, f32) {
    VP_W.with(|cw| {
        VP_H.with(|ch| {
            let cached_w = cw.get();
            let cached_h = ch.get();
            if cached_w > 0.0 && cached_h > 0.0 {
                // Check if JS has newer values
                if let Some((jw, jh)) = read_js_vp_size() {
                    if (jw - cached_w).abs() > 1.0 || (jh - cached_h).abs() > 1.0 {
                        cw.set(jw);
                        ch.set(jh);
                        return (jw, jh);
                    }
                }
                return (cached_w, cached_h);
            }
            let (w, h) = read_js_vp_size().unwrap_or_else(estimate_viewport_size);
            cw.set(w);
            ch.set(h);
            (w, h)
        })
    })
}

/// Force-refresh the cached viewport size from JS on next read.
pub fn invalidate_viewport_cache() {
    VP_W.with(|c| c.set(0.0));
    VP_H.with(|c| c.set(0.0));
}

thread_local! {
    static VP_W: Cell<f32> = const { Cell::new(0.0) };
    static VP_H: Cell<f32> = const { Cell::new(0.0) };
}

fn read_js_vp_size() -> Option<(f32, f32)> {
    #[cfg(not(target_arch = "wasm32"))]
    { return None; }

    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let w_val = js_sys::Reflect::get(&window, &web_sys::wasm_bindgen::JsValue::from_str("__vpW")).ok()?;
        let h_val = js_sys::Reflect::get(&window, &web_sys::wasm_bindgen::JsValue::from_str("__vpH")).ok()?;
        let w = w_val.as_f64()? as f32;
        let h = h_val.as_f64()? as f32;
        Some((w, h))
    }
}

fn estimate_viewport_size() -> (f32, f32) {
    #[cfg(not(target_arch = "wasm32"))]
    { return (1024.0, 768.0); }

    #[cfg(target_arch = "wasm32")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return (1024.0, 768.0),
        };
        let w = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1024.0) as f32;
        let h = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(768.0) as f32;
        // Conservative: leave room for header (~60px) + padding (40px)
        let vp_w = (w - 40.0).max(200.0).floor();
        let vp_h = (h - 100.0).max(150.0).floor();
        (vp_w, vp_h)
    }
}

/// Position in pixels, absolute within the viewport
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0 };

    /// Center of the current viewport.
    pub fn center() -> Self {
        let (vp_w, vp_h) = viewport_size();
        Self { x: vp_w / 2.0, y: vp_h / 2.0 }
    }

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create from percentage of viewport (0.0 - 1.0)
    pub fn from_fraction(fx: f32, fy: f32) -> Self {
        let (vp_w, vp_h) = viewport_size();
        Self {
            x: fx * vp_w,
            y: fy * vp_h,
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
        let (vp_w, vp_h) = viewport_size();
        Self {
            x: self.x.clamp(0.0, vp_w - elem_width),
            y: self.y.clamp(0.0, vp_h - elem_height),
        }
    }

    pub fn to_css(&self) -> String {
        format!("left: {}px; top: {}px;", self.x, self.y)
    }

    pub fn describe(&self) -> &'static str {
        let (vp_w, vp_h) = viewport_size();
        let third_x = vp_w / 3.0;
        let third_y = vp_h / 3.0;
        let col = if self.x < third_x { 0 } else if self.x < third_x * 2.0 { 1 } else { 2 };
        let row = if self.y < third_y { 0 } else if self.y < third_y * 2.0 { 1 } else { 2 };

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
        // In test (non-WASM), viewport_size() falls back to (1024.0, 768.0)
        let p = Position::from_fraction(0.5, 0.5);
        assert_eq!(p.x, 512.0);
        assert_eq!(p.y, 384.0);
    }

    #[test]
    fn position_clamp() {
        let p = Position::new(1000.0, 1000.0);
        let clamped = p.clamp_to_viewport(100.0, 100.0);
        assert_eq!(clamped.x, 924.0);
        assert_eq!(clamped.y, 668.0);
    }

    #[test]
    fn position_describe() {
        // With (1024, 768) fallback: third_x = 341.33, third_y = 256.0
        assert_eq!(Position::new(512.0, 384.0).describe(), "center");
        assert_eq!(Position::new(50.0, 50.0).describe(), "top-left");
        assert_eq!(Position::new(512.0, 50.0).describe(), "top-center");
        assert_eq!(Position::new(900.0, 384.0).describe(), "center-right");
        assert_eq!(Position::new(200.0, 600.0).describe(), "bottom-left");
        assert_eq!(Position::new(512.0, 700.0).describe(), "bottom-center");
    }
}
