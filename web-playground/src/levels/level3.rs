use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, describe_position};

const WORDS: &[&str] = &[
    "hello", "world", "search", "login", "submit", "click", "enter",
    "password", "email", "username", "address", "phone", "name",
    "send", "save", "open", "close", "next", "back", "done",
    "start", "stop", "play", "pause", "edit", "delete", "copy",
    "paste", "undo", "redo", "find", "help", "home", "menu",
    "settings", "profile", "inbox", "chat", "share", "upload",
];

struct InputStyle {
    style: &'static str,
    label: &'static str,
    width: f32,
    height: f32,
}

const INPUT_STYLES: &[InputStyle] = &[
    InputStyle {
        style: "padding: 10px 14px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; width: 220px; background: white; color: #111;",
        label: "bordered rounded input",
        width: 250.0,
        height: 42.0,
    },
    InputStyle {
        style: "padding: 10px 4px; border: none; border-bottom: 2px solid #6366f1; font-size: 14px; font-family: system-ui, sans-serif; outline: none; width: 200px; background: transparent; color: white;",
        label: "underline input",
        width: 220.0,
        height: 42.0,
    },
    InputStyle {
        style: "padding: 10px 16px; border: 1px solid #e5e7eb; border-radius: 9999px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; width: 240px; background: #f9fafb; color: #111;",
        label: "pill search input",
        width: 270.0,
        height: 44.0,
    },
];

struct Level3State {
    word: String,
    x: f32,
    y: f32,
    style_idx: usize,
}

fn random_level3() -> Level3State {
    let mut rng = fresh_rng();
    let word_idx = rng.random_range(0..WORDS.len());
    let style_idx = rng.random_range(0..INPUT_STYLES.len());
    let is = &INPUT_STYLES[style_idx];
    let pad = 150.0;
    let x = rng.random_range(pad..(Position::VIEWPORT - is.width - pad).max(pad));
    let y = rng.random_range(pad..(Position::VIEWPORT - is.height - pad).max(pad));

    Level3State {
        word: WORDS[word_idx].to_string(),
        x,
        y,
        style_idx,
    }
}

#[component]
pub fn Level3() -> Element {
    let mut state = use_signal(|| random_level3());
    let mut score = use_signal(|| 0u32);
    let mut input_value = use_signal(|| String::new());
    let mut bg = use_signal(|| random_canvas_bg());

    let st = state.read();
    let target_word = st.word.clone();
    let input_x = st.x;
    let input_y = st.y;
    let style_idx = st.style_idx;
    let input_style = INPUT_STYLES[style_idx].style;
    let input_label = INPUT_STYLES[style_idx].label;
    drop(st);

    let pos_style = format!(
        "position: absolute; left: {input_x}px; top: {input_y}px;"
    );

    let input_w = INPUT_STYLES[style_idx].width;
    let input_h = INPUT_STYLES[style_idx].height;
    let position_desc = describe_position(input_x, input_y, input_w, input_h);
    let current_val = input_value.read().clone();
    let description = if current_val.is_empty() {
        format!("{} (text input), target: \"{}\", at {}", input_label, target_word, position_desc)
    } else {
        format!("{} (text input), target: \"{}\", current: \"{}\", at {}", input_label, target_word, current_val, position_desc)
    };

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
                    "Level 3"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Type: "
                }
                span {
                    style: "color: #f59e0b; font-size: 16px; font-weight: 600; font-family: monospace;",
                    "{target_word}"
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
                    style: "{pos_style}",
                    input {
                        class: "target",
                        r#type: "text",
                        tabindex: "-1",
                        style: "{input_style}",
                        placeholder: "Type here...",
                        value: "{input_value}",
                        oninput: move |e: Event<FormData>| {
                            let val = e.value();
                            input_value.set(val.clone());
                            if val == target_word {
                                score.set(score() + 1);
                                state.set(random_level3());
                                input_value.set(String::new());
                                bg.set(random_canvas_bg());
                                document::eval("document.activeElement?.blur()");
                            }
                        },
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: input_x,
                target_y: input_y,
                target_w: input_w,
                target_h: input_h,
                steps: format!(r#"[{{"action":"type","target":"Type here...","value":"{}"}}]"#, target_word),
            }
        }
    }
}
