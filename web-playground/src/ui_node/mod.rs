//! UINode — typed UI element tree for composable ground truth generation.
//!
//! Each level constructs a UINode tree describing its interactive elements.
//! Resolving the tree produces description, action steps, and a VLM thinking
//! chain — replacing hand-written ground truth strings.

mod builder;
mod check;
mod prism;
mod resolve;

pub use builder::*;
pub use check::Completion;
pub use resolve::ResolvedGroundTruth;

use crate::primitives::Position;

// ── Rect ────────────────────────────────────────────────────────────────

/// Axis-aligned bounding box in viewport-pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    /// Region name like "top-left", "center", etc.
    pub fn region(&self) -> &'static str {
        let (cx, cy) = self.center();
        Position::new(cx, cy).describe()
    }

    /// Full coordinate description: "near the top-left (120,200 80x40)"
    pub fn describe(&self) -> String {
        format!(
            "near the {} ({},{} {}x{})",
            self.region(),
            self.x as i32, self.y as i32,
            self.w as i32, self.h as i32,
        )
    }

    /// Region name relative to a parent rect's bounds.
    pub fn region_within(&self, parent: &Rect) -> &'static str {
        let (cx, cy) = self.center();
        // Normalize to 0..1 within the parent
        let rx = if parent.w > 0.0 { (cx - parent.x) / parent.w } else { 0.5 };
        let ry = if parent.h > 0.0 { (cy - parent.y) / parent.h } else { 0.5 };
        let col = if rx < 1.0 / 3.0 { 0 } else if rx < 2.0 / 3.0 { 1 } else { 2 };
        let row = if ry < 1.0 / 3.0 { 0 } else if ry < 2.0 / 3.0 { 1 } else { 2 };
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

    /// Coordinate description relative to a named parent:
    /// "near the top-left of the card (120,200 80x40)"
    pub fn describe_within(&self, parent: &Rect, parent_label: &str) -> String {
        format!(
            "near the {} of the {} ({},{} {}x{})",
            self.region_within(parent),
            parent_label,
            self.x as i32, self.y as i32,
            self.w as i32, self.h as i32,
        )
    }

    /// Apply a viewport transform to get window-space pixel coordinates.
    pub fn to_window(&self, vt: &ViewportTransform) -> (i32, i32, i32, i32) {
        vt.apply(self)
    }

    pub fn offset(&self, parent_x: f32, parent_y: f32) -> Self {
        Self {
            x: self.x + parent_x,
            y: self.y + parent_y,
            w: self.w,
            h: self.h,
        }
    }
}

// ── ViewportTransform ────────────────────────────────────────────────────

/// Maps viewport-local pixel coordinates to window-space pixel coordinates.
#[derive(Debug, Clone, Copy)]
pub struct ViewportTransform {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
}

impl ViewportTransform {
    /// Identity transform — viewport coords pass through unchanged.
    pub fn identity() -> Self {
        Self { offset_x: 0.0, offset_y: 0.0, scale: 1.0 }
    }

    /// Build from the DOM viewport bbox [x, y, width, height].
    /// Coordinates in UINode Rects are in viewport-pixel space, so scale
    /// is the ratio of DOM width to internal coordinate width (~1.0).
    pub fn from_viewport(vp: &[f64; 4]) -> Self {
        let (vp_w, _vp_h) = crate::primitives::viewport_size();
        Self {
            offset_x: vp[0] as f32,
            offset_y: vp[1] as f32,
            scale: if vp_w > 0.0 { vp[2] as f32 / vp_w } else { 1.0 },
        }
    }

    /// Convert a viewport-space Rect to window-space (x, y, w, h).
    pub fn apply(&self, rect: &Rect) -> (i32, i32, i32, i32) {
        (
            (self.offset_x + rect.x * self.scale) as i32,
            (self.offset_y + rect.y * self.scale) as i32,
            (rect.w * self.scale) as i32,
            (rect.h * self.scale) as i32,
        )
    }
}

// ── Action ──────────────────────────────────────────────────────────────

/// A single solver action primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Click { target: String },
    Type { target: String, value: String },
    Drag { from: String, to: String },
    RightClick { target: String },
    Scroll { target: String },
}

impl Action {
    pub fn click(target: impl Into<String>) -> Self {
        Self::Click { target: target.into() }
    }

    pub fn type_text(target: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Type { target: target.into(), value: value.into() }
    }

    pub fn drag(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Drag { from: from.into(), to: to.into() }
    }

    pub fn right_click(target: impl Into<String>) -> Self {
        Self::RightClick { target: target.into() }
    }

    pub fn scroll(target: impl Into<String>) -> Self {
        Self::Scroll { target: target.into() }
    }

    /// Serialize to the JSON format expected by the solver.
    pub fn to_json(&self) -> String {
        match self {
            Self::Click { target } => {
                format!(r#"{{"action":"click","target":"{}"}}"#, escape_json(target))
            }
            Self::Type { target, value } => {
                format!(
                    r#"{{"action":"type","target":"{}","value":"{}"}}"#,
                    escape_json(target),
                    escape_json(value),
                )
            }
            Self::Drag { from, to } => {
                format!(
                    r#"{{"action":"drag","from":"{}","to":"{}"}}"#,
                    escape_json(from),
                    escape_json(to),
                )
            }
            Self::RightClick { target } => {
                format!(r#"{{"action":"right_click","target":"{}"}}"#, escape_json(target))
            }
            Self::Scroll { target } => {
                format!(r#"{{"action":"scroll","target":"{}"}}"#, escape_json(target))
            }
        }
    }
}

/// Serialize a Vec<Action> to a JSON array string.
pub fn actions_to_json(actions: &[Action]) -> String {
    let inner: Vec<String> = actions.iter().map(|a| a.to_json()).collect();
    format!("[{}]", inner.join(","))
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// ── Visual ──────────────────────────────────────────────────────────────

/// Shared visual properties embedded in every UINode variant.
#[derive(Debug, Clone, PartialEq)]
pub struct Visual {
    pub label: String,
    pub rect: Rect,
    pub color: Option<String>,
    pub is_target: bool,
}

impl Visual {
    pub fn new(label: impl Into<String>, rect: Rect) -> Self {
        Self {
            label: label.into(),
            rect,
            color: None,
            is_target: false,
        }
    }

    pub fn target(mut self) -> Self {
        self.is_target = true;
        self
    }

    pub fn color(mut self, c: impl Into<String>) -> Self {
        self.color = Some(c.into());
        self
    }
}

// ── State structs ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ToggleState {
    pub is_on: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckState {
    pub is_checked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagState {
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToastState {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StarState {
    pub current: usize,
    pub target: usize,
    pub max: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputState {
    pub placeholder: String,
    pub current_value: String,
    pub target_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliderState {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub current_val: i32,
    pub target_val: i32,
    /// Bounding box of the thumb at current position (drag-from).
    pub thumb_rect: Rect,
    /// Bounding box of the thumb at target position (drag-to).
    pub target_thumb_rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DropdownState {
    pub options: Vec<String>,
    pub selected: Option<String>,
    pub target_option: String,
    pub trigger_label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuState {
    pub items: Vec<String>,
    pub target_item: String,
    pub trigger_label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepperState {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub current_val: i32,
    pub target_val: i32,
    pub minus_label: String,
    pub plus_label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadioState {
    pub options: Vec<String>,
    pub selected: Option<usize>,
    pub target_option: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormState {
    pub submit_label: String,
    pub cancel_label: Option<String>,
}

// ── UINode ──────────────────────────────────────────────────────────────

/// A node in the UI description tree.
#[derive(Debug, Clone, PartialEq)]
pub enum UINode {
    // Simple click targets
    Button(Visual),
    Toggle(Visual, ToggleState),
    Checkbox(Visual, CheckState),
    Tab(Visual),
    Accordion(Visual),
    Tag(Visual, TagState),
    Toast(Visual, ToastState),
    Star(Visual, StarState),
    ModalButton(Visual),

    // Text input
    TextInput(Visual, InputState),

    // Drag
    Slider(Visual, SliderState),
    DragSource(Visual),
    DropZone(Visual),

    // Composite (multi-step)
    Dropdown(Visual, DropdownState),
    ContextMenu(Visual, ContextMenuState),
    Stepper(Visual, StepperState),
    RadioGroup(Visual, RadioState),

    // Containers
    Card(Visual, Vec<UINode>),
    Form(Visual, FormState, Vec<UINode>),
}
