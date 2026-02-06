use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, UINode, Visual, Rect};
use super::{fresh_rng, random_canvas_bg, ordinal};

const TAB_LABELS: &[&str] = &[
    "General", "Settings", "Profile", "Account", "Security",
    "Notifications", "Privacy", "Billing", "Appearance", "Advanced",
    "Help", "About", "Dashboard", "Analytics", "Reports",
];

const TAB_CONTENTS: &[&str] = &[
    "Configure your basic preferences and default settings for this application.",
    "Manage your account details, email address, and personal information here.",
    "Review your security settings including two-factor authentication and login history.",
    "Control how and when you receive notifications from the system.",
    "Adjust your privacy settings and manage data sharing preferences.",
    "View and manage your billing information, invoices, and payment methods.",
    "Customize the look and feel of the interface to match your preferences.",
    "Access advanced configuration options for power users and administrators.",
    "Find answers to common questions and contact our support team.",
    "View application version, licenses, and system information.",
    "Monitor your dashboard metrics and key performance indicators.",
    "Review detailed analytics and usage statistics for your account.",
    "Generate and download reports based on your activity and data.",
    "Set up integrations with third-party services and tools.",
    "Manage user roles, permissions, and access control settings.",
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

#[derive(Clone)]
struct TabInfo {
    label: String,
    content: String,
}

struct Level20State {
    tabs: Vec<TabInfo>,
    target_tab: usize,
    initial_tab: usize,
    mode: u8,
    style: u8,
    accent: String,
    x: f32,
    y: f32,
    card_w: f32,
    card_h: f32,
}

fn random_level20() -> Level20State {
    let mut rng = fresh_rng();
    let count = rng.random_range(3..=5usize);

    let mut label_pool: Vec<usize> = (0..TAB_LABELS.len()).collect();
    let mut content_pool: Vec<usize> = (0..TAB_CONTENTS.len()).collect();
    let mut tabs = Vec::new();

    for _ in 0..count {
        let li = rng.random_range(0..label_pool.len());
        let label = TAB_LABELS[label_pool.remove(li)].to_string();

        let ci = rng.random_range(0..content_pool.len());
        let content = TAB_CONTENTS[content_pool.remove(ci)].to_string();

        tabs.push(TabInfo { label, content });
    }

    let target_tab = rng.random_range(0..count);
    let mut initial_tab = rng.random_range(0..count);
    while initial_tab == target_tab {
        initial_tab = rng.random_range(0..count);
    }

    let mode = rng.random_range(0..2u8);
    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    let card_w = rng.random_range(350.0..=500.0f32);
    let card_h = rng.random_range(280.0..=400.0f32);
    let margin = 50.0;
    let (x, y) = super::safe_position(&mut rng, card_w, card_h, margin);

    Level20State { tabs, target_tab, initial_tab, mode, style, accent, x, y, card_w, card_h }
}

#[component]
pub fn Level20() -> Element {
    let mut state = use_signal(|| random_level20());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_tab = state.read().initial_tab;
    let mut active = use_signal(move || initial_tab);
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let tabs: Vec<TabInfo> = st.tabs.clone();
    let target_tab = st.target_tab;
    let mode = st.mode;
    let style = st.style;
    let accent = st.accent.clone();
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    let card_h = st.card_h;
    drop(st);

    let tab_count = tabs.len();
    let is_wrong = wrong();
    let cur_active = active();

    let target_label = tabs[target_tab].label.clone();

    let instruction = match mode {
        1 => {
            let ord = ordinal(target_tab + 1);
            format!("Switch to the {} tab", ord)
        }
        _ => {
            format!("Switch to the \"{}\" tab", target_label)
        }
    };

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; height: {}px; box-sizing: border-box; display: flex; flex-direction: column; overflow: hidden;",
        card_x, card_y, card_w, card_h
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth — build UINode tree
    let card_rect = Rect::new(card_x, card_y, card_w, card_h);
    let children: Vec<UINode> = tabs.iter().enumerate().map(|(i, t)| {
        let tab_rect = Rect::new(card_x, card_y, card_w, card_h);
        if i == target_tab {
            ui_node::tab(&t.label, tab_rect)
        } else {
            // Non-target tab
            UINode::Tab(Visual::new(&t.label, tab_rect))
        }
    }).collect();
    let tree = ui_node::form(card_rect, "Submit", children);
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), false);

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
                    "Level 9"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Tabs"
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
                        style: "margin: 0; padding: 12px 16px; font-size: 14px; color: #374151; font-weight: 500; flex-shrink: 0;",
                        "{instruction}"
                    }

                    // Tab bar
                    {
                        let bar_style = match style {
                            0 => "display: flex; border-bottom: 1px solid #e5e7eb; padding: 0 16px; flex-shrink: 0;".to_string(),
                            1 => "display: flex; gap: 6px; padding: 8px 16px; flex-shrink: 0;".to_string(),
                            _ => "display: flex; padding: 0 16px; padding-top: 8px; flex-shrink: 0;".to_string(),
                        };
                        rsx! {
                            div {
                                style: "{bar_style}",

                                for ti in 0..tab_count {
                                    {
                                        let t = tabs[ti].clone();
                                        let is_active = ti == cur_active;
                                        let accent_c = accent.clone();

                                        let tab_style = match style {
                                            // Underline
                                            0 => {
                                                let border = if is_active {
                                                    format!("border-bottom: 2px solid {};", accent_c)
                                                } else {
                                                    "border-bottom: 2px solid transparent;".to_string()
                                                };
                                                let color = if is_active { accent_c.clone() } else { "#6b7280".to_string() };
                                                let weight = if is_active { "600" } else { "400" };
                                                format!("padding: 10px 16px; background: none; border: none; {} font-size: 13px; color: {}; font-weight: {}; cursor: pointer; font-family: system-ui, sans-serif; white-space: nowrap;", border, color, weight)
                                            }
                                            // Pill
                                            1 => {
                                                let (bg, color) = if is_active {
                                                    (accent_c.clone(), "white".to_string())
                                                } else {
                                                    ("#f3f4f6".to_string(), "#374151".to_string())
                                                };
                                                format!("padding: 6px 14px; background: {}; color: {}; border: none; border-radius: 20px; font-size: 13px; font-weight: 500; cursor: pointer; font-family: system-ui, sans-serif; white-space: nowrap;", bg, color)
                                            }
                                            // Boxed
                                            _ => {
                                                let (bg, color, bb) = if is_active {
                                                    ("white".to_string(), "#111827".to_string(), "border-bottom: 1px solid white; margin-bottom: -1px;".to_string())
                                                } else {
                                                    ("#f3f4f6".to_string(), "#6b7280".to_string(), "border-bottom: 1px solid #e5e7eb;".to_string())
                                                };
                                                format!("padding: 8px 14px; background: {}; color: {}; border: 1px solid #e5e7eb; border-bottom: none; {} border-radius: 6px 6px 0 0; font-size: 13px; font-weight: 500; cursor: pointer; font-family: system-ui, sans-serif; white-space: nowrap;", bg, color, bb)
                                            }
                                        };

                                        rsx! {
                                            button {
                                                class: if ti == target_tab { "target" } else { "" },
                                                "data-label": "{t.label}",
                                                style: "{tab_style}",
                                                tabindex: "-1",
                                                onclick: move |_| {
                                                    active.set(ti);
                                                },
                                                "{t.label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Boxed style needs a top border on the panel
                    if style == 2 {
                        div { style: "border-top: 1px solid #e5e7eb; margin: 0 16px;" }
                    }

                    // Panel content
                    div {
                        style: "flex: 1; padding: 16px; overflow-y: auto; min-height: 0;",

                        p {
                            style: "color: #374151; font-size: 14px; line-height: 1.6; margin: 0;",
                            "{tabs[cur_active].content}"
                        }
                    }

                    // Submit
                    div {
                        style: "padding: 12px 16px; flex-shrink: 0;",
                        button {
                            style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s;",
                            tabindex: "-1",
                            onclick: move |_| {
                                if cur_active == target_tab {
                                    score.set(score() + 1);
                                    bg.set(random_canvas_bg());
                                    let new_st = random_level20();
                                    let new_active = new_st.initial_tab;
                                    state.set(new_st);
                                    active.set(new_active);
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
            }

            super::GroundTruth {
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: card_w,
                target_h: card_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
