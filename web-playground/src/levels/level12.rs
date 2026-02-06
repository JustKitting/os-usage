use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, ordinal, describe_position};

const FIELD_NAMES: &[&str] = &[
    "Name", "Email", "Phone", "Address", "City", "State",
    "Zip", "Country", "Username", "Company", "Website", "Age",
    "Title", "Notes", "Fax", "Date", "Time", "Color",
    "Price", "Qty", "Code", "ID", "Ref", "Tag", "URL",
];

const TYPE_WORDS: &[&str] = &[
    "hello", "world", "test", "alpha", "bravo", "delta",
    "echo", "fox", "kilo", "lima", "oscar", "tango",
];

#[derive(Clone)]
struct GridCell {
    has_label: bool,
    name: String,
}

struct Level12State {
    cols: usize,
    rows: usize,
    cells: Vec<Option<GridCell>>,
    target_input: usize,
    target_word: String,
    mode: u8, // 0=ordinal, 1=by placeholder, 2=by label
    x: f32,
    y: f32,
}

fn random_level12() -> Level12State {
    let mut rng = fresh_rng();
    let cols = rng.random_range(4..=6usize);
    let rows = rng.random_range(3..=5usize);
    let total = cols * rows;
    let input_count = rng.random_range((total * 3 / 4).max(10)..=total);

    let mut indices: Vec<usize> = (0..total).collect();
    let mut selected = Vec::new();
    for _ in 0..input_count {
        let i = rng.random_range(0..indices.len());
        selected.push(indices.remove(i));
    }
    selected.sort();

    let mut name_pool: Vec<usize> = (0..FIELD_NAMES.len()).collect();
    let mut cells: Vec<Option<GridCell>> = vec![None; total];
    let mut label_idxs: Vec<usize> = Vec::new();
    let mut placeholder_idxs: Vec<usize> = Vec::new();

    for (input_i, &cell_idx) in selected.iter().enumerate() {
        let ni = rng.random_range(0..name_pool.len());
        let name = FIELD_NAMES[name_pool.remove(ni)].to_string();

        let remaining = input_count - input_i - 1;
        let has_label = if label_idxs.is_empty() && remaining == 0 {
            true
        } else if placeholder_idxs.is_empty() && remaining == 0 {
            false
        } else {
            rng.random_bool(0.5)
        };

        if has_label { label_idxs.push(input_i); } else { placeholder_idxs.push(input_i); }
        cells[cell_idx] = Some(GridCell { has_label, name });
    }

    let mut mode = rng.random_range(0..3u8);
    let target_input = match mode {
        1 if !placeholder_idxs.is_empty() => {
            placeholder_idxs[rng.random_range(0..placeholder_idxs.len())]
        }
        2 if !label_idxs.is_empty() => {
            label_idxs[rng.random_range(0..label_idxs.len())]
        }
        _ => {
            mode = 0;
            rng.random_range(0..input_count)
        }
    };

    let wi = rng.random_range(0..TYPE_WORDS.len());
    let target_word = TYPE_WORDS[wi].to_string();

    let cell_w: f32 = if cols <= 4 { 120.0 } else { 100.0 };
    let gap: f32 = 8.0;
    let pad_inner: f32 = 16.0;
    let grid_w = cols as f32 * cell_w + (cols as f32 - 1.0) * gap;
    let card_w = grid_w + 2.0 * pad_inner;
    let row_h: f32 = if rows <= 3 { 65.0 } else { 55.0 };
    let card_h = rows as f32 * row_h + (rows as f32 - 1.0) * gap + 110.0;

    let margin = 60.0;
    let x = rng.random_range(margin..(Position::VIEWPORT - card_w - margin).max(margin + 1.0));
    let y = rng.random_range(margin..(Position::VIEWPORT - card_h - margin).max(margin + 1.0));

    Level12State { cols, rows, cells, target_input, target_word, mode, x, y }
}

#[component]
pub fn Level12() -> Element {
    let mut state = use_signal(|| random_level12());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_count = state.read().cells.iter().filter(|c| c.is_some()).count();
    let mut inputs_text = use_signal(move || vec![String::new(); initial_count]);
    let mut wrong = use_signal(|| false);
    let mut wrong_field = use_signal(|| Option::<usize>::None);

    let st = state.read();
    let cols = st.cols;
    let rows = st.rows;
    let cells: Vec<Option<GridCell>> = st.cells.clone();
    let target_input = st.target_input;
    let target_word = st.target_word.clone();
    let mode = st.mode;
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let total_cells = cols * rows;

    // Map each grid cell to (has_input, has_label, name, input_index)
    let render_info: Vec<(bool, bool, String, usize)> = {
        let mut idx = 0usize;
        cells.iter().map(|c| {
            match c {
                Some(cell) => {
                    let i = idx;
                    idx += 1;
                    (true, cell.has_label, cell.name.clone(), i)
                }
                None => (false, false, String::new(), 0),
            }
        }).collect()
    };

    let target_cell = cells.iter().filter_map(|c| c.as_ref()).nth(target_input).unwrap();
    let target_name = target_cell.name.clone();
    let target_ord = ordinal(target_input + 1);
    let wf = wrong_field();
    let is_wrong = wrong();

    let cell_w: f32 = if cols <= 4 { 120.0 } else { 100.0 };
    let content_w = cols as f32 * cell_w + (cols as f32 - 1.0) * 8.0;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px;",
        card_x, card_y, content_w
    );
    let grid_style = format!(
        "display: grid; grid-template-columns: repeat({}, {}px); gap: 8px; margin-bottom: 10px;",
        cols, cell_w as u32
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth
    let input_count = cells.iter().filter(|c| c.is_some()).count();
    let inputs_desc: String = {
        let mut ii = 0usize;
        let mut parts = Vec::new();
        for (ci, c) in cells.iter().enumerate() {
            if let Some(cell) = c {
                let r = ci / cols + 1;
                let col = ci % cols + 1;
                let kind = if cell.has_label { "label" } else { "placeholder" };
                let marker = if ii == target_input { " (target)" } else { "" };
                parts.push(format!("#{} row {} col {}: {} \"{}\"{}",
                    ii + 1, r, col, kind, cell.name, marker));
                ii += 1;
            }
        }
        parts.join(", ")
    };
    let mode_desc = match mode {
        1 => format!("by placeholder \"{}\"", target_name),
        2 => format!("by label \"{}\"", target_name),
        _ => format!("by ordinal ({})", target_ord),
    };
    let card_total_w = content_w + 32.0;
    let row_h: f32 = if rows <= 3 { 65.0 } else { 55.0 };
    let card_h = rows as f32 * row_h + (rows as f32 - 1.0) * 8.0 + 110.0;
    let position_desc = describe_position(card_x, card_y, card_total_w, card_h);
    let description = format!(
        "grid form {}x{}, {} inputs: [{}], mode: {}, type \"{}\", at {}",
        cols, rows, input_count, inputs_desc, mode_desc, target_word, position_desc
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
                    "Level 18"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Grid Form"
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

                    // Instruction
                    p {
                        style: "margin: 0 0 14px 0; font-size: 15px; color: #374151; font-weight: 500;",
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
                                " into the input with placeholder "
                                span { style: "font-weight: 700; color: #111;", "\"{target_name}\"" }
                            }
                        } else {
                            span {
                                "Type "
                                span { style: "font-weight: 700; color: #111; font-family: monospace;", "\"{target_word}\"" }
                                " into the input labeled "
                                span { style: "font-weight: 700; color: #111;", "\"{target_name}\"" }
                            }
                        }
                    }

                    // Grid
                    div {
                        style: "{grid_style}",

                        for ci in 0..total_cells {
                            {
                                let has_input = render_info[ci].0;
                                let has_lbl = render_info[ci].1;
                                let nm = render_info[ci].2.clone();
                                let iidx = render_info[ci].3;

                                if has_input {
                                    let val = inputs_text.read().get(iidx).cloned().unwrap_or_default();
                                    let border_c = if wf == Some(iidx) { "#ef4444" } else { "#d1d5db" };
                                    let ph = if has_lbl { String::new() } else { nm.clone() };
                                    rsx! {
                                        div {
                                            style: "display: flex; flex-direction: column;",
                                            if has_lbl {
                                                label {
                                                    style: "font-size: 12px; color: #374151; font-weight: 500; margin-bottom: 4px;",
                                                    "{nm}"
                                                }
                                            }
                                            input {
                                                r#type: "text",
                                                tabindex: "-1",
                                                class: if iidx == target_input { "target" } else { "" },
                                                "data-label": "{nm}",
                                                style: "width: 100%; padding: 5px 6px; border: 1px solid {border_c}; border-radius: 4px; font-size: 12px; font-family: system-ui, sans-serif; outline: none; box-sizing: border-box; transition: border-color 0.15s;",
                                                placeholder: "{ph}",
                                                value: "{val}",
                                                oninput: move |e: Event<FormData>| {
                                                    let mut vals = inputs_text.write();
                                                    if let Some(v) = vals.get_mut(iidx) {
                                                        *v = e.value();
                                                    }
                                                },
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div {}
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
                            let val = inputs_text.read().get(target_input).cloned().unwrap_or_default();
                            if val.eq_ignore_ascii_case(&target_word) {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level12();
                                let count = new_st.cells.iter().filter(|c| c.is_some()).count();
                                state.set(new_st);
                                inputs_text.set(vec![String::new(); count]);
                                wrong.set(false);
                                wrong_field.set(None);
                                document::eval("document.activeElement?.blur()");
                            } else {
                                wrong.set(true);
                                wrong_field.set(Some(target_input));
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
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: card_total_w,
                target_h: card_h,
                steps: format!(r#"[{{"action":"type","target":"{}","value":"{}"}},{{"action":"click","target":"Submit"}}]"#, target_name, target_word),
            }
        }
    }
}
