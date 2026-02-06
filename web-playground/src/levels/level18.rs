use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect, UINode, Visual, StepperState};
use super::{fresh_rng, random_canvas_bg, ordinal};

const STEPPER_LABELS: &[&str] = &[
    "Quantity", "Guests", "Adults", "Children", "Rooms",
    "Tickets", "Copies", "Servings", "Players", "Seats",
    "Items", "Bags", "Boxes", "Units", "Pieces",
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

#[derive(Clone)]
struct StepperInfo {
    label: String,
    min: i32,
    max: i32,
    step: i32,
    target_val: i32,
    start_val: i32,
    accent: String,
    style: u8, // 0=pill, 1=outlined, 2=compact
}

struct Level18State {
    steppers: Vec<StepperInfo>,
    target_stepper: usize,
    mode: u8, // 0=by label, 1=by ordinal
    x: f32,
    y: f32,
    card_w: f32,
}

fn random_level18() -> Level18State {
    let mut rng = fresh_rng();
    let count = rng.random_range(1..=4usize);

    let mut label_pool: Vec<usize> = (0..STEPPER_LABELS.len()).collect();
    let mut color_pool: Vec<usize> = (0..ACCENT_COLORS.len()).collect();
    let mut steppers = Vec::new();

    for _ in 0..count {
        let li = rng.random_range(0..label_pool.len());
        let label = STEPPER_LABELS[label_pool.remove(li)].to_string();

        let ci = rng.random_range(0..color_pool.len());
        let accent = ACCENT_COLORS[color_pool.remove(ci)].to_string();

        let (min, max, step) = match rng.random_range(0..3u8) {
            0 => (0, 20, 1),
            1 => (1, 10, 1),
            _ => (0, 100, 5),
        };

        let steps = (max - min) / step;
        let target_step = rng.random_range(1..steps);
        let target_val = min + target_step * step;

        let start_val = if rng.random_bool(0.6) {
            min
        } else {
            let mut sv = target_val;
            while sv == target_val {
                sv = min + rng.random_range(0..=steps) * step;
            }
            sv
        };

        let style = rng.random_range(0..3u8);

        steppers.push(StepperInfo { label, min, max, step, target_val, start_val, accent, style });
    }

    let target_stepper = rng.random_range(0..count);
    let mode = if count == 1 { 0 } else { rng.random_range(0..2u8) };

    let card_w = rng.random_range(260.0..=400.0f32);
    let stepper_h = 70.0;
    let card_h = count as f32 * stepper_h + 100.0;
    let margin = 50.0;
    let (x, y) = super::safe_position(&mut rng, card_w, card_h, margin);

    Level18State { steppers, target_stepper, mode, x, y, card_w }
}

#[component]
pub fn Level18() -> Element {
    let mut state = use_signal(|| random_level18());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_vals: Vec<i32> = state.read().steppers.iter().map(|s| s.start_val).collect();
    let mut values = use_signal(move || initial_vals);
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let steppers: Vec<StepperInfo> = st.steppers.clone();
    let target_stepper = st.target_stepper;
    let mode = st.mode;
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    drop(st);

    let stepper_count = steppers.len();
    let is_wrong = wrong();
    let viewport_style = super::viewport_style(&bg(), false);
    let cur_vals: Vec<i32> = values.read().clone();

    let target_label = steppers[target_stepper].label.clone();
    let target_val = steppers[target_stepper].target_val;

    let instruction = match mode {
        1 => {
            let ord = ordinal(target_stepper + 1);
            format!("Set the {} stepper to {}", ord, target_val)
        }
        _ => {
            if stepper_count == 1 {
                format!("Set to {}", target_val)
            } else {
                format!("Set \"{}\" to {}", target_label, target_val)
            }
        }
    };

    let stepper_h = 70.0;
    let card_h = stepper_count as f32 * stepper_h + 100.0;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; box-sizing: border-box;",
        card_x, card_y, card_w
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth via UINode tree
    let stepper_nodes: Vec<UINode> = steppers.iter().enumerate().map(|(i, s)| {
        let cv = cur_vals.get(i).copied().unwrap_or(s.start_val);
        let row_y = 40.0 + i as f32 * stepper_h;
        let mut node = UINode::Stepper(
            Visual::new(&s.label, Rect::new(card_x + 16.0, card_y + row_y, card_w - 32.0, stepper_h)),
            StepperState {
                min: s.min,
                max: s.max,
                step: s.step,
                current_val: cv,
                target_val: s.target_val,
                minus_label: format!("\u{2212}: {}", s.label),
                plus_label: format!("+: {}", s.label),
            },
        );
        if i == target_stepper {
            node.visual_mut().is_target = true;
        }
        node
    }).collect();
    let tree = ui_node::form(
        Rect::new(card_x, card_y, card_w, card_h),
        "Submit",
        stepper_nodes,
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
                    "Level 7"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Stepper"
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
                        style: "margin: 0 0 16px 0; font-size: 14px; color: #374151; font-weight: 500;",
                        "{instruction}"
                    }

                    for si in 0..stepper_count {
                        {
                            let s = steppers[si].clone();
                            let val = cur_vals.get(si).copied().unwrap_or(s.start_val);
                            let at_min = val <= s.min;
                            let at_max = val >= s.max;
                            let is_last = si == stepper_count - 1;
                            let mb = if is_last { "0" } else { "12px" };

                            let minus_opacity = if at_min { "0.3" } else { "1" };
                            let plus_opacity = if at_max { "0.3" } else { "1" };

                            // Style variants
                            let (btn_style_minus, btn_style_plus, val_style, row_style) = match s.style {
                                // Style 0: pill buttons
                                0 => {
                                    let btn_base = format!("width: 36px; height: 36px; border-radius: 50%; border: none; font-size: 18px; font-weight: 700; cursor: pointer; display: flex; align-items: center; justify-content: center; font-family: system-ui, sans-serif; transition: opacity 0.1s;");
                                    let minus = format!("{} background: #f3f4f6; color: #374151; opacity: {};", btn_base, minus_opacity);
                                    let plus = format!("{} background: {}; color: white; opacity: {};", btn_base, s.accent, plus_opacity);
                                    let v = "font-size: 20px; font-weight: 700; color: #111827; min-width: 48px; text-align: center; font-family: monospace;".to_string();
                                    let r = "display: flex; align-items: center; gap: 12px; justify-content: center;".to_string();
                                    (minus, plus, v, r)
                                }
                                // Style 1: outlined buttons
                                1 => {
                                    let btn_base = format!("width: 32px; height: 32px; border-radius: 6px; font-size: 16px; font-weight: 700; cursor: pointer; display: flex; align-items: center; justify-content: center; font-family: system-ui, sans-serif; transition: opacity 0.1s;");
                                    let minus = format!("{} background: white; color: {}; border: 2px solid {}; opacity: {};", btn_base, s.accent, s.accent, minus_opacity);
                                    let plus = format!("{} background: white; color: {}; border: 2px solid {}; opacity: {};", btn_base, s.accent, s.accent, plus_opacity);
                                    let v = format!("font-size: 18px; font-weight: 600; color: {}; min-width: 44px; text-align: center; font-family: monospace;", s.accent);
                                    let r = "display: flex; align-items: center; gap: 10px; justify-content: center;".to_string();
                                    (minus, plus, v, r)
                                }
                                // Style 2: compact inline
                                _ => {
                                    let btn_base = "width: 28px; height: 28px; border-radius: 4px; border: 1px solid #d1d5db; font-size: 14px; font-weight: 700; cursor: pointer; display: flex; align-items: center; justify-content: center; font-family: system-ui, sans-serif; transition: opacity 0.1s;".to_string();
                                    let minus = format!("{} background: #f9fafb; color: #374151; opacity: {};", btn_base, minus_opacity);
                                    let plus = format!("{} background: #f9fafb; color: #374151; opacity: {};", btn_base, plus_opacity);
                                    let v = "font-size: 15px; font-weight: 600; color: #111827; min-width: 36px; text-align: center; font-family: monospace; padding: 4px 8px; border: 1px solid #e5e7eb; border-radius: 4px;".to_string();
                                    let r = "display: flex; align-items: center; gap: 6px; justify-content: center;".to_string();
                                    (minus, plus, v, r)
                                }
                            };

                            let smin = s.min;
                            let smax = s.max;
                            let sstep = s.step;

                            rsx! {
                                div {
                                    style: "margin-bottom: {mb};",

                                    // Label
                                    div {
                                        style: "font-size: 13px; font-weight: 500; color: #374151; margin-bottom: 8px;",
                                        "{s.label}"
                                    }

                                    // Stepper row
                                    div {
                                        style: "{row_style}",

                                        // Minus button
                                        button {
                                            class: if si == target_stepper { "target" } else { "" },
                                            "data-label": "\u{2212}: {s.label}",
                                            style: "{btn_style_minus}",
                                            tabindex: "-1",
                                            disabled: at_min,
                                            onclick: move |_| {
                                                let mut v = values.write();
                                                if let Some(val) = v.get_mut(si) {
                                                    *val = (*val - sstep).max(smin);
                                                }
                                            },
                                            "\u{2212}"
                                        }

                                        // Value display
                                        span {
                                            style: "{val_style}",
                                            "{val}"
                                        }

                                        // Plus button
                                        button {
                                            class: if si == target_stepper { "target" } else { "" },
                                            "data-label": "+: {s.label}",
                                            style: "{btn_style_plus}",
                                            tabindex: "-1",
                                            disabled: at_max,
                                            onclick: move |_| {
                                                let mut v = values.write();
                                                if let Some(val) = v.get_mut(si) {
                                                    *val = (*val + sstep).min(smax);
                                                }
                                            },
                                            "+"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        class: "target",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 16px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let v = values.read().get(target_stepper).copied().unwrap_or(0);
                            if v == target_val {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level18();
                                let new_vals: Vec<i32> = new_st.steppers.iter().map(|s| s.start_val).collect();
                                state.set(new_st);
                                values.set(new_vals);
                                wrong.set(false);
                            } else {
                                wrong.set(true);
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                    wrong.set(false);
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
                target_w: card_w,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
