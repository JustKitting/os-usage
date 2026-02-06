use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect, UINode};
use super::{fresh_rng, random_canvas_bg, ordinal};

const WORDS: &[&str] = &[
    "hello", "world", "search", "login", "submit", "click", "enter",
    "send", "save", "open", "close", "next", "back", "done",
    "start", "stop", "play", "pause", "edit", "delete", "copy",
    "find", "help", "home", "menu", "chat", "share", "test",
];

const INPUT_LABELS: &[&str] = &[
    "Username", "Email", "Password", "First name", "Last name",
    "Phone", "Address", "City", "Zip code", "Company",
    "Website", "Bio", "Title", "Comment", "Search",
];

struct Level7State {
    word: String,
    target: usize,
    labels: Vec<String>,
    x: f32,
    y: f32,
}

fn random_level7() -> Level7State {
    let mut rng = fresh_rng();
    let count = rng.random_range(3..=5usize);

    let mut indices: Vec<usize> = (0..INPUT_LABELS.len()).collect();
    let mut labels = Vec::with_capacity(count);
    for _ in 0..count {
        let i = rng.random_range(0..indices.len());
        labels.push(INPUT_LABELS[indices.remove(i)].to_string());
    }

    let word_idx = rng.random_range(0..WORDS.len());
    let word = WORDS[word_idx].to_string();
    let target = rng.random_range(0..count);

    let card_w = 340.0;
    let card_h = 70.0 + (count as f32 * 72.0);
    let pad = 80.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, pad, vp_w * 1.3, vp_h * 1.3);

    Level7State { word, target, labels, x, y }
}

#[component]
pub fn Level7() -> Element {
    let mut state = use_signal(|| random_level7());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut inputs = use_signal(|| vec![String::new(); 5]);
    let mut wrong_idx = use_signal(|| None::<usize>);

    let st = state.read();
    let word = st.word.clone();
    let target = st.target;
    let labels = st.labels.clone();
    let card_x = st.x;
    let card_y = st.y;
    let input_count = labels.len();
    drop(st);

    let pressed = wrong_idx();
    let ordinal_str = ordinal(target + 1);
    let card_h = 70.0 + (labels.len() as f32 * 72.0);

    // Build UINode tree for ground truth
    let input_nodes: Vec<UINode> = labels.iter().enumerate().map(|(i, l)| {
        if i == target {
            ui_node::text_input(
                l.as_str(),
                Rect::new(card_x + 20.0, card_y + 50.0 + i as f32 * 72.0, 260.0, 36.0),
                "Type here...",
                &word,
            )
        } else {
            // Non-target text inputs: just describe them but don't mark as target
            UINode::TextInput(
                crate::ui_node::Visual::new(l.as_str(), Rect::new(card_x + 20.0, card_y + 50.0 + i as f32 * 72.0, 260.0, 36.0)),
                crate::ui_node::InputState {
                    placeholder: "Type here...".into(),
                    current_value: String::new(),
                    target_value: String::new(),
                },
            )
        }
    }).collect();
    let tree = ui_node::card(
        Rect::new(card_x, card_y, 340.0, card_h),
        input_nodes,
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
                    "Level 12"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Type into the right input"
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
                        style: "margin: 0 0 16px 0; font-size: 15px; color: #374151; font-weight: 500;",
                        "Enter "
                        span {
                            style: "font-weight: 700; color: #111; font-family: monospace;",
                            "\"{word}\""
                        }
                        " into the "
                        span {
                            style: "font-weight: 700; color: #111;",
                            "{ordinal_str}"
                        }
                        " input"
                    }

                    div {
                        style: "display: flex; flex-direction: column; gap: 12px;",
                        for (i, label) in labels.iter().enumerate() {
                            {
                                let is_wrong = pressed == Some(i);
                                let border_color = if is_wrong { "#ef4444" } else { "#d1d5db" };
                                let label_clone = label.clone();
                                let input_val = inputs.read()[i].clone();
                                let is_target = i == target;
                                let target_word = word.clone();
                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 4px;",
                                        label {
                                            style: "font-size: 13px; color: #6b7280; font-weight: 500;",
                                            "{label_clone}"
                                        }
                                        input {
                                            class: if is_target { "target" } else { "" },
                                            r#type: "text",
                                            tabindex: "-1",
                                            style: "padding: 8px 12px; border: 1px solid {border_color}; border-radius: 6px; font-size: 14px; font-family: system-ui, sans-serif; outline: none; background: white; color: #111; transition: border-color 0.15s;",
                                            placeholder: "Type here...",
                                            value: "{input_val}",
                                            oninput: move |e: Event<FormData>| {
                                                let val = e.value();
                                                inputs.write()[i] = val.clone();
                                                if val == target_word {
                                                    if is_target {
                                                        score.set(score() + 1);
                                                        wrong_idx.set(None);
                                                        bg.set(random_canvas_bg());
                                                        state.set(random_level7());
                                                        inputs.set(vec![String::new(); input_count]);
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
