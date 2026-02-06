use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect, UINode, Visual, DropdownState};
use super::{fresh_rng, random_canvas_bg, ordinal};

const DROPDOWN_GROUPS: &[(&str, &[&str])] = &[
    ("Color", &["Red", "Blue", "Green", "Yellow", "Purple", "Orange", "Pink"]),
    ("Fruit", &["Apple", "Banana", "Cherry", "Grape", "Mango", "Peach", "Pear"]),
    ("Animal", &["Cat", "Dog", "Bird", "Fish", "Horse", "Bear", "Wolf"]),
    ("Country", &["France", "Japan", "Brazil", "Canada", "Italy", "Spain", "India"]),
    ("Language", &["Python", "Rust", "Java", "Go", "Ruby", "Swift", "Kotlin"]),
    ("Planet", &["Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Neptune", "Uranus"]),
];

struct DropdownInfo {
    label: String,
    options: Vec<String>,
}

struct Level8State {
    select_by_word: bool,
    dropdowns: Vec<DropdownInfo>,
    target_dropdown: usize,
    target_value: String,
    target_option_pos: usize,
    x: f32,
    y: f32,
}

fn random_level8() -> Level8State {
    let mut rng = fresh_rng();
    let dropdown_count = rng.random_range(2..=4usize);
    let select_by_word = rng.random_range(0..2u8) == 0;

    let mut group_indices: Vec<usize> = (0..DROPDOWN_GROUPS.len()).collect();
    let mut dropdowns = Vec::with_capacity(dropdown_count);

    for _ in 0..dropdown_count {
        let gi = rng.random_range(0..group_indices.len());
        let group_idx = group_indices.remove(gi);
        let (label, all_options) = DROPDOWN_GROUPS[group_idx];

        let count = rng.random_range(4..=all_options.len().min(6));
        let mut opt_indices: Vec<usize> = (0..all_options.len()).collect();
        let mut options = Vec::with_capacity(count);
        for _ in 0..count {
            let oi = rng.random_range(0..opt_indices.len());
            options.push(all_options[opt_indices.remove(oi)].to_string());
        }

        dropdowns.push(DropdownInfo { label: label.to_string(), options });
    }

    let target_dropdown = rng.random_range(0..dropdown_count);
    let target_option_idx = rng.random_range(0..dropdowns[target_dropdown].options.len());
    let target_value = dropdowns[target_dropdown].options[target_option_idx].clone();
    let target_option_pos = target_option_idx + 1;

    let card_w = 340.0;
    let card_h = 80.0 + (dropdown_count as f32 * 80.0);
    let pad = 80.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, pad, vp_w * 1.3, vp_h * 1.3);

    Level8State { select_by_word, dropdowns, target_dropdown, target_value, target_option_pos, x, y }
}

#[component]
pub fn Level8() -> Element {
    let mut state = use_signal(|| random_level8());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong_idx = use_signal(|| None::<usize>);

    let st = state.read();
    let select_by_word = st.select_by_word;
    let dropdowns_data: Vec<(String, Vec<String>)> = st.dropdowns.iter()
        .map(|d| (d.label.clone(), d.options.clone()))
        .collect();
    let target_dropdown = st.target_dropdown;
    let target_value = st.target_value.clone();
    let target_option_pos = st.target_option_pos;
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let dropdown_count = dropdowns_data.len();
    let pressed = wrong_idx();
    let dropdown_ord = ordinal(target_dropdown + 1);
    let option_ord = ordinal(target_option_pos);

    // Ground truth
    let card_h = 80.0 + (dropdown_count as f32 * 80.0);

    // Build UINode tree for ground truth
    let dropdown_nodes: Vec<UINode> = dropdowns_data.iter().enumerate().map(|(i, (label, opts))| {
        let is_target = i == target_dropdown;
        if is_target {
            ui_node::dropdown(
                label.as_str(),
                Rect::new(card_x + 20.0, card_y + 60.0 + i as f32 * 80.0, 260.0, 36.0),
                opts.clone(),
                &target_value,
            )
        } else {
            UINode::Dropdown(
                Visual::new(label.as_str(), Rect::new(card_x + 20.0, card_y + 60.0 + i as f32 * 80.0, 260.0, 36.0)),
                DropdownState {
                    options: opts.clone(),
                    selected: None,
                    target_option: String::new(),
                    trigger_label: "Choose...".into(),
                },
            )
        }
    }).collect();
    let tree = ui_node::card(
        Rect::new(card_x, card_y, 340.0, card_h),
        dropdown_nodes,
    );
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), true);

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 20px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); width: 300px; font-family: system-ui, sans-serif;",
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
                    "Level 14"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Multi-dropdown"
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

                    if select_by_word {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Select "
                            span {
                                style: "font-weight: 700; color: #111;",
                                "\"{target_value}\""
                            }
                            " from the "
                            span {
                                style: "font-weight: 700; color: #111;",
                                "{dropdown_ord}"
                            }
                            " dropdown"
                        }
                    } else {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Select the "
                            span {
                                style: "font-weight: 700; color: #111;",
                                "{option_ord}"
                            }
                            " option from the "
                            span {
                                style: "font-weight: 700; color: #111;",
                                "{dropdown_ord}"
                            }
                            " dropdown"
                        }
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 12px;",
                        for (i, (label, options)) in dropdowns_data.iter().enumerate() {
                            {
                                let is_wrong = pressed == Some(i);
                                let border_color = if is_wrong { "#ef4444" } else { "#d1d5db" };
                                let label_clone = label.clone();
                                let is_target = i == target_dropdown;
                                let expected_value = target_value.clone();
                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 4px;",
                                        label {
                                            style: "font-size: 13px; color: #6b7280; font-weight: 500;",
                                            "{label_clone}"
                                        }
                                        super::CustomSelect {
                                            options: options.clone(),
                                            is_target: is_target,
                                            target_option: if is_target { expected_value.clone() } else { String::new() },
                                            border_color: border_color.to_string(),
                                            on_select: move |val: String| {
                                                if is_target && val == expected_value {
                                                    score.set(score() + 1);
                                                    wrong_idx.set(None);
                                                    bg.set(random_canvas_bg());
                                                    state.set(random_level8());
                                                } else {
                                                    wrong_idx.set(Some(i));
                                                    spawn(async move {
                                                        gloo_timers::future::TimeoutFuture::new(400).await;
                                                        wrong_idx.set(None);
                                                    });
                                                }
                                            },
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
                target_w: 340.0,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
