use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, describe_position};

const DROPDOWN_GROUPS: &[(&str, &[&str])] = &[
    ("Color", &["Red", "Blue", "Green", "Yellow", "Purple", "Orange", "Pink"]),
    ("Fruit", &["Apple", "Banana", "Cherry", "Grape", "Mango", "Peach", "Pear"]),
    ("Animal", &["Cat", "Dog", "Bird", "Fish", "Horse", "Bear", "Wolf"]),
    ("Country", &["France", "Japan", "Brazil", "Canada", "Italy", "Spain", "India"]),
    ("Language", &["Python", "Rust", "Java", "Go", "Ruby", "Swift", "Kotlin"]),
    ("Planet", &["Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Neptune", "Uranus"]),
];

struct Level4State {
    label: String,
    options: Vec<String>,
    target: String,
    x: f32,
    y: f32,
}

fn random_level4() -> Level4State {
    let mut rng = fresh_rng();
    let group_idx = rng.random_range(0..DROPDOWN_GROUPS.len());
    let (label, all_options) = DROPDOWN_GROUPS[group_idx];

    let count = rng.random_range(4..=all_options.len().min(6));
    let mut indices: Vec<usize> = (0..all_options.len()).collect();
    let mut options = Vec::with_capacity(count);
    for _ in 0..count {
        let i = rng.random_range(0..indices.len());
        options.push(all_options[indices.remove(i)].to_string());
    }

    let target_idx = rng.random_range(0..options.len());
    let target = options[target_idx].clone();

    let card_w = 300.0;
    let card_h = 130.0;
    let pad = 80.0;
    let x = rng.random_range(pad..(Position::VIEWPORT - card_w - pad).max(pad));
    let y = rng.random_range(pad..(Position::VIEWPORT - card_h - pad).max(pad));

    Level4State { label: label.to_string(), options, target, x, y }
}

#[component]
pub fn Level4() -> Element {
    let mut state = use_signal(|| random_level4());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());

    let st = state.read();
    let label = st.label.clone();
    let options = st.options.clone();
    let target = st.target.clone();
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let position_desc = describe_position(card_x, card_y, 300.0, 130.0);
    let options_desc = options.iter()
        .map(|o| if *o == target { format!("\"{}\" (target)", o) } else { format!("\"{}\"", o) })
        .collect::<Vec<_>>()
        .join(", ");
    let description = format!(
        "dropdown ({}), {} options: {}, target: \"{}\", at {}",
        label, options.len(), options_desc, target, position_desc
    );

    let steps = format!(r#"[{{"action":"click","target":"Choose..."}},{{"action":"click","target":"{}"}}]"#, target);

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
                    "Level 4"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Select the right option"
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
                    style: "{card_style}",

                    p {
                        style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                        "Select "
                        span {
                            style: "font-weight: 700; color: #111;",
                            "\"{target}\""
                        }
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 6px;",
                        label {
                            style: "font-size: 13px; color: #6b7280; font-weight: 500;",
                            "{label}"
                        }
                        super::CustomSelect {
                            options: options.clone(),
                            is_target: true,
                            target_option: target.clone(),
                            border_color: "#d1d5db".to_string(),
                            on_select: move |val: String| {
                                if val == target {
                                    score.set(score() + 1);
                                    bg.set(random_canvas_bg());
                                    state.set(random_level4());
                                }
                            },
                        }
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: 300.0,
                target_h: 130.0,
                steps: steps,
            }
        }
    }
}
