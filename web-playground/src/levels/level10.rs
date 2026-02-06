use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, describe_position};

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
struct FormInput {
    label: String,
    kind: u8,
    dropdown_options: Vec<String>,
}

struct FormTask {
    input_idx: usize,
    word: String,
    select_val: String,
}

struct Level10State {
    inputs: Vec<FormInput>,
    tasks: Vec<FormTask>,
    x: f32,
    y: f32,
}

fn random_level10() -> Level10State {
    let mut rng = fresh_rng();
    let input_count = rng.random_range(3..=5usize);

    let mut label_indices: Vec<usize> = (0..INPUT_LABELS.len()).collect();
    let mut group_indices: Vec<usize> = (0..DROPDOWN_GROUPS.len()).collect();
    let mut inputs = Vec::with_capacity(input_count);

    for _ in 0..input_count {
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

        inputs.push(FormInput { label, kind, dropdown_options });
    }

    let task_count = rng.random_range(2..=3usize).min(input_count);
    let mut available: Vec<usize> = (0..input_count).collect();
    let mut tasks = Vec::with_capacity(task_count);

    for _ in 0..task_count {
        let ti = rng.random_range(0..available.len());
        let idx = available.remove(ti);
        let kind = inputs[idx].kind;

        let word = if kind == 0 {
            WORDS[rng.random_range(0..WORDS.len())].to_string()
        } else {
            String::new()
        };

        let select_val = if kind == 1 {
            let opts = &inputs[idx].dropdown_options;
            opts[rng.random_range(0..opts.len())].clone()
        } else {
            String::new()
        };

        tasks.push(FormTask { input_idx: idx, word, select_val });
    }

    tasks.sort_by_key(|t| t.input_idx);

    let card_w = 340.0;
    let card_h = 140.0 + (input_count as f32 * 68.0);
    let pad = 80.0;
    let x = rng.random_range(pad..(Position::VIEWPORT - card_w - pad).max(pad));
    let y = rng.random_range(pad..(Position::VIEWPORT - card_h - pad).max(pad));

    Level10State { inputs, tasks, x, y }
}

#[component]
pub fn Level10() -> Element {
    let mut state = use_signal(|| random_level10());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut inputs_text = use_signal(|| vec![String::new(); 5]);
    let mut selections = use_signal(|| vec![String::new(); 5]);
    let mut toggled = use_signal(|| vec![false; 5]);
    let mut wrong_btn = use_signal(|| None::<bool>);
    let mut wrong_fields = use_signal(|| vec![false; 5]);

    let st = state.read();
    let inputs_data: Vec<(String, u8, Vec<String>)> = st.inputs.iter()
        .map(|inp| (inp.label.clone(), inp.kind, inp.dropdown_options.clone()))
        .collect();
    let tasks_data: Vec<(usize, String, String)> = st.tasks.iter()
        .map(|t| (t.input_idx, t.word.clone(), t.select_val.clone()))
        .collect();
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let input_count = inputs_data.len();
    let btn_flash = wrong_btn();

    // Task display list for the instruction
    let tasks_display: Vec<(u8, String, String, String)> = tasks_data.iter()
        .map(|(idx, word, sel)| (
            inputs_data[*idx].1,
            inputs_data[*idx].0.clone(),
            word.clone(),
            sel.clone(),
        ))
        .collect();

    // Clone for Submit closure
    let tasks_check: Vec<(usize, u8, String, String)> = tasks_data.iter()
        .map(|(idx, word, sel)| (*idx, inputs_data[*idx].1, word.clone(), sel.clone()))
        .collect();

    // Ground truth
    let card_h = 140.0 + (input_count as f32 * 68.0);
    let position_desc = describe_position(card_x, card_y, 340.0, card_h);

    let inputs_desc = inputs_data.iter().enumerate()
        .map(|(i, (label, kind, opts))| {
            let kind_str = match kind {
                0 => "text".to_string(),
                1 => format!("dropdown: {}", opts.iter().map(|o| format!("\"{}\"", o)).collect::<Vec<_>>().join(", ")),
                _ => "toggle".to_string(),
            };
            if let Some(task) = tasks_data.iter().find(|(idx, _, _)| *idx == i) {
                let action = match kind {
                    0 => format!(", task: type \"{}\"", task.1),
                    1 => format!(", task: select \"{}\"", task.2),
                    _ => ", task: toggle on".to_string(),
                };
                format!("\"{}\" ({}{})", label, kind_str, action)
            } else {
                format!("\"{}\" ({})", label, kind_str)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let description = format!(
        "form card, {} inputs: {}, submit + cancel buttons, {} tasks, at {}",
        input_count, inputs_desc, tasks_data.len(), position_desc
    );

    let steps = {
        let mut parts: Vec<String> = Vec::new();
        for (idx, word, sel) in tasks_data.iter() {
            let (label, kind, _) = &inputs_data[*idx];
            match kind {
                0 => parts.push(format!(r#"{{"action":"type","target":"{}","value":"{}"}}"#, label, word)),
                1 => {
                    parts.push(r#"{"action":"click","target":"Choose..."}"#.to_string());
                    parts.push(format!(r#"{{"action":"click","target":"{}"}}"#, sel));
                }
                _ => parts.push(format!(r#"{{"action":"click","target":"{}"}}"#, label)),
            }
        }
        parts.push(r#"{"action":"click","target":"Submit"}"#.to_string());
        format!("[{}]", parts.join(","))
    };

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 20px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); width: 300px; font-family: system-ui, sans-serif;",
        card_x, card_y
    );

    let submit_bg = if btn_flash == Some(true) { "#ef4444" } else { "#4f46e5" };
    let cancel_bg = if btn_flash == Some(false) { "#ef4444" } else { "#6b7280" };

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
                    "Level 16"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Form submission"
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

                    // Instruction header
                    p {
                        style: "margin: 0 0 6px 0; font-size: 15px; color: #374151; font-weight: 600;",
                        "Complete the form:"
                    }

                    // Task list
                    div {
                        style: "margin-bottom: 14px;",
                        for td in tasks_display.iter() {
                            {
                                let kind = td.0;
                                let label = td.1.clone();
                                let word = td.2.clone();
                                let sel = td.3.clone();
                                rsx! {
                                    if kind == 0 {
                                        p {
                                            style: "margin: 2px 0; font-size: 13px; color: #6b7280;",
                                            "\u{2022} Type "
                                            span { style: "font-weight: 600; color: #374151; font-family: monospace;", "\"{word}\"" }
                                            " into "
                                            span { style: "font-weight: 600; color: #374151;", "\"{label}\"" }
                                        }
                                    } else if kind == 1 {
                                        p {
                                            style: "margin: 2px 0; font-size: 13px; color: #6b7280;",
                                            "\u{2022} Select "
                                            span { style: "font-weight: 600; color: #374151;", "\"{sel}\"" }
                                            " from "
                                            span { style: "font-weight: 600; color: #374151;", "\"{label}\"" }
                                        }
                                    } else {
                                        p {
                                            style: "margin: 2px 0; font-size: 13px; color: #6b7280;",
                                            "\u{2022} Toggle "
                                            span { style: "font-weight: 600; color: #374151;", "\"{label}\"" }
                                            " on"
                                        }
                                    }
                                }
                            }
                        }
                        p {
                            style: "margin: 6px 0 0 0; font-size: 13px; color: #6b7280;",
                            "Then click "
                            span { style: "font-weight: 600; color: #374151;", "Submit" }
                        }
                    }

                    // Form fields
                    div {
                        style: "display: flex; flex-direction: column; gap: 10px;",
                        for (i, (label, kind, opts)) in inputs_data.iter().enumerate() {
                            {
                                let field_wrong = wrong_fields.read().get(i).copied().unwrap_or(false);
                                let border_color = if field_wrong { "#ef4444" } else { "#d1d5db" };
                                let label_clone = label.clone();
                                let kind_val = *kind;
                                let opts_clone = opts.clone();
                                let input_val = inputs_text.read().get(i).cloned().unwrap_or_default();
                                let sel_val = selections.read().get(i).cloned().unwrap_or_default();

                                let has_task = tasks_data.iter().any(|(idx, _, _)| *idx == i);
                                let is_on = toggled.read().get(i).copied().unwrap_or(false);
                                let track_color = if field_wrong { "#ef4444" } else if is_on { "#3b82f6" } else { "#d1d5db" };
                                let knob_left = if is_on { "22px" } else { "2px" };
                                let toggle_text = if is_on { "On" } else { "Off" };

                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 4px;",
                                        label {
                                            style: "font-size: 13px; color: #6b7280; font-weight: 500;",
                                            "{label_clone}"
                                        }
                                        if kind_val == 0 {
                                            input {
                                                r#type: "text",
                                                tabindex: "-1",
                                                class: if has_task { "target" } else { "" },
                                                "data-label": "{label_clone}",
                                                style: "padding: 8px 12px; border: 1px solid {border_color}; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; background: white; color: #111; transition: border-color 0.15s;",
                                                placeholder: "Type here...",
                                                value: "{input_val}",
                                                oninput: move |e: Event<FormData>| {
                                                    if let Some(slot) = inputs_text.write().get_mut(i) {
                                                        *slot = e.value();
                                                    }
                                                },
                                            }
                                        } else if kind_val == 1 {
                                            {
                                                let task_select_val = tasks_data.iter()
                                                    .find(|(idx, _, _)| *idx == i)
                                                    .map(|(_, _, sel)| sel.clone())
                                                    .unwrap_or_default();
                                                rsx! {
                                                    super::CustomSelect {
                                                        options: opts_clone.clone(),
                                                        is_target: has_task,
                                                        target_option: task_select_val,
                                                        border_color: border_color.to_string(),
                                                        on_select: move |val: String| {
                                                            if let Some(slot) = selections.write().get_mut(i) {
                                                                *slot = val;
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        } else {
                                            div {
                                                class: if has_task { "target" } else { "" },
                                                "data-label": "{label_clone}",
                                                style: "display: flex; align-items: center; justify-content: space-between; cursor: pointer;",
                                                onclick: move |_| {
                                                    if let Some(slot) = toggled.write().get_mut(i) {
                                                        *slot = !*slot;
                                                    }
                                                },
                                                span {
                                                    style: "font-size: 14px; color: #374151;",
                                                    "{toggle_text}"
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

                    // Buttons
                    div {
                        style: "display: flex; gap: 8px; margin-top: 16px;",
                        button {
                            style: "flex: 1; padding: 10px; background: {cancel_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; cursor: pointer; transition: background 0.15s;",
                            tabindex: "-1",
                            onclick: move |_| {
                                wrong_btn.set(Some(false));
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(400).await;
                                    wrong_btn.set(None);
                                });
                            },
                            "Cancel"
                        }
                        button {
                            class: "target",
                            style: "flex: 1; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; transition: background 0.15s;",
                            tabindex: "-1",
                            onclick: move |_| {
                                let mut all_correct = true;
                                let mut bad = vec![false; 5];

                                for &(idx, kind, ref word, ref sel) in tasks_check.iter() {
                                    let correct = match kind {
                                        0 => inputs_text.read().get(idx).map(|v| v == word).unwrap_or(false),
                                        1 => selections.read().get(idx).map(|v| v == sel).unwrap_or(false),
                                        _ => toggled.read().get(idx).copied().unwrap_or(false),
                                    };
                                    if !correct {
                                        all_correct = false;
                                        bad[idx] = true;
                                    }
                                }

                                if all_correct {
                                    score.set(score() + 1);
                                    bg.set(random_canvas_bg());
                                    state.set(random_level10());
                                    inputs_text.set(vec![String::new(); 5]);
                                    selections.set(vec![String::new(); 5]);
                                    toggled.set(vec![false; 5]);
                                    wrong_btn.set(None);
                                    wrong_fields.set(vec![false; 5]);
                                    document::eval("document.activeElement?.blur()");
                                } else {
                                    wrong_btn.set(Some(true));
                                    wrong_fields.set(bad);
                                    spawn(async move {
                                        gloo_timers::future::TimeoutFuture::new(600).await;
                                        wrong_btn.set(None);
                                        wrong_fields.set(vec![false; 5]);
                                    });
                                }
                            },
                            "Submit"
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
                steps: steps,
            }
        }
    }
}
