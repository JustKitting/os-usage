use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg};

/// Context-menu scenarios: a trigger element + menu items.
struct MenuScenario {
    trigger_label: &'static str,
    items: &'static [&'static str],
}

const SCENARIOS: &[MenuScenario] = &[
    MenuScenario { trigger_label: "document.pdf", items: &["Open", "Rename", "Copy", "Move to Trash"] },
    MenuScenario { trigger_label: "photo.jpg", items: &["View", "Edit", "Share", "Delete"] },
    MenuScenario { trigger_label: "Inbox (24)", items: &["Mark All Read", "Archive", "Move to Spam", "Delete All"] },
    MenuScenario { trigger_label: "main.rs", items: &["Open in Editor", "Copy Path", "Rename", "Delete"] },
    MenuScenario { trigger_label: "Profile Picture", items: &["Change Photo", "Remove Photo", "View Full Size"] },
    MenuScenario { trigger_label: "Shopping Cart", items: &["View Cart", "Clear Cart", "Save for Later", "Checkout"] },
    MenuScenario { trigger_label: "Notification Bell", items: &["Mark All Read", "Mute", "Settings"] },
    MenuScenario { trigger_label: "playlist.m3u", items: &["Play", "Shuffle", "Add to Queue", "Delete"] },
    MenuScenario { trigger_label: "meeting_notes.docx", items: &["Open", "Download", "Share Link", "Move", "Delete"] },
    MenuScenario { trigger_label: "User Avatar", items: &["View Profile", "Send Message", "Block", "Report"] },
    MenuScenario { trigger_label: "server-01", items: &["Connect", "Restart", "View Logs", "Terminate"] },
    MenuScenario { trigger_label: "backup_2024.zip", items: &["Extract", "Download", "Rename", "Delete"] },
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

const TRIGGER_ICONS: &[&str] = &[
    "\u{1F4C4}", "\u{1F4F7}", "\u{1F4E8}", "\u{1F4DD}", "\u{1F464}",
    "\u{1F6D2}", "\u{1F514}", "\u{1F3B5}", "\u{1F4C3}", "\u{1F468}",
    "\u{1F5A5}", "\u{1F4E6}",
];

struct Level23State {
    scenario_idx: usize,
    target_item: usize,
    style: u8,
    accent: String,
    trigger_x: f32,
    trigger_y: f32,
    menu_offset_x: f32,
    menu_offset_y: f32,
    has_separator: bool,
    has_icons: bool,
}

fn random_level23() -> Level23State {
    let mut rng = fresh_rng();
    let scenario_idx = rng.random_range(0..SCENARIOS.len());
    let scenario = &SCENARIOS[scenario_idx];
    let target_item = rng.random_range(0..scenario.items.len());
    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    let trigger_w = 200.0f32;
    let trigger_h = 48.0f32;
    let menu_w = 200.0f32;
    let item_h = 36.0f32;
    let menu_h = scenario.items.len() as f32 * item_h + 16.0;

    // Position trigger so menu fits in viewport
    let margin = 60.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (trigger_x, trigger_y) = super::safe_position_in(&mut rng, trigger_w + menu_w, trigger_h + menu_h, margin, vp_w * 1.3, vp_h * 1.3);

    // Menu appears near the trigger (like a real right-click menu)
    let menu_offset_x = rng.random_range(10.0..40.0f32);
    let menu_offset_y = rng.random_range(-10.0..20.0f32);

    let has_separator = rng.random_bool(0.4);
    let has_icons = rng.random_bool(0.5);

    Level23State {
        scenario_idx, target_item, style, accent,
        trigger_x, trigger_y, menu_offset_x, menu_offset_y,
        has_separator, has_icons,
    }
}

// Simple menu-item icons (single characters)
const ITEM_ICONS: &[&str] = &[
    "\u{2702}", "\u{270F}", "\u{2709}", "\u{2605}", "\u{2764}",
    "\u{21BB}", "\u{2716}", "\u{2714}", "\u{2B06}", "\u{2B07}",
    "\u{1F50D}", "\u{1F517}", "\u{2699}", "\u{1F512}", "\u{1F5D1}",
];

#[component]
pub fn Level23() -> Element {
    let mut state = use_signal(|| random_level23());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);
    let mut menu_open = use_signal(|| true);

    let st = state.read();
    let scenario = &SCENARIOS[st.scenario_idx];
    let trigger_label = scenario.trigger_label;
    let items: Vec<&str> = scenario.items.to_vec();
    let target_item = st.target_item;
    let style = st.style;
    let trigger_x = st.trigger_x;
    let trigger_y = st.trigger_y;
    let menu_offset_x = st.menu_offset_x;
    let menu_offset_y = st.menu_offset_y;
    let has_separator = st.has_separator;
    let has_icons = st.has_icons;
    let scenario_idx = st.scenario_idx;
    drop(st);

    let item_count = items.len();
    let is_wrong = wrong();
    let is_open = menu_open();

    let target_label = items[target_item];
    let instruction = format!("Right-click \"{}\", then click \"{}\"", trigger_label, target_label);

    // Trigger element styling
    let trigger_icon = TRIGGER_ICONS[scenario_idx];
    let trigger_w = 200.0f32;
    let trigger_h = 48.0f32;
    let trigger_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; \
         background: white; border-radius: 8px; padding: 0 16px; \
         display: flex; align-items: center; gap: 10px; \
         box-shadow: 0 2px 12px rgba(0,0,0,0.15); \
         font-family: system-ui, sans-serif; font-size: 14px; color: #374151; \
         cursor: context-menu; user-select: none; box-sizing: border-box;",
        trigger_x, trigger_y, trigger_w, trigger_h
    );

    // Menu position
    let menu_x = trigger_x + menu_offset_x;
    let menu_y = trigger_y + trigger_h + menu_offset_y;
    let menu_w = 200.0f32;

    // Menu styling based on style variant
    let (menu_radius, menu_shadow, menu_border, item_radius) = match style {
        0 => ("12px", "0 8px 30px rgba(0,0,0,0.2)", "1px solid #e5e7eb", "8px"),
        1 => ("4px", "0 2px 12px rgba(0,0,0,0.15)", "1px solid #d1d5db", "2px"),
        _ => ("8px", "0 4px 20px rgba(0,0,0,0.18)", "none", "6px"),
    };

    let menu_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; \
         background: white; border-radius: {}; box-shadow: {}; border: {}; \
         padding: 6px; font-family: system-ui, sans-serif; z-index: 20; box-sizing: border-box;",
        menu_x, menu_y, menu_w, menu_radius, menu_shadow, menu_border
    );

    // Separator index: place one separator roughly in the middle
    let sep_after = if has_separator { item_count / 2 } else { usize::MAX };

    // Ground truth via UINode tree
    let item_h_est = 36.0f32;
    let menu_h_est = item_count as f32 * item_h_est + 16.0;
    let tree = ui_node::context_menu(
        Rect::new(trigger_x, trigger_y, trigger_w, trigger_h),
        trigger_label,
        items.iter().map(|s| s.to_string()).collect(),
        target_label,
    );
    let description = String::new();
    let viewport_style = super::viewport_style(&bg(), true);

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
                    "Level 23"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Context Menu"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "{viewport_style}",

                // Instruction
                div {
                    style: "position: absolute; left: 0; right: 0; top: 16px; text-align: center; z-index: 30;",
                    div {
                        style: "display: inline-block; background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 8px; color: white; font-size: 14px; font-weight: 500;",
                        "{instruction}"
                    }
                }

                // Trigger element
                div {
                    style: "{trigger_style}",
                    "data-label": "trigger",
                    oncontextmenu: move |evt| {
                        evt.prevent_default();
                        menu_open.set(true);
                    },
                    span { style: "font-size: 18px;", "{trigger_icon}" }
                    span { "{trigger_label}" }
                }

                // Context menu (shown when open)
                if is_open {
                    div {
                        style: "{menu_style}",

                        for mi in 0..item_count {
                            {
                                let label = items[mi];
                                let hover_bg = if is_wrong && mi == target_item {
                                    "#fecaca".to_string()
                                } else {
                                    "transparent".to_string()
                                };

                                let icon_char = if has_icons {
                                    ITEM_ICONS[mi % ITEM_ICONS.len()]
                                } else {
                                    ""
                                };

                                let item_style = format!(
                                    "display: flex; align-items: center; gap: 10px; \
                                     width: 100%; padding: 8px 12px; background: {}; \
                                     border: none; border-radius: {}; font-size: 13px; \
                                     color: #374151; cursor: pointer; text-align: left; \
                                     font-family: system-ui, sans-serif; box-sizing: border-box; \
                                     transition: background 0.1s;",
                                    hover_bg, item_radius
                                );

                                rsx! {
                                    button {
                                        class: if mi == target_item { "target" } else { "" },
                                        "data-label": "{label}",
                                        style: "{item_style}",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            if mi == target_item {
                                                score.set(score() + 1);
                                                bg.set(random_canvas_bg());
                                                state.set(random_level23());
                                                wrong.set(false);
                                                menu_open.set(true);
                                            } else {
                                                wrong.set(true);
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                                    wrong.set(false);
                                                });
                                            }
                                        },
                                        if has_icons {
                                            span {
                                                style: "font-size: 14px; width: 18px; text-align: center; flex-shrink: 0;",
                                                "{icon_char}"
                                            }
                                        }
                                        span { "{label}" }
                                    }

                                    // Separator line
                                    if mi == sep_after {
                                        div {
                                            style: "height: 1px; background: #e5e7eb; margin: 4px 8px;",
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
                target_x: menu_x,
                target_y: menu_y,
                target_w: menu_w,
                target_h: menu_h_est,
                tree: Some(tree.clone()),
            }
        }
    }
}
