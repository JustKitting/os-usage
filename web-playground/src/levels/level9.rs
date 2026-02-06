use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, ordinal, describe_position};

const INPUT_LABELS: &[&str] = &[
    "Username", "Email", "Password", "First name", "Last name",
    "Phone", "Address", "City", "Zip code", "Company",
    "Website", "Bio", "Title", "Comment", "Search",
];

const WORDS: &[&str] = &[
    "hello", "world", "search", "login", "submit", "click", "enter",
    "send", "save", "open", "close", "next", "back", "done",
];

const DROPDOWN_GROUPS: &[(&str, &[&str])] = &[
    ("Color", &["Red", "Blue", "Green", "Yellow", "Purple", "Orange"]),
    ("Fruit", &["Apple", "Banana", "Cherry", "Grape", "Mango", "Peach"]),
    ("Animal", &["Cat", "Dog", "Bird", "Fish", "Horse", "Bear"]),
    ("Planet", &["Mercury", "Venus", "Mars", "Jupiter", "Saturn"]),
];

// kind: 0=text, 1=dropdown, 2=toggle
struct MixedInput {
    label: String,
    kind: u8,
    dropdown_options: Vec<String>,
}

struct Level9State {
    by_name: bool,
    inputs: Vec<MixedInput>,
    target_idx: usize,
    target_word: String,
    target_select: String,
    x: f32,
    y: f32,
}

fn random_level9() -> Level9State {
    let mut rng = fresh_rng();
    let count = rng.random_range(3..=5usize);
    let by_name = rng.random_range(0..2u8) == 0;

    let mut label_indices: Vec<usize> = (0..INPUT_LABELS.len()).collect();
    let mut group_indices: Vec<usize> = (0..DROPDOWN_GROUPS.len()).collect();
    let mut inputs = Vec::with_capacity(count);

    for _ in 0..count {
        let li = rng.random_range(0..label_indices.len());
        let label = INPUT_LABELS[label_indices.remove(li)].to_string();

        let mut kind = rng.random_range(0..3u8);
        if kind == 1 && group_indices.is_empty() {
            kind = 0;
        }

        let dropdown_options = if kind == 1 {
            let gi = rng.random_range(0..group_indices.len());
            let group_idx = group_indices.remove(gi);
            let (_, all_opts) = DROPDOWN_GROUPS[group_idx];
            let opt_count = rng.random_range(4..=all_opts.len().min(5));
            let mut oi: Vec<usize> = (0..all_opts.len()).collect();
            let mut opts = Vec::with_capacity(opt_count);
            for _ in 0..opt_count {
                let j = rng.random_range(0..oi.len());
                opts.push(all_opts[oi.remove(j)].to_string());
            }
            opts
        } else {
            Vec::new()
        };

        inputs.push(MixedInput { label, kind, dropdown_options });
    }

    let target_idx = rng.random_range(0..count);

    let target_word = if inputs[target_idx].kind == 0 {
        WORDS[rng.random_range(0..WORDS.len())].to_string()
    } else {
        String::new()
    };

    let target_select = if inputs[target_idx].kind == 1 {
        let opts = &inputs[target_idx].dropdown_options;
        opts[rng.random_range(0..opts.len())].clone()
    } else {
        String::new()
    };

    let card_w = 340.0;
    let card_h = 80.0 + (count as f32 * 72.0);
    let pad = 80.0;
    let x = rng.random_range(pad..(Position::VIEWPORT - card_w - pad).max(pad));
    let y = rng.random_range(pad..(Position::VIEWPORT - card_h - pad).max(pad));

    Level9State { by_name, inputs, target_idx, target_word, target_select, x, y }
}

#[component]
pub fn Level9() -> Element {
    let mut state = use_signal(|| random_level9());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong_idx = use_signal(|| None::<usize>);
    let mut inputs_text = use_signal(|| vec![String::new(); 5]);

    let st = state.read();
    let by_name = st.by_name;
    let inputs_data: Vec<(String, u8, Vec<String>)> = st.inputs.iter()
        .map(|inp| (inp.label.clone(), inp.kind, inp.dropdown_options.clone()))
        .collect();
    let target_idx = st.target_idx;
    let target_word = st.target_word.clone();
    let target_select = st.target_select.clone();
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let input_count = inputs_data.len();
    let pressed = wrong_idx();
    let target_kind = inputs_data[target_idx].1;
    let target_label = inputs_data[target_idx].0.clone();
    let target_ord = ordinal(target_idx + 1);

    // Ground truth
    let card_h = 80.0 + (input_count as f32 * 72.0);
    let position_desc = describe_position(card_x, card_y, 340.0, card_h);

    let inputs_desc = inputs_data.iter().enumerate()
        .map(|(i, (label, kind, opts))| {
            let kind_str = match kind {
                0 => "text".to_string(),
                1 => format!("dropdown: {}", opts.iter().map(|o| format!("\"{}\"", o)).collect::<Vec<_>>().join(", ")),
                _ => "toggle".to_string(),
            };
            if i == target_idx {
                format!("\"{}\" ({}, target)", label, kind_str)
            } else {
                format!("\"{}\" ({})", label, kind_str)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let action_desc = match target_kind {
        0 => format!("type \"{}\"", target_word),
        1 => format!("select \"{}\"", target_select),
        _ => "toggle on".to_string(),
    };

    let ref_desc = if by_name {
        format!("\"{}\" (by name)", target_label)
    } else {
        format!("{} input (by ordinal)", target_ord)
    };

    let description = format!(
        "mixed input card, {} inputs: {}, {}, target: {}, at {}",
        input_count, inputs_desc, action_desc, ref_desc, position_desc
    );

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
                    "Level 15"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Mixed inputs"
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

                    // Instruction — varies by (target_kind, by_name)
                    if target_kind == 0 && by_name {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Type "
                            span { style: "font-weight: 700; color: #111; font-family: monospace;", "\"{target_word}\"" }
                            " into "
                            span { style: "font-weight: 700; color: #111;", "\"{target_label}\"" }
                        }
                    } else if target_kind == 0 {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Type "
                            span { style: "font-weight: 700; color: #111; font-family: monospace;", "\"{target_word}\"" }
                            " into the "
                            span { style: "font-weight: 700; color: #111;", "{target_ord}" }
                            " input"
                        }
                    } else if target_kind == 1 && by_name {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Select "
                            span { style: "font-weight: 700; color: #111;", "\"{target_select}\"" }
                            " from "
                            span { style: "font-weight: 700; color: #111;", "\"{target_label}\"" }
                        }
                    } else if target_kind == 1 {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Select "
                            span { style: "font-weight: 700; color: #111;", "\"{target_select}\"" }
                            " from the "
                            span { style: "font-weight: 700; color: #111;", "{target_ord}" }
                            " input"
                        }
                    } else if by_name {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Toggle "
                            span { style: "font-weight: 700; color: #111;", "\"{target_label}\"" }
                            " on"
                        }
                    } else {
                        p {
                            style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                            "Toggle the "
                            span { style: "font-weight: 700; color: #111;", "{target_ord}" }
                            " input on"
                        }
                    }

                    // Mixed inputs
                    div {
                        style: "display: flex; flex-direction: column; gap: 12px;",
                        for (i, (label, kind, opts)) in inputs_data.iter().enumerate() {
                            {
                                let is_wrong = pressed == Some(i);
                                let border_color = if is_wrong { "#ef4444" } else { "#d1d5db" };
                                let label_clone = label.clone();
                                let kind_val = *kind;
                                let is_target = i == target_idx;
                                let tw = target_word.clone();
                                let ts = target_select.clone();
                                let input_val = inputs_text.read().get(i).cloned().unwrap_or_default();
                                let opts_clone = opts.clone();

                                // Toggle visuals
                                let track_color = if is_wrong { "#ef4444" } else { "#d1d5db" };
                                let knob_left = if is_wrong { "22px" } else { "2px" };

                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 4px;",
                                        label {
                                            style: "font-size: 13px; color: #6b7280; font-weight: 500;",
                                            "{label_clone}"
                                        }
                                        if kind_val == 0 {
                                            input {
                                                class: if is_target { "target" } else { "" },
                                                r#type: "text",
                                                tabindex: "-1",
                                                style: "padding: 8px 12px; border: 1px solid {border_color}; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; background: white; color: #111; transition: border-color 0.15s;",
                                                placeholder: "Type here...",
                                                value: "{input_val}",
                                                oninput: move |e: Event<FormData>| {
                                                    let val = e.value();
                                                    if let Some(slot) = inputs_text.write().get_mut(i) {
                                                        *slot = val.clone();
                                                    }
                                                    if !tw.is_empty() && val == tw {
                                                        if is_target {
                                                            score.set(score() + 1);
                                                            wrong_idx.set(None);
                                                            bg.set(random_canvas_bg());
                                                            state.set(random_level9());
                                                            inputs_text.set(vec![String::new(); 5]);
                                                            document::eval("document.activeElement?.blur()");
                                                        } else {
                                                            wrong_idx.set(Some(i));
                                                            spawn(async move {
                                                                gloo_timers::future::TimeoutFuture::new(400).await;
                                                                wrong_idx.set(None);
                                                            });
                                                        }
                                                    }
                                                },
                                            }
                                        } else if kind_val == 1 {
                                            super::CustomSelect {
                                                options: opts_clone.clone(),
                                                is_target: is_target,
                                                target_option: if is_target { ts.clone() } else { String::new() },
                                                border_color: border_color.to_string(),
                                                on_select: move |val: String| {
                                                    if is_target && val == ts {
                                                        score.set(score() + 1);
                                                        wrong_idx.set(None);
                                                        bg.set(random_canvas_bg());
                                                        state.set(random_level9());
                                                        inputs_text.set(vec![String::new(); 5]);
                                                    } else {
                                                        wrong_idx.set(Some(i));
                                                        spawn(async move {
                                                            gloo_timers::future::TimeoutFuture::new(400).await;
                                                            wrong_idx.set(None);
                                                        });
                                                    }
                                                },
                                            }
                                        } else {
                                            div {
                                                class: if is_target { "target" } else { "" },
                                                "data-label": "{label_clone}",
                                                style: "display: flex; align-items: center; justify-content: space-between; cursor: pointer;",
                                                onclick: move |_| {
                                                    if is_target {
                                                        score.set(score() + 1);
                                                        wrong_idx.set(None);
                                                        bg.set(random_canvas_bg());
                                                        state.set(random_level9());
                                                        inputs_text.set(vec![String::new(); 5]);
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
                                                    "Off"
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
                }
            }

            super::GroundTruth {
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: 340.0,
                target_h: card_h,
                steps: match target_kind {
                    0 => format!(r#"[{{"action":"type","target":"Type here...","value":"{}"}}]"#, target_word),
                    1 => format!(r#"[{{"action":"click","target":"Choose..."}},{{"action":"click","target":"{}"}}]"#, target_select),
                    _ => format!(r#"[{{"action":"click","target":"{}"}}]"#, target_label),
                },
            }
        }
    }
}
