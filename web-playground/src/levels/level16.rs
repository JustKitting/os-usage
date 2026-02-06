use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect, Visual, UINode, SliderState};
use super::{fresh_rng, random_canvas_bg};

const SLIDER_LABELS: &[&str] = &[
    "Volume", "Brightness", "Contrast", "Opacity", "Speed",
    "Quality", "Zoom", "Balance", "Intensity", "Threshold",
    "Temperature", "Saturation", "Sharpness", "Exposure", "Gain",
];

const TRACK_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

struct SliderInfo {
    label: String,
    min: i32,
    max: i32,
    step: i32,
    target_val: i32,
    current_val: i32,
    track_color: String,
    show_ticks: bool,
}

struct Level16State {
    sliders: Vec<SliderInfo>,
    target_slider: usize,
    mode: u8, // 0=by label, 1=by ordinal
    x: f32,
    y: f32,
    card_w: f32,
}

fn random_level16() -> Level16State {
    let mut rng = fresh_rng();
    let count = rng.random_range(1..=4usize);

    let mut label_pool: Vec<usize> = (0..SLIDER_LABELS.len()).collect();
    let mut color_pool: Vec<usize> = (0..TRACK_COLORS.len()).collect();
    let mut sliders = Vec::new();

    for _ in 0..count {
        let li = rng.random_range(0..label_pool.len());
        let label = SLIDER_LABELS[label_pool.remove(li)].to_string();

        let ci = rng.random_range(0..color_pool.len());
        let track_color = TRACK_COLORS[color_pool.remove(ci)].to_string();

        // Pick a range style
        let (min, max, step) = match rng.random_range(0..4u8) {
            0 => (0, 100, 1),
            1 => (0, 100, 5),
            2 => (0, 10, 1),
            _ => (0, 255, 1),
        };

        let steps = (max - min) / step;
        let target_step = rng.random_range(1..steps); // avoid endpoints
        let target_val = min + target_step * step;

        // Current value: either min or a random different value
        let current_val = if rng.random_bool(0.5) {
            min
        } else {
            let mut cv = target_val;
            while cv == target_val {
                cv = min + rng.random_range(0..=steps) * step;
            }
            cv
        };

        let show_ticks = step >= 5 || max <= 10;

        sliders.push(SliderInfo {
            label, min, max, step, target_val, current_val, track_color, show_ticks,
        });
    }

    let target_slider = rng.random_range(0..count);
    let mode = if count == 1 { 0 } else { rng.random_range(0..2u8) };

    let card_w = rng.random_range(300.0..=450.0f32);
    let slider_h = 72.0;
    let card_h = count as f32 * slider_h + 120.0;
    let margin = 50.0;
    let (x, y) = super::safe_position(&mut rng, card_w, card_h, margin);

    Level16State { sliders, target_slider, mode, x, y, card_w }
}

#[component]
pub fn Level16() -> Element {
    let mut state = use_signal(|| random_level16());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_vals: Vec<i32> = state.read().sliders.iter().map(|s| s.current_val).collect();
    let mut values = use_signal(move || initial_vals);
    let mut wrong = use_signal(|| false);
    let mut drag_idx = use_signal(|| Option::<usize>::None);

    let st = state.read();
    let sliders: Vec<SliderInfo> = st.sliders.iter().map(|s| SliderInfo {
        label: s.label.clone(),
        min: s.min,
        max: s.max,
        step: s.step,
        target_val: s.target_val,
        current_val: s.current_val,
        track_color: s.track_color.clone(),
        show_ticks: s.show_ticks,
    }).collect();
    let target_slider = st.target_slider;
    let mode = st.mode;
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    drop(st);

    let slider_count = sliders.len();
    let is_wrong = wrong();
    let viewport_style = super::viewport_style(&bg(), false);
    let cur_vals: Vec<i32> = values.read().clone();
    let cur_drag = drag_idx();

    let target_label = sliders[target_slider].label.clone();
    let target_val = sliders[target_slider].target_val;
    let instruction = match mode {
        1 => {
            let ord = super::ordinal(target_slider + 1);
            format!("Set the {} slider to {}", ord, target_val)
        }
        _ => format!("Set \"{}\" to {}", target_label, target_val),
    };

    let slider_h = 72.0;
    let card_h = slider_count as f32 * slider_h + 120.0;
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; box-sizing: border-box;",
        card_x, card_y, card_w
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    let track_w = card_w - 32.0; // padding
    let thumb_w: f32 = 18.0;
    let usable_w = track_w - thumb_w;

    // Build UINode tree for ground truth
    let slider_nodes: Vec<UINode> = sliders.iter().enumerate().map(|(i, s)| {
        let is_target = i == target_slider;
        let val = cur_vals.get(i).copied().unwrap_or(s.current_val);
        let ratio = if s.max > s.min { (val - s.min) as f32 / (s.max - s.min) as f32 } else { 0.0 };
        let thumb_left = ratio * usable_w;
        let target_ratio = if s.max > s.min { (s.target_val - s.min) as f32 / (s.max - s.min) as f32 } else { 0.0 };
        let target_thumb_left = target_ratio * usable_w;
        let row_y = 60.0 + i as f32 * slider_h;

        let mut node = UINode::Slider(
            Visual::new(&s.label, Rect::new(card_x + 16.0, card_y + row_y, track_w, 28.0))
                .color(&s.track_color),
            SliderState {
                min: s.min,
                max: s.max,
                step: s.step,
                current_val: val,
                target_val: s.target_val,
                thumb_rect: Rect::new(card_x + 16.0 + thumb_left, card_y + row_y + 4.0, thumb_w, 20.0),
                target_thumb_rect: Rect::new(card_x + 16.0 + target_thumb_left, card_y + row_y + 4.0, thumb_w, 20.0),
            },
        );
        if is_target {
            node.visual_mut().is_target = true;
        }
        node
    }).collect();

    let tree = ui_node::form(
        Rect::new(card_x, card_y, card_w, card_h),
        "Submit",
        slider_nodes,
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
                    "Level 6"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Slider"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            // Canvas
            div {
                id: "viewport",
                style: "{viewport_style}",

                div {
                    style: "{card_style}",

                    // Instruction
                    p {
                        style: "margin: 0 0 16px 0; font-size: 14px; color: #374151; font-weight: 500;",
                        "{instruction}"
                    }

                    // Sliders
                    for si in 0..slider_count {
                        {
                            let s = &sliders[si];
                            let label = s.label.clone();
                            let min = s.min;
                            let max = s.max;
                            let step = s.step;
                            let track_color = s.track_color.clone();
                            let show_ticks = s.show_ticks;
                            let val = cur_vals.get(si).copied().unwrap_or(min);
                            let ratio = if max > min { (val - min) as f32 / (max - min) as f32 } else { 0.0 };
                            let thumb_left = ratio * usable_w;
                            let fill_w = thumb_left + thumb_w / 2.0;
                            let target_ratio = if max > min { (s.target_val - min) as f32 / (max - min) as f32 } else { 0.0 };
                            let target_thumb_left = target_ratio * usable_w;
                            let is_target_slider = si == target_slider;

                            rsx! {
                                div {
                                    style: "margin-bottom: 16px;",

                                    // Label + value
                                    div {
                                        style: "display: flex; justify-content: space-between; margin-bottom: 6px;",
                                        span {
                                            style: "font-size: 12px; color: #374151; font-weight: 500;",
                                            "{label}"
                                        }
                                        span {
                                            style: "font-size: 12px; color: #6b7280; font-family: monospace; min-width: 32px; text-align: right;",
                                            "{val}"
                                        }
                                    }

                                    // Track container
                                    div {
                                        style: "position: relative; height: 28px; cursor: pointer;",
                                        tabindex: "-1",

                                        // Track background
                                        div {
                                            style: "position: absolute; top: 10px; left: 0; right: 0; height: 8px; background: #e5e7eb; border-radius: 4px; pointer-events: none;",
                                        }

                                        // Track fill
                                        div {
                                            style: "position: absolute; top: 10px; left: 0; width: {fill_w}px; height: 8px; background: {track_color}; border-radius: 4px; pointer-events: none; transition: width 0.05s;",
                                        }

                                        // Tick marks
                                        if show_ticks {
                                            {
                                                let steps = (max - min) / step;
                                                rsx! {
                                                    for ti in 0..=steps {
                                                        {
                                                            let t_ratio = ti as f32 / steps as f32;
                                                            let t_left = t_ratio * usable_w + thumb_w / 2.0;
                                                            rsx! {
                                                                div {
                                                                    style: "position: absolute; top: 22px; left: {t_left}px; width: 1px; height: 6px; background: #d1d5db; pointer-events: none;",
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Thumb
                                        div {
                                            style: "position: absolute; top: 4px; left: {thumb_left}px; width: {thumb_w}px; height: 20px; background: white; border: 2px solid {track_color}; border-radius: 10px; box-shadow: 0 1px 4px rgba(0,0,0,0.2); pointer-events: none; transition: left 0.05s;",
                                        }

                                        // Ground truth drag markers
                                        if is_target_slider {
                                            div {
                                                class: "target",
                                                "data-label": "drag-from: {label}",
                                                style: "position: absolute; top: 4px; left: {thumb_left}px; width: {thumb_w}px; height: 20px; pointer-events: none;",
                                            }
                                            div {
                                                class: "target",
                                                "data-label": "drag-to: {label}",
                                                style: "position: absolute; top: 4px; left: {target_thumb_left}px; width: {thumb_w}px; height: 20px; pointer-events: none;",
                                            }
                                        }

                                        // Invisible hit area for mouse events
                                        div {
                                            style: "position: absolute; inset: 0; z-index: 1;",
                                            onmousedown: move |e: Event<MouseData>| {
                                                e.prevent_default();
                                                drag_idx.set(Some(si));
                                                let coords = e.element_coordinates();
                                                let mx = coords.x as f32;
                                                let raw_ratio = ((mx - thumb_w / 2.0) / usable_w).clamp(0.0, 1.0);
                                                let steps = (max - min) / step;
                                                let snapped = min + (raw_ratio * steps as f32).round() as i32 * step;
                                                let mut v = values.write();
                                                if let Some(val) = v.get_mut(si) {
                                                    *val = snapped.clamp(min, max);
                                                }
                                            },
                                            onmousemove: move |e: Event<MouseData>| {
                                                if cur_drag == Some(si) {
                                                    let coords = e.element_coordinates();
                                                    let mx = coords.x as f32;
                                                    let raw_ratio = ((mx - thumb_w / 2.0) / usable_w).clamp(0.0, 1.0);
                                                    let steps = (max - min) / step;
                                                    let snapped = min + (raw_ratio * steps as f32).round() as i32 * step;
                                                    let mut v = values.write();
                                                    if let Some(val) = v.get_mut(si) {
                                                        *val = snapped.clamp(min, max);
                                                    }
                                                }
                                            },
                                            onmouseup: move |_| {
                                                drag_idx.set(None);
                                            },
                                            onmouseleave: move |_| {
                                                drag_idx.set(None);
                                            },
                                        }
                                    }

                                    // Min/max labels
                                    div {
                                        style: "display: flex; justify-content: space-between; margin-top: 2px;",
                                        span {
                                            style: "font-size: 10px; color: #9ca3af;",
                                            "{min}"
                                        }
                                        span {
                                            style: "font-size: 10px; color: #9ca3af;",
                                            "{max}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        class: "target",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 8px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let v = values.read().get(target_slider).copied().unwrap_or(0);
                            if v == target_val {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level16();
                                let new_vals: Vec<i32> = new_st.sliders.iter().map(|s| s.current_val).collect();
                                state.set(new_st);
                                values.set(new_vals);
                                wrong.set(false);
                                drag_idx.set(None);
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
