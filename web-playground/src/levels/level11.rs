use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg, ordinal};

const SLIDE_COLORS: &[&str] = &[
    "#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6",
    "#1abc9c", "#e67e22", "#34495e", "#c0392b", "#2980b9",
];

const SLIDE_WORDS: &[&str] = &[
    "ALPHA", "BRAVO", "DELTA", "ECHO", "FOXTROT",
    "GOLF", "HOTEL", "INDIA", "JULIET", "KILO",
    "LIMA", "MIKE", "OSCAR", "PAPA", "ROMEO",
    "SIERRA", "TANGO", "VICTOR", "WHISKEY", "ZULU",
];

// nav_type: 0=arrows, 1=dots, 2=arrows+dots, 3=numbered tabs, 4=ring dots, 5=auto-slide
struct Level11State {
    slides: Vec<(String, String)>, // (color, text)
    target_slide: usize,
    nav_type: u8,
    x: f32,
    y: f32,
}

fn random_level11() -> Level11State {
    let mut rng = fresh_rng();
    let slide_count = rng.random_range(3..=6usize);
    let nav_type = rng.random_range(0..6u8);

    let mut color_indices: Vec<usize> = (0..SLIDE_COLORS.len()).collect();
    let mut word_indices: Vec<usize> = (0..SLIDE_WORDS.len()).collect();
    let mut slides = Vec::with_capacity(slide_count);

    for _ in 0..slide_count {
        let ci = rng.random_range(0..color_indices.len());
        let color = SLIDE_COLORS[color_indices.remove(ci)].to_string();
        let wi = rng.random_range(0..word_indices.len());
        let text = SLIDE_WORDS[word_indices.remove(wi)].to_string();
        slides.push((color, text));
    }

    // Target is never slide 0 so the user always needs to navigate
    let target_slide = rng.random_range(1..slide_count);

    let card_w = 340.0;
    let card_h = 400.0;
    let pad = 80.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, pad, vp_w * 1.3, vp_h * 1.3);

    Level11State { slides, target_slide, nav_type, x, y }
}

#[component]
pub fn Level11() -> Element {
    let mut state = use_signal(|| random_level11());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut current = use_signal(|| 0usize);
    let mut input_text = use_signal(|| String::new());
    let mut wrong = use_signal(|| false);
    let mut auto_gen = use_signal(|| 0u32);

    // Auto-slide timer for nav_type 5
    use_effect(move || {
        let g = auto_gen();
        let st = state.read();
        let nt = st.nav_type;
        let count = st.slides.len();
        drop(st);

        if nt == 5 {
            spawn(async move {
                loop {
                    gloo_timers::future::TimeoutFuture::new(2500).await;
                    if auto_gen() != g { break; }
                    let next = (current() + 1) % count;
                    current.set(next);
                }
            });
        }
    });

    let st = state.read();
    let slides: Vec<(String, String)> = st.slides.clone();
    let target_slide = st.target_slide;
    let nav_type = st.nav_type;
    let card_x = st.x;
    let card_y = st.y;
    drop(st);

    let slide_count = slides.len();
    let cur = current();
    let target_ord = ordinal(target_slide + 1);
    let target_text = slides[target_slide].1.clone();
    let cur_color = slides[cur.min(slide_count - 1)].0.clone();
    let cur_text = slides[cur.min(slide_count - 1)].1.clone();
    let input_val = input_text.read().clone();
    let is_wrong = wrong();
    let viewport_style = super::viewport_style(&bg(), true);

    let left_opacity = if cur == 0 { "0.3" } else { "0.8" };
    let right_opacity = if cur >= slide_count - 1 { "0.3" } else { "0.8" };

    let _nav_desc = match nav_type {
        0 => "arrows",
        1 => "dots",
        2 => "arrows+dots",
        3 => "numbered tabs",
        4 => "ring dots",
        _ => "auto-slide",
    };

    // Build UINode tree for ground truth
    // The carousel has a text input and submit button as a form
    let tree = ui_node::form(
        Rect::new(card_x, card_y, 340.0, 400.0),
        "Submit",
        vec![
            ui_node::text_input(
                "Enter slide text",
                Rect::new(card_x + 20.0, card_y + 300.0, 260.0, 36.0),
                "Enter slide text...",
                &target_text,
            ),
        ],
    );
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 20px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); width: 300px; font-family: system-ui, sans-serif;",
        card_x, card_y
    );

    let border_color = if is_wrong { "#ef4444" } else { "#d1d5db" };
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

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
                    "Level 17"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Carousel"
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
                        style: "margin: 0 0 12px 0; font-size: 15px; color: #374151; font-weight: 500;",
                        "Enter the text from the "
                        span { style: "font-weight: 700; color: #111;", "{target_ord}" }
                        " slide"
                    }

                    // Numbered tabs — above the slide (nav_type 3)
                    if nav_type == 3 {
                        div {
                            style: "display: flex; gap: 4px; margin-bottom: 8px;",
                            for si in 0..slide_count {
                                {
                                    let is_cur = si == cur;
                                    let tab_bg = if is_cur { "#4f46e5" } else { "#e5e7eb" };
                                    let tab_color = if is_cur { "white" } else { "#6b7280" };
                                    rsx! {
                                        button {
                                            class: "target",
                                            style: "width: 32px; height: 28px; background: {tab_bg}; color: {tab_color}; border: none; border-radius: 4px; font-size: 13px; font-weight: 600; cursor: pointer; font-family: monospace; transition: background 0.15s;",
                                            tabindex: "-1",
                                            onclick: move |_| current.set(si),
                                            "{si + 1}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Carousel slide with overlaid arrows
                    div {
                        style: "position: relative; width: 260px; height: 150px; margin-bottom: 8px;",

                        // Slide
                        div {
                            style: "width: 100%; height: 100%; background: {cur_color}; display: flex; align-items: center; justify-content: center; border-radius: 8px; user-select: none;",
                            span {
                                style: "color: white; font-size: 28px; font-weight: 700; letter-spacing: 2px; text-shadow: 0 2px 4px rgba(0,0,0,0.3);",
                                "{cur_text}"
                            }
                        }

                        // Left arrow (nav_type 0 or 2)
                        if nav_type == 0 || nav_type == 2 {
                            button {
                                class: "target",
                                style: "position: absolute; left: 6px; top: 50%; transform: translateY(-50%); width: 28px; height: 28px; background: rgba(0,0,0,0.4); color: white; border: none; border-radius: 50%; font-size: 14px; cursor: pointer; display: flex; align-items: center; justify-content: center; opacity: {left_opacity}; transition: opacity 0.15s;",
                                tabindex: "-1",
                                disabled: cur == 0,
                                onclick: move |_| current.set(current().saturating_sub(1)),
                                "\u{2190}"
                            }
                        }

                        // Right arrow (nav_type 0 or 2)
                        if nav_type == 0 || nav_type == 2 {
                            button {
                                class: "target",
                                style: "position: absolute; right: 6px; top: 50%; transform: translateY(-50%); width: 28px; height: 28px; background: rgba(0,0,0,0.4); color: white; border: none; border-radius: 50%; font-size: 14px; cursor: pointer; display: flex; align-items: center; justify-content: center; opacity: {right_opacity}; transition: opacity 0.15s;",
                                tabindex: "-1",
                                disabled: cur >= slide_count - 1,
                                onclick: move |_| current.set((current() + 1).min(slide_count - 1)),
                                "\u{2192}"
                            }
                        }
                    }

                    // Dot indicators (nav_type 1 or 2)
                    if nav_type == 1 || nav_type == 2 {
                        div {
                            style: "display: flex; gap: 6px; justify-content: center; margin-bottom: 8px;",
                            for si in 0..slide_count {
                                {
                                    let is_cur = si == cur;
                                    let dot_bg = if is_cur { "#4f46e5" } else { "#d1d5db" };
                                    let dot_size = if is_cur { "10px" } else { "8px" };
                                    rsx! {
                                        div {
                                            class: "target",
                                            style: "width: {dot_size}; height: {dot_size}; border-radius: 50%; background: {dot_bg}; cursor: pointer; transition: all 0.15s;",
                                            onclick: move |_| current.set(si),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Ring dot indicators (nav_type 4)
                    if nav_type == 4 {
                        div {
                            style: "display: flex; gap: 8px; justify-content: center; margin-bottom: 8px;",
                            for si in 0..slide_count {
                                {
                                    let is_cur = si == cur;
                                    let ring_bg = if is_cur { "#4f46e5" } else { "transparent" };
                                    let ring_border = if is_cur { "#4f46e5" } else { "#9ca3af" };
                                    rsx! {
                                        button {
                                            class: "target",
                                            style: "width: 14px; height: 14px; border-radius: 50%; background: {ring_bg}; border: 2px solid {ring_border}; cursor: pointer; transition: all 0.15s; padding: 0;",
                                            tabindex: "-1",
                                            onclick: move |_| current.set(si),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Auto-slide indicator (nav_type 5)
                    if nav_type == 5 {
                        div {
                            style: "display: flex; gap: 6px; justify-content: center; align-items: center; margin-bottom: 8px;",
                            span {
                                style: "background: #4f46e5; color: white; padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: 700; font-family: monospace; letter-spacing: 1px;",
                                "AUTO"
                            }
                            for si in 0..slide_count {
                                {
                                    let is_cur = si == cur;
                                    let dot_bg = if is_cur { "#4f46e5" } else { "#d1d5db" };
                                    let dot_size = if is_cur { "8px" } else { "6px" };
                                    rsx! {
                                        div {
                                            style: "width: {dot_size}; height: {dot_size}; border-radius: 50%; background: {dot_bg}; transition: all 0.3s;",
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Slide counter
                    p {
                        style: "margin: 0 0 10px 0; font-size: 12px; color: #9ca3af; text-align: center; font-family: monospace;",
                        "{cur + 1} / {slide_count}"
                    }

                    // Text input
                    input {
                        r#type: "text",
                        tabindex: "-1",
                        class: "target",
                        style: "width: 100%; padding: 8px 12px; border: 1px solid {border_color}; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; background: white; color: #111; box-sizing: border-box; transition: border-color 0.15s;",
                        placeholder: "Enter slide text...",
                        value: "{input_val}",
                        oninput: move |e: Event<FormData>| {
                            input_text.set(e.value());
                        },
                    }

                    // Submit
                    button {
                        class: "target",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; margin-top: 10px; box-sizing: border-box; transition: background 0.15s;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let val = input_text.read().clone();
                            if val.eq_ignore_ascii_case(&target_text) {
                                score.set(score() + 1);
                                auto_gen.set(auto_gen() + 1);
                                bg.set(random_canvas_bg());
                                state.set(random_level11());
                                current.set(0);
                                input_text.set(String::new());
                                wrong.set(false);
                                document::eval("document.activeElement?.blur()");
                            } else {
                                wrong.set(true);
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(400).await;
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
                target_w: 340.0,
                target_h: 400.0,
                tree: Some(tree.clone()),
            }
        }
    }
}
