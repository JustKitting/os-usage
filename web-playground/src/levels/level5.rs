use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg};

const BUTTON_LABELS: &[&str] = &[
    "Submit", "Cancel", "Confirm", "Delete", "Save", "Edit", "Next",
    "Back", "Close", "Open", "Send", "Reset", "Apply", "Continue",
    "Skip", "Retry", "Accept", "Decline", "Update", "Remove",
    "Add", "Create", "Sign in", "Log out", "Upload", "Download",
    "Share", "Print", "Copy", "Paste", "Refresh", "Search",
];

const BUTTON_COLORS: &[(&str, &str)] = &[
    ("#3b82f6", "#1d4ed8"),
    ("#22c55e", "#16a34a"),
    ("#ef4444", "#b91c1c"),
    ("#8b5cf6", "#6d28d9"),
    ("#f59e0b", "#d97706"),
    ("#6366f1", "#4338ca"),
    ("#ec4899", "#be185d"),
];

struct Level5State {
    target: String,
    labels: Vec<String>,
    colors: Vec<usize>,
    x: f32,
    y: f32,
}

fn random_level5() -> Level5State {
    let mut rng = fresh_rng();
    let btn_count = rng.random_range(3..=5usize);

    let mut indices: Vec<usize> = (0..BUTTON_LABELS.len()).collect();
    let mut labels = Vec::with_capacity(btn_count);
    for _ in 0..btn_count {
        let i = rng.random_range(0..indices.len());
        labels.push(BUTTON_LABELS[indices.remove(i)].to_string());
    }

    let colors: Vec<usize> = (0..btn_count)
        .map(|_| rng.random_range(0..BUTTON_COLORS.len()))
        .collect();

    let target_idx = rng.random_range(0..labels.len());
    let target = labels[target_idx].clone();

    let card_w = 320.0;
    let card_h = 70.0 + (btn_count as f32 * 48.0);
    let pad = 80.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, pad, vp_w * 1.3, vp_h * 1.3);

    Level5State { target, labels, colors, x, y }
}

#[component]
pub fn Level5() -> Element {
    let mut state = use_signal(|| random_level5());
    let mut score = use_signal(|| 0u32);
    let mut wrong_idx = use_signal(|| None::<usize>);
    let mut bg = use_signal(|| random_canvas_bg());

    let st = state.read();
    let target = st.target.clone();
    let labels = st.labels.clone();
    let colors = st.colors.clone();
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let pressed = wrong_idx();
    let card_h = 70.0 + (labels.len() as f32 * 48.0);

    // Build UINode tree for ground truth
    let card_rect = Rect::new(card_x, card_y, 320.0, card_h);
    let children: Vec<_> = labels.iter().map(|l| {
        let btn_rect = Rect::new(card_x, card_y, 320.0, card_h);
        if *l == target {
            ui_node::target_button(l.as_str(), btn_rect)
        } else {
            ui_node::button(l.as_str(), btn_rect)
        }
    }).collect();
    let tree = ui_node::card(card_rect, children);
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), true);

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 20px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); width: 280px; font-family: system-ui, sans-serif;",
        card_x, card_y
    );

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
                    "Level 10"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Find the right button"
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
                    style: "{card_style}",

                    p {
                        style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                        "Click the button that says "
                        span {
                            style: "font-weight: 700; color: #111;",
                            "\"{target}\""
                        }
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 10px;",
                        for (i, label) in labels.iter().enumerate() {
                            {
                                let (btn_color, btn_pressed) = BUTTON_COLORS[colors[i]];
                                let is_target = *label == target;
                                let is_wrong = pressed == Some(i);
                                let btn_bg = if is_wrong { btn_pressed } else { btn_color };
                                let transform = if is_wrong { "scale(0.95)" } else { "scale(1)" };
                                let label_clone = label.clone();
                                rsx! {
                                    button {
                                        class: if is_target { "target" } else { "" },
                                        style: "padding: 10px 20px; background: {btn_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; cursor: pointer; text-align: left; font-family: system-ui, sans-serif; transition: transform 0.1s, background 0.1s; transform: {transform};",
                                        onclick: move |_| {
                                            if is_target {
                                                score.set(score() + 1);
                                                wrong_idx.set(None);
                                                bg.set(random_canvas_bg());
                                                state.set(random_level5());
                                            } else {
                                                wrong_idx.set(Some(i));
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(200).await;
                                                    wrong_idx.set(None);
                                                });
                                            }
                                        },
                                        "{label_clone}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: 320.0,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
