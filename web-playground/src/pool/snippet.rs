//! DesignSnippet - a concrete HTML+CSS element from the pool

use super::kind::ElementKind;

/// A single design variant from the pool
///
/// Each snippet has two HTML states: default and active (clicked/checked/toggled).
/// Clicking swaps between them. Elements without meaningful active states
/// (like text or inputs) can set html_active = html for no visual change,
/// or provide a subtle feedback state.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignSnippet {
    /// Unique identifier
    pub id: String,
    /// What kind of element this is
    pub kind: ElementKind,
    /// Human-readable label (e.g. "material primary button", "bootstrap search input")
    pub label: String,
    /// Default state HTML+CSS
    pub html: String,
    /// Active/clicked state HTML+CSS
    pub html_active: String,
    /// Approximate width in px (for layout/collision avoidance)
    pub approx_width: f32,
    /// Approximate height in px
    pub approx_height: f32,
}

impl DesignSnippet {
    pub fn new(
        id: impl Into<String>,
        kind: ElementKind,
        label: impl Into<String>,
        html: impl Into<String>,
        html_active: impl Into<String>,
        approx_width: f32,
        approx_height: f32,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            html: html.into(),
            html_active: html_active.into(),
            approx_width,
            approx_height,
        }
    }

    /// Convenience: same HTML for both states (no visual change on click)
    pub fn static_new(
        id: impl Into<String>,
        kind: ElementKind,
        label: impl Into<String>,
        html: impl Into<String>,
        approx_width: f32,
        approx_height: f32,
    ) -> Self {
        let html_str: String = html.into();
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            html: html_str.clone(),
            html_active: html_str,
            approx_width,
            approx_height,
        }
    }

    pub fn describe(&self) -> String {
        format!("{} ({})", self.label, self.kind.describe())
    }
}
