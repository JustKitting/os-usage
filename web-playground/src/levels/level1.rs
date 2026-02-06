use dioxus::prelude::*;

use crate::Route;
use crate::pool::{ElementPool, ElementKind};
use crate::ui_node::{self, Rect};
use super::{random_element, random_canvas_bg};

#[component]
pub fn Level1() -> Element {
    let pool = use_hook(|| ElementPool::with_builtins());

    let mut placed = use_signal(|| random_element(&pool, ElementKind::Button));
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());

    let current = placed.read();
    let style = current.wrapper_style();
    let html = current.snippet.html.clone();
    let description = current.describe();
    let (bx, by, bw, bh) = current.bounds();
    let target_text = super::ground_truth::strip_tags(&html).trim().to_string();
    let viewport_style = super::viewport_style(&bg(), false);

    // Build UINode tree for ground truth
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
                    "Level 1"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Click the button"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "{viewport_style}",

                div {
                    class: "target",
                    style: "{style}",
                    cursor: "pointer",
                    onclick: move |_| {
                        placed.set(random_element(&pool_click, ElementKind::Button));
                        score.set(score() + 1);
                        bg.set(random_canvas_bg());
                    },
                    div {
                        dangerous_inner_html: "{html}"
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: bx,
                target_y: by,
                target_w: bw,
                target_h: bh,
                tree: Some(tree.clone()),
            }
        }
    }
}
