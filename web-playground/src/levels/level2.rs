use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::pool::{ElementPool, ElementKind};
use crate::primitives::Position;
use crate::transform::{PlacedElement, Sampler};
use super::{fresh_rng, random_canvas_bg};

fn random_toggle(pool: &ElementPool) -> PlacedElement {
    let mut rng = fresh_rng();
    let kinds = [ElementKind::Toggle, ElementKind::Checkbox];
    let kind = kinds[rng.random_range(0..kinds.len())];
    let snippet = Sampler::pick_kind(&mut rng, pool, kind)
        .expect("pool has toggles/checkboxes");

    let pad = 150.0;
    let x = rng.random_range(pad..(Position::VIEWPORT - pad));
    let y = rng.random_range(pad..(Position::VIEWPORT - pad));
    let pos = Position::new(x, y);

    PlacedElement::new(snippet, pos)
}

#[component]
pub fn Level2() -> Element {
    let pool = use_hook(|| ElementPool::with_builtins());

    let mut placed = use_signal(|| random_toggle(&pool));
    let mut score = use_signal(|| 0u32);
    let mut is_active = use_signal(|| false);
    let mut bg = use_signal(|| random_canvas_bg());

    let current = placed.read();
    let style = current.wrapper_style();
    let html = if *is_active.read() {
        current.snippet.html_active.clone()
    } else {
        current.snippet.html.clone()
    };
    let active_str = if *is_active.read() { "on" } else { "off" };
    let description = format!("{}, state: {}", current.describe(), active_str);
    let (bx, by, bw, bh) = current.bounds();
    let target_text = super::ground_truth::strip_tags(&html).trim().to_string();
    let steps = format!(r#"[{{"action":"click","target":"{}"}}]"#, target_text);
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
                    "Level 2"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Toggle the switch"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "width: 1024px; height: 1024px; background: {bg}; position: relative; border: 1px solid #2a2a4a; overflow: hidden; transition: background 0.4s;",

                div {
                    class: "target",
                    style: "{style}",
                    cursor: "pointer",
                    onclick: move |_| {
                        is_active.toggle();
                        score.set(score() + 1);
                        placed.set(random_toggle(&pool_click));
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
                steps: steps,
            }
        }
    }
}
