use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, UINode, Visual, Rect, ToggleState};
use super::{fresh_rng, random_canvas_bg, ordinal};

const TOGGLE_LABELS: &[&str] = &[
    "Dark mode", "Notifications", "Auto-save", "Sync", "Airplane mode",
    "Bluetooth", "Wi-Fi", "Location", "Do not disturb", "Night shift",
    "Low power", "VPN", "Hotspot", "NFC", "Auto-rotate",
];

const TOGGLE_TRACK_COLORS: &[(&str, &str)] = &[
    ("#d1d5db", "#3b82f6"),
    ("#d1d5db", "#22c55e"),
    ("#d1d5db", "#8b5cf6"),
    ("#d1d5db", "#f59e0b"),
    ("#d1d5db", "#ec4899"),
    ("#d1d5db", "#6366f1"),
];

struct Level6State {
    target: usize,
    labels: Vec<String>,
    color_indices: Vec<usize>,
    x: f32,
    y: f32,
}

fn random_level6() -> Level6State {
    let mut rng = fresh_rng();
    let count = rng.random_range(3..=6usize);

    let mut indices: Vec<usize> = (0..TOGGLE_LABELS.len()).collect();
    let mut labels = Vec::with_capacity(count);
    for _ in 0..count {
        let i = rng.random_range(0..indices.len());
        labels.push(TOGGLE_LABELS[indices.remove(i)].to_string());
    }

    let color_indices: Vec<usize> = (0..count)
        .map(|_| rng.random_range(0..TOGGLE_TRACK_COLORS.len()))
        .collect();

    let target = rng.random_range(0..count);

    let card_w = 300.0;
    let card_h = 60.0 + (count as f32 * 52.0);
    let pad = 80.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, pad, vp_w * 1.3, vp_h * 1.3);

    Level6State { target, labels, color_indices, x, y }
}

#[component]
pub fn Level6() -> Element {
    let mut state = use_signal(|| random_level6());
    let mut score = use_signal(|| 0u32);
    let mut wrong_idx = use_signal(|| None::<usize>);
    let mut bg = use_signal(|| random_canvas_bg());

    let st = state.read();
    let target = st.target;
    let labels = st.labels.clone();
    let color_indices = st.color_indices.clone();
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let pressed = wrong_idx();
    let ordinal_str = ordinal(target + 1);
    let card_h = 60.0 + (labels.len() as f32 * 52.0);

    // Build UINode tree for ground truth
    let card_rect = Rect::new(card_x, card_y, 300.0, card_h);
    let children: Vec<UINode> = labels.iter().enumerate().map(|(i, l)| {
        let toggle_rect = Rect::new(card_x, card_y, 300.0, card_h);
        if i == target {
            // Target toggle — use the builder which sets is_target = true
            ui_node::toggle(l.as_str(), toggle_rect, false)
        } else {
            // Non-target toggle — manually construct without target flag
            UINode::Toggle(Visual::new(l.as_str(), toggle_rect), ToggleState { is_on: false })
        }
    }).collect();
    let tree = ui_node::card(card_rect, children);
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), true);

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 20px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); width: 260px; font-family: system-ui, sans-serif;",
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
                    "Level 11"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Click the right toggle"
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
                        "Click the "
                        span {
                            style: "font-weight: 700; color: #111;",
                            "{ordinal_str}"
                        }
                        " toggle"
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 14px;",
                        for (i, label) in labels.iter().enumerate() {
                            {
                                let is_target = i == target;
                                let is_wrong = pressed == Some(i);
                                let (track_off, track_on) = TOGGLE_TRACK_COLORS[color_indices[i]];
                                let track_color = if is_wrong { track_on } else { track_off };
                                let knob_left = if is_wrong { "22px" } else { "2px" };
                                let shake = if is_wrong { "translateX(2px)" } else { "translateX(0)" };
                                let label_clone = label.clone();
                                rsx! {
                                    div {
                                        class: if is_target { "target" } else { "" },
                                        "data-label": "{label_clone}",
                                        style: "display: flex; align-items: center; justify-content: space-between; cursor: pointer; transition: transform 0.1s; transform: {shake};",
                                        onclick: move |_| {
                                            if is_target {
                                                score.set(score() + 1);
                                                wrong_idx.set(None);
                                                bg.set(random_canvas_bg());
                                                state.set(random_level6());
                                            } else {
                                                wrong_idx.set(Some(i));
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(200).await;
                                                    wrong_idx.set(None);
                                                });
                                            }
                                        },

                                        span {
                                            style: "font-size: 14px; color: #374151;",
                                            "{label_clone}"
                                        }

                                        div {
                                            style: "width: 44px; height: 24px; background: {track_color}; border-radius: 12px; position: relative; flex-shrink: 0; transition: background 0.15s;",

                                            div {
                                                style: "width: 20px; height: 20px; background: white; border-radius: 50%; position: absolute; top: 2px; left: {knob_left}; box-shadow: 0 1px 3px rgba(0,0,0,0.2); transition: left 0.15s;",
                                            }
                                        }
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
                target_w: 300.0,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
