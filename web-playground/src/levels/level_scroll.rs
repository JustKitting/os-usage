use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::pool::{ElementPool, ElementKind};
use crate::primitives::{Position, viewport_size};
use crate::transform::{PlacedElement, Sampler};
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg};

/// Place a button guaranteed to be at least partially off-screen so the user
/// must scroll the viewport to find it.
fn random_offscreen_element(pool: &ElementPool) -> PlacedElement {
    let mut rng = fresh_rng();
    let snippet = Sampler::pick_kind(&mut rng, pool, ElementKind::Button)
        .expect("pool has buttons");

    let (vp_w, vp_h) = viewport_size();
    let w = snippet.approx_width;
    let h = snippet.approx_height;
    let canvas_w = vp_w * 1.5;
    let canvas_h = vp_h * 1.5;
    let pad = 40.0;

    // Pick a position in the extended canvas that is at least partially
    // outside the visible viewport (x + w > vp_w  OR  y + h > vp_h).
    // Strategy: choose which axis overflows, then place accordingly.
    let overflow_axis = rng.random_range(0..3u8); // 0=right, 1=bottom, 2=both
    let (x, y) = match overflow_axis {
        0 => {
            // Off the right edge: x is in [vp_w - w/2, canvas_w - w]
            let min_x = (vp_w - w * 0.5).max(0.0);
            let max_x = (canvas_w - w).max(min_x);
            let x = rng.random_range(min_x..max_x.max(min_x + 1.0));
            let y = rng.random_range(pad..(vp_h - h - pad).max(pad + 1.0));
            (x, y)
        }
        1 => {
            // Off the bottom edge: y is in [vp_h - h/2, canvas_h - h]
            let x = rng.random_range(pad..(vp_w - w - pad).max(pad + 1.0));
            let min_y = (vp_h - h * 0.5).max(0.0);
            let max_y = (canvas_h - h).max(min_y);
            let y = rng.random_range(min_y..max_y.max(min_y + 1.0));
            (x, y)
        }
        _ => {
            // Off both edges
            let min_x = (vp_w - w * 0.5).max(0.0);
            let max_x = (canvas_w - w).max(min_x);
            let x = rng.random_range(min_x..max_x.max(min_x + 1.0));
            let min_y = (vp_h - h * 0.5).max(0.0);
            let max_y = (canvas_h - h).max(min_y);
            let y = rng.random_range(min_y..max_y.max(min_y + 1.0));
            (x, y)
        }
    };

    PlacedElement::new(snippet, Position::new(x, y))
}

#[component]
pub fn LevelScroll() -> Element {
    let pool = use_hook(|| ElementPool::with_builtins());

    let mut placed = use_signal(|| random_offscreen_element(&pool));
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());

    let current = placed.read();
    let style = current.wrapper_style();
    let html = current.snippet.html.clone();
    let (bx, by, bw, bh) = current.bounds();
    let target_text = super::ground_truth::strip_tags(&html).trim().to_string();

    let (vp_w, vp_h) = viewport_size();
    let canvas_w = vp_w * 1.5;
    let canvas_h = vp_h * 1.5;
    let viewport_style = super::viewport_style(&bg(), true);

    // Ground truth: scroll to target, then click
    let tree = ui_node::target_button(&target_text, Rect::new(bx, by, bw, bh));
    drop(current);

    let pool_click = pool.clone();

    rsx! {
        div {
            style: "min-height: 100vh; background: #0f0f1a; display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: system-ui, sans-serif;",

            div {
                style: "display: flex; gap: 16px; align-items: center; margin-bottom: 16px;",
                Link {
                    to: Route::LevelSelect {},
                    style: "color: #6b7280; text-decoration: none; font-size: 14px;",
                    "\u{2190} Levels"
                }
                h2 {
                    style: "color: #e5e7eb; margin: 0; font-size: 20px;",
                    "Scroll & Click"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Scroll to find the button, then click it"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "{viewport_style}",

                // Invisible spacer that makes the scrollable area match the
                // extended canvas size.
                div {
                    style: "position: absolute; left: 0; top: 0; width: {canvas_w}px; height: {canvas_h}px; pointer-events: none;",
                }

                div {
                    class: "target",
                    style: "{style}",
                    cursor: "pointer",
                    onclick: move |_| {
                        placed.set(random_offscreen_element(&pool_click));
                        score.set(score() + 1);
                        bg.set(random_canvas_bg());
                        // Reset scroll position for next round
                        document::eval("document.getElementById('viewport')?.scrollTo(0, 0)");
                    },
                    div {
                        dangerous_inner_html: "{html}"
                    }
                }
            }

            super::GroundTruth {
                description: String::new(),
                target_x: bx,
                target_y: by,
                target_w: bw,
                target_h: bh,
                tree: Some(tree.clone()),
            }
        }
    }
}
