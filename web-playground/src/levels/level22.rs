use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, describe_position};

struct DialogScenario {
    title: &'static str,
    message: &'static str,
    buttons: &'static [&'static str],
}

const SCENARIOS: &[DialogScenario] = &[
    DialogScenario { title: "Delete Account", message: "Are you sure you want to delete your account? This action cannot be undone.", buttons: &["Delete", "Cancel"] },
    DialogScenario { title: "Unsaved Changes", message: "You have unsaved changes. Do you want to save before leaving?", buttons: &["Save", "Discard", "Cancel"] },
    DialogScenario { title: "Confirm Purchase", message: "You are about to purchase this item for $29.99. Proceed?", buttons: &["Buy Now", "Cancel"] },
    DialogScenario { title: "Log Out", message: "Are you sure you want to log out of your account?", buttons: &["Log Out", "Cancel"] },
    DialogScenario { title: "Cancel Subscription", message: "Your subscription will end at the current billing period. Continue?", buttons: &["Yes, Cancel", "Keep Subscription"] },
    DialogScenario { title: "Clear Data", message: "This will permanently delete all your local data and preferences.", buttons: &["Clear All", "Cancel"] },
    DialogScenario { title: "Send Report", message: "Submit this report to the administrator for review?", buttons: &["Send", "Cancel"] },
    DialogScenario { title: "Remove Item", message: "Remove this item from your cart?", buttons: &["Remove", "Keep"] },
    DialogScenario { title: "Share Document", message: "Share this document with all team members?", buttons: &["Share", "Cancel"] },
    DialogScenario { title: "Reset Password", message: "A password reset link will be sent to your email address.", buttons: &["Send Link", "Cancel"] },
    DialogScenario { title: "Enable Notifications", message: "Allow this application to send you push notifications?", buttons: &["Allow", "Don't Allow"] },
    DialogScenario { title: "Update Available", message: "A new version is available. Would you like to update now?", buttons: &["Update", "Later", "Skip"] },
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

struct Level22State {
    scenario_idx: usize,
    target_button: usize,
    style: u8,
    accent: String,
    modal_w: f32,
    modal_x: f32,
    modal_y: f32,
    has_close: bool,
    target_is_close: bool,
}

fn random_level22() -> Level22State {
    let mut rng = fresh_rng();
    let scenario_idx = rng.random_range(0..SCENARIOS.len());
    let scenario = &SCENARIOS[scenario_idx];
    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();
    let modal_w = rng.random_range(320.0..=440.0f32);
    let modal_h = 220.0;
    let margin = 60.0;
    let modal_x = rng.random_range(margin..(Position::VIEWPORT - modal_w - margin).max(margin + 1.0));
    let modal_y = rng.random_range(margin..(Position::VIEWPORT - modal_h - margin).max(margin + 1.0));

    let has_close = rng.random_bool(0.5);

    // Pick target: either a button or the close X
    let total_targets = scenario.buttons.len() + if has_close { 1 } else { 0 };
    let target_idx = rng.random_range(0..total_targets);
    let target_is_close = has_close && target_idx == scenario.buttons.len();
    let target_button = if target_is_close { 0 } else { target_idx };

    Level22State { scenario_idx, target_button, style, accent, modal_w, modal_x, modal_y, has_close, target_is_close }
}

#[component]
pub fn Level22() -> Element {
    let mut state = use_signal(|| random_level22());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let scenario = &SCENARIOS[st.scenario_idx];
    let title = scenario.title;
    let message = scenario.message;
    let buttons: Vec<&str> = scenario.buttons.to_vec();
    let target_button = st.target_button;
    let target_is_close = st.target_is_close;
    let style = st.style;
    let accent = st.accent.clone();
    let modal_w = st.modal_w;
    let modal_x = st.modal_x;
    let modal_y = st.modal_y;
    let has_close = st.has_close;
    drop(st);

    let btn_count = buttons.len();
    let is_wrong = wrong();

    let target_label = if target_is_close {
        "the close button (X)".to_string()
    } else {
        format!("\"{}\"", buttons[target_button])
    };
    let instruction = format!("Click {}", target_label);

    // Modal styling
    let border_radius = match style { 0 => "16px", 1 => "8px", _ => "12px" };
    let shadow = match style {
        0 => "0 20px 60px rgba(0,0,0,0.5)",
        1 => "0 4px 24px rgba(0,0,0,0.4)",
        _ => "0 8px 32px rgba(0,0,0,0.45)",
    };
    let modal_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; background: white; border-radius: {}; box-shadow: {}; font-family: system-ui, sans-serif; z-index: 20; box-sizing: border-box; padding: 24px;",
        modal_x, modal_y, modal_w, border_radius, shadow
    );

    // Ground truth
    let modal_h_est = 220.0f32;
    let position_desc = describe_position(modal_x, modal_y, modal_w, modal_h_est);
    let buttons_desc: String = buttons.iter().enumerate().map(|(i, b)| {
        let marker = if !target_is_close && i == target_button { " (TARGET)" } else { "" };
        format!("\"{}\"{}",  b, marker)
    }).collect::<Vec<_>>().join(", ");
    let close_desc = if has_close {
        if target_is_close { ", close X (TARGET)" } else { ", close X" }
    } else { "" };
    let description = format!(
        "modal dialog, title: \"{}\", buttons: [{}{}], style: {}, at {}",
        title, buttons_desc, close_desc,
        match style { 0 => "rounded", 1 => "sharp", _ => "standard" },
        position_desc
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
                    "Level 22"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Modal"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "width: 1024px; height: 1024px; background: {bg}; position: relative; border: 1px solid #2a2a4a; overflow: hidden; transition: background 0.4s;",

                // Fake background content (dimmed by overlay)
                div {
                    style: "position: absolute; left: 200px; top: 300px; width: 300px; background: white; border-radius: 12px; padding: 24px; box-shadow: 0 2px 12px rgba(0,0,0,0.2); font-family: system-ui, sans-serif; z-index: 1;",
                    div { style: "width: 60%; height: 12px; background: #e5e7eb; border-radius: 6px; margin-bottom: 12px;" }
                    div { style: "width: 90%; height: 8px; background: #f3f4f6; border-radius: 4px; margin-bottom: 8px;" }
                    div { style: "width: 75%; height: 8px; background: #f3f4f6; border-radius: 4px; margin-bottom: 8px;" }
                    div { style: "width: 85%; height: 8px; background: #f3f4f6; border-radius: 4px; margin-bottom: 16px;" }
                    div { style: "width: 100px; height: 32px; background: #e5e7eb; border-radius: 6px;" }
                }

                // Backdrop overlay
                div {
                    style: "position: absolute; inset: 0; background: rgba(0,0,0,0.5); z-index: 10;",
                }

                // Instruction above modal
                div {
                    style: "position: absolute; left: 0; right: 0; top: 16px; text-align: center; z-index: 30;",
                    div {
                        style: "display: inline-block; background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 8px; color: white; font-size: 14px; font-weight: 500;",
                        "{instruction}"
                    }
                }

                // Modal dialog
                div {
                    style: "{modal_style}",

                    // Close button
                    if has_close {
                        {
                            let wrong_bg = if is_wrong && target_is_close { "#fecaca" } else { "transparent" };
                            rsx! {
                                button {
                                    class: if target_is_close { "target" } else { "" },
                                    "data-label": "close",
                                    style: "position: absolute; top: 12px; right: 12px; width: 28px; height: 28px; background: {wrong_bg}; border: none; border-radius: 6px; font-size: 18px; color: #9ca3af; cursor: pointer; display: flex; align-items: center; justify-content: center; font-family: system-ui, sans-serif;",
                                    tabindex: "-1",
                                    onclick: move |_| {
                                        if target_is_close {
                                            score.set(score() + 1);
                                            bg.set(random_canvas_bg());
                                            state.set(random_level22());
                                            wrong.set(false);
                                        } else {
                                            wrong.set(true);
                                            spawn(async move {
                                                gloo_timers::future::TimeoutFuture::new(600).await;
                                                wrong.set(false);
                                            });
                                        }
                                    },
                                    "\u{2715}"
                                }
                            }
                        }
                    }

                    // Title
                    h3 {
                        style: "margin: 0 0 12px 0; font-size: 18px; color: #111827; font-weight: 600;",
                        "{title}"
                    }

                    // Message
                    p {
                        style: "margin: 0 0 24px 0; font-size: 14px; color: #6b7280; line-height: 1.5;",
                        "{message}"
                    }

                    // Buttons row
                    div {
                        style: "display: flex; gap: 8px; justify-content: flex-end;",

                        for bi in 0..btn_count {
                            {
                                let label = buttons[bi];
                                let is_primary = bi == 0;
                                let accent_c = accent.clone();

                                let btn_bg = if is_wrong && !target_is_close && bi == target_button {
                                    "#ef4444".to_string()
                                } else if is_primary {
                                    accent_c
                                } else {
                                    "#f3f4f6".to_string()
                                };
                                let btn_color = if is_wrong && !target_is_close && bi == target_button {
                                    "white".to_string()
                                } else if is_primary {
                                    "white".to_string()
                                } else {
                                    "#374151".to_string()
                                };
                                let btn_border = if is_primary { "none" } else { "1px solid #e5e7eb" };
                                let btn_radius = match style { 0 => "10px", 1 => "4px", _ => "6px" };

                                rsx! {
                                    button {
                                        class: if !target_is_close && bi == target_button { "target" } else { "" },
                                        "data-label": "{label}",
                                        style: "padding: 8px 18px; background: {btn_bg}; color: {btn_color}; border: {btn_border}; border-radius: {btn_radius}; font-size: 14px; font-weight: 500; cursor: pointer; font-family: system-ui, sans-serif; transition: background 0.15s;",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            if !target_is_close && bi == target_button {
                                                score.set(score() + 1);
                                                bg.set(random_canvas_bg());
                                                state.set(random_level22());
                                                wrong.set(false);
                                            } else {
                                                wrong.set(true);
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                                    wrong.set(false);
                                                });
                                            }
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: modal_x,
                target_y: modal_y,
                target_w: modal_w,
                target_h: modal_h_est,
                steps: if target_is_close {
                    r#"[{"action":"click","target":"close"}]"#.to_string()
                } else {
                    format!(r#"[{{"action":"click","target":"{}"}}]"#, buttons[target_button])
                },
            }
        }
    }
}
