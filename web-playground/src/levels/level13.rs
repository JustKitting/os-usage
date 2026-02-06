use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect, UINode, Visual, InputState};
use super::{fresh_rng, random_canvas_bg, ordinal};

const COLUMN_NAMES: &[&str] = &[
    "Name", "Email", "Phone", "City", "Code", "Notes",
    "Price", "Qty", "ID", "Tag", "Ref", "Status",
];

const PH_WORDS: &[&str] = &[
    "enter...", "type here", "value", "input", "data",
    "fill in", "add text", "enter", "type", "edit",
    "info", "detail", "text", "write", "note",
    "item", "record", "field", "entry", "memo",
    "desc", "label", "key", "val", "src",
];

const TYPE_WORDS: &[&str] = &[
    "hello", "world", "test", "alpha", "bravo", "delta",
    "echo", "fox", "kilo", "lima", "oscar", "tango",
];

struct Level13State {
    cols: usize,
    body_rows: usize,
    headers: Vec<String>,
    placeholders: Vec<String>, // flat len = cols * body_rows, empty = none
    target_row: usize,
    target_col: usize,
    target_word: String,
    mode: u8, // 0=ordinal, 1=row+column, 2=by placeholder
    x: f32,
    y: f32,
}

fn random_level13() -> Level13State {
    let mut rng = fresh_rng();
    let cols = rng.random_range(3..=6usize);
    let body_rows = rng.random_range(4..=7usize);
    let total = cols * body_rows;

    // Column headers
    let mut header_pool: Vec<usize> = (0..COLUMN_NAMES.len()).collect();
    let mut headers = Vec::new();
    for _ in 0..cols {
        let i = rng.random_range(0..header_pool.len());
        headers.push(COLUMN_NAMES[header_pool.remove(i)].to_string());
    }

    // Assign placeholders to 25-50% of cells
    let ph_count = rng.random_range(total / 4..=total / 2);
    let mut cell_pool: Vec<usize> = (0..total).collect();
    let mut ph_word_pool: Vec<usize> = (0..PH_WORDS.len()).collect();
    let mut placeholders = vec![String::new(); total];

    for _ in 0..ph_count.min(ph_word_pool.len()) {
        let ci = rng.random_range(0..cell_pool.len());
        let idx = cell_pool.remove(ci);
        let pi = rng.random_range(0..ph_word_pool.len());
        placeholders[idx] = PH_WORDS[ph_word_pool.remove(pi)].to_string();
    }

    // Pick mode and target
    let mut mode = rng.random_range(0..3u8);
    let (target_row, target_col) = match mode {
        2 => {
            let with_ph: Vec<usize> = (0..total).filter(|&i| !placeholders[i].is_empty()).collect();
            if with_ph.is_empty() {
                mode = 1;
                (rng.random_range(0..body_rows), rng.random_range(0..cols))
            } else {
                let idx = with_ph[rng.random_range(0..with_ph.len())];
                (idx / cols, idx % cols)
            }
        }
        _ => {
            (rng.random_range(0..body_rows), rng.random_range(0..cols))
        }
    };

    let wi = rng.random_range(0..TYPE_WORDS.len());
    let target_word = TYPE_WORDS[wi].to_string();

    let col_w: f32 = match cols { 3 => 130.0, 4 => 110.0, 5 => 95.0, _ => 82.0 };
    let card_pad: f32 = 16.0;
    let card_w = cols as f32 * col_w + 2.0 * card_pad;
    let row_h: f32 = 34.0;
    let card_h = (body_rows + 1) as f32 * row_h + 110.0;

    let margin = 50.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, margin, vp_w * 1.3, vp_h * 1.3);

    Level13State { cols, body_rows, headers, placeholders, target_row, target_col, target_word, mode, x, y }
}

#[component]
pub fn Level13() -> Element {
    let mut state = use_signal(|| random_level13());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_total = { let s = state.read(); s.cols * s.body_rows };
    let mut inputs_text = use_signal(move || vec![String::new(); initial_total]);
    let mut wrong = use_signal(|| false);
    let mut wrong_field = use_signal(|| Option::<usize>::None);

    let st = state.read();
    let cols = st.cols;
    let body_rows = st.body_rows;
    let headers: Vec<String> = st.headers.clone();
    let placeholders: Vec<String> = st.placeholders.clone();
    let target_row = st.target_row;
    let target_col = st.target_col;
    let target_word = st.target_word.clone();
    let mode = st.mode;
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let target_idx = target_row * cols + target_col;
    let target_ord = ordinal(target_idx + 1);
    let target_header = headers[target_col].clone();
    let target_ph = placeholders[target_idx].clone();
    let wf = wrong_field();
    let is_wrong = wrong();
    let viewport_style = super::viewport_style(&bg(), true);

    let col_w: f32 = match cols { 3 => 130.0, 4 => 110.0, 5 => 95.0, _ => 82.0 };
    let content_w = cols as f32 * col_w;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px;",
        card_x, card_y, content_w
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth
    let card_total_w = content_w + 32.0;
    let card_h = (body_rows + 1) as f32 * 34.0 + 110.0;

    // Build UINode tree for ground truth
    let row_h: f32 = 34.0;
    let input_nodes: Vec<UINode> = {
        let mut nodes = Vec::new();
        for ri in 0..body_rows {
            for ci in 0..cols {
                let cell_idx = ri * cols + ci;
                let ph = &placeholders[cell_idx];
                let cell_rect = Rect::new(
                    card_x + 16.0 + ci as f32 * col_w,
                    card_y + 70.0 + (ri + 1) as f32 * row_h,
                    col_w,
                    row_h,
                );
                if cell_idx == target_idx {
                    nodes.push(ui_node::text_input(&headers[ci], cell_rect, ph.as_str(), &target_word));
                } else {
                    nodes.push(UINode::TextInput(
                        Visual::new(&headers[ci], cell_rect),
                        InputState { placeholder: ph.clone(), current_value: String::new(), target_value: String::new() },
                    ));
                }
            }
        }
        nodes
    };
    let tree = ui_node::form(
        Rect::new(card_x, card_y, card_total_w, card_h),
        "Submit",
        input_nodes,
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
                    "Level 19"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Table"
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

                    // Instruction
                    p {
                        style: "margin: 0 0 12px 0; font-size: 14px; color: #374151; font-weight: 500;",
                        if mode == 0 {
                            span {
                                "Type "
                                span { style: "font-weight: 700; color: #111; font-family: monospace;", "\"{target_word}\"" }
                                " into the "
                                span { style: "font-weight: 700; color: #111;", "{target_ord}" }
                                " input"
                            }
                        } else if mode == 1 {
                            span {
                                "Type "
                                span { style: "font-weight: 700; color: #111; font-family: monospace;", "\"{target_word}\"" }
                                " into row "
                                span { style: "font-weight: 700; color: #111;", "{target_row + 1}" }
                                ", "
                                span { style: "font-weight: 700; color: #111;", "\"{target_header}\"" }
                                " column"
                            }
                        } else {
                            span {
                                "Type "
                                span { style: "font-weight: 700; color: #111; font-family: monospace;", "\"{target_word}\"" }
                                " into the input with placeholder "
                                span { style: "font-weight: 700; color: #111;", "\"{target_ph}\"" }
                            }
                        }
                    }

                    // Table
                    table {
                        style: "border-collapse: collapse; width: 100%; margin-bottom: 10px;",

                        thead {
                            tr {
                                for hi in 0..cols {
                                    {
                                        let h = headers[hi].clone();
                                        rsx! {
                                            th {
                                                style: "padding: 6px 4px; background: #f3f4f6; border: 1px solid #d1d5db; font-size: 11px; font-weight: 600; color: #374151; text-align: left;",
                                                "{h}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        tbody {
                            for ri in 0..body_rows {
                                {
                                    let row_bg = if ri % 2 == 0 { "white" } else { "#f9fafb" };
                                    rsx! {
                                        tr {
                                            style: "background: {row_bg};",
                                            for ci in 0..cols {
                                                {
                                                    let cell_idx = ri * cols + ci;
                                                    let val = inputs_text.read().get(cell_idx).cloned().unwrap_or_default();
                                                    let ph = placeholders[cell_idx].clone();
                                                    let input_border = if wf == Some(cell_idx) { "#ef4444" } else { "transparent" };
                                                    rsx! {
                                                        td {
                                                            style: "padding: 2px; border: 1px solid #d1d5db;",
                                                            input {
                                                                r#type: "text",
                                                                tabindex: "-1",
                                                                class: if cell_idx == target_idx { "target" } else { "" },
                                                                "data-label": "{headers[ci]}",
                                                                style: "width: 100%; padding: 4px 3px; border: 1px solid {input_border}; border-radius: 2px; font-size: 11px; font-family: system-ui, sans-serif; outline: none; box-sizing: border-box; background: transparent;",
                                                                placeholder: "{ph}",
                                                                value: "{val}",
                                                                oninput: move |e: Event<FormData>| {
                                                                    let mut vals = inputs_text.write();
                                                                    if let Some(v) = vals.get_mut(cell_idx) {
                                                                        *v = e.value();
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
                            }
                        }
                    }

                    // Submit
                    button {
                        class: "target",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let val = inputs_text.read().get(target_idx).cloned().unwrap_or_default();
                            if val.eq_ignore_ascii_case(&target_word) {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level13();
                                let count = new_st.cols * new_st.body_rows;
                                state.set(new_st);
                                inputs_text.set(vec![String::new(); count]);
                                wrong.set(false);
                                wrong_field.set(None);
                                document::eval("document.activeElement?.blur()");
                            } else {
                                wrong.set(true);
                                wrong_field.set(Some(target_idx));
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                    wrong.set(false);
                                    wrong_field.set(None);
                                });
                            }
                        },
                        "Submit"
                    }
                }
            }

            super::GroundTruth {
                description: String::new(),
                target_x: card_x,
                target_y: card_y,
                target_w: card_total_w,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
