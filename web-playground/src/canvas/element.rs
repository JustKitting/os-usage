//! Element renderer - wraps a pool snippet in a transform container
//!
//! Structure:
//!   outer div (position + static transforms + data-* attributes)
//!     animation div (CSS animation - only if animated)
//!       inner div (snippet HTML via dangerous_inner_html)
//!
//! Clicking toggles between html and html_active states.
//! Each element exposes its state via data-* attributes for DOM queries.

use dioxus::prelude::*;
use crate::transform::PlacedElement;

/// Renders a PlacedElement on the canvas
#[component]
pub fn CanvasElement(
    placed: PlacedElement,
    on_click: EventHandler<String>,
) -> Element {
    let wrapper_style = placed.wrapper_style();
    let anim_style = placed.animation_style();
    let html_default = placed.snippet.html.clone();
    let html_active = placed.snippet.html_active.clone();
    let id = placed.snippet.id.clone();
    let kind = placed.snippet.kind.describe().to_string();
    let label = placed.snippet.label.clone();
    let x = format!("{:.1}", placed.position.x);
    let y = format!("{:.1}", placed.position.y);
    let (_, _, bw, bh) = placed.bounds();
    let width = format!("{:.1}", bw);
    let height = format!("{:.1}", bh);
    let scale = format!("{:.2}", placed.scale.value());
    let angle = format!("{:.1}", placed.angle.degrees());
    let opacity = format!("{:.2}", placed.opacity.value());
    let animation = placed.animation.describe();
    let description = placed.describe();
    let has_animation = !placed.animation.is_none();

    let mut is_active = use_signal(|| false);

    let current_html = if *is_active.read() {
        html_active.clone()
    } else {
        html_default.clone()
    };
    let active_str = if *is_active.read() { "true" } else { "false" };

    rsx! {
        div {
            style: "{wrapper_style}",
            cursor: "pointer",
            "data-element-id": "{id}",
            "data-kind": "{kind}",
            "data-label": "{label}",
            "data-x": "{x}",
            "data-y": "{y}",
            "data-width": "{width}",
            "data-height": "{height}",
            "data-scale": "{scale}",
            "data-angle": "{angle}",
            "data-opacity": "{opacity}",
            "data-animation": "{animation}",
            "data-active": "{active_str}",
            "data-description": "{description}",
            onclick: move |_| {
                is_active.toggle();
                on_click(id.clone());
            },
            if has_animation {
                div {
                    style: "{anim_style}",
                    div {
                        dangerous_inner_html: "{current_html}"
                    }
                }
            } else {
                div {
                    dangerous_inner_html: "{current_html}"
                }
            }
        }
    }
}
