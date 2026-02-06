use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, Rect};
use super::{fresh_rng, random_canvas_bg};

/// Each scenario has a search placeholder and a pool of suggestions.
struct SearchScenario {
    placeholder: &'static str,
    suggestions: &'static [&'static str],
}

const SCENARIOS: &[SearchScenario] = &[
    SearchScenario { placeholder: "Search cities...", suggestions: &[
        "New York", "Los Angeles", "Chicago", "Houston", "Phoenix",
        "San Antonio", "San Diego", "Dallas", "Austin", "Jacksonville",
    ]},
    SearchScenario { placeholder: "Search products...", suggestions: &[
        "Wireless Headphones", "Laptop Stand", "USB-C Hub", "Mechanical Keyboard",
        "Monitor Arm", "Webcam HD", "Desk Lamp", "Mouse Pad XL", "Cable Organizer", "Power Strip",
    ]},
    SearchScenario { placeholder: "Search languages...", suggestions: &[
        "Rust", "Python", "TypeScript", "Go", "Java",
        "C++", "Swift", "Kotlin", "Ruby", "Haskell",
    ]},
    SearchScenario { placeholder: "Search contacts...", suggestions: &[
        "Alice Johnson", "Bob Smith", "Carol White", "David Brown", "Eve Davis",
        "Frank Miller", "Grace Lee", "Henry Wilson", "Iris Chen", "Jack Taylor",
    ]},
    SearchScenario { placeholder: "Search countries...", suggestions: &[
        "United States", "United Kingdom", "Canada", "Australia", "Germany",
        "France", "Japan", "Brazil", "India", "South Korea",
    ]},
    SearchScenario { placeholder: "Search recipes...", suggestions: &[
        "Pasta Carbonara", "Chicken Tikka", "Caesar Salad", "Beef Tacos",
        "Pad Thai", "Mushroom Risotto", "Fish and Chips", "Veggie Burger", "Tom Yum Soup", "Sushi Roll",
    ]},
    SearchScenario { placeholder: "Search files...", suggestions: &[
        "README.md", "package.json", "index.html", "styles.css",
        "app.tsx", "config.yaml", "Dockerfile", "Makefile", "main.rs", ".gitignore",
    ]},
    SearchScenario { placeholder: "Search songs...", suggestions: &[
        "Bohemian Rhapsody", "Hotel California", "Stairway to Heaven", "Imagine",
        "Yesterday", "Smells Like Teen Spirit", "Billie Jean", "Hey Jude", "Wonderwall", "Losing My Religion",
    ]},
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

struct Level24State {
    scenario_idx: usize,
    visible_items: Vec<usize>,
    target_item: usize,
    style: u8,
    accent: String,
    card_x: f32,
    card_y: f32,
    card_w: f32,
    prefill: String,
}

fn random_level24() -> Level24State {
    let mut rng = fresh_rng();
    let scenario_idx = rng.random_range(0..SCENARIOS.len());
    let scenario = &SCENARIOS[scenario_idx];
    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    // Pick 4-7 suggestions to show in dropdown
    let count = rng.random_range(4..=7usize).min(scenario.suggestions.len());
    let mut pool: Vec<usize> = (0..scenario.suggestions.len()).collect();
    let mut visible_items = Vec::new();
    for _ in 0..count {
        let idx = rng.random_range(0..pool.len());
        visible_items.push(pool.remove(idx));
    }

    let target_item = rng.random_range(0..visible_items.len());

    // Prefill: first 1-3 characters of the target (to simulate typing)
    let target_text = scenario.suggestions[visible_items[target_item]];
    let prefill_len = rng.random_range(1..=3usize).min(target_text.len());
    let prefill = target_text[..prefill_len].to_lowercase();

    let card_w = rng.random_range(280.0..=400.0f32);
    let item_h = 40.0f32;
    let card_h = 52.0 + count as f32 * item_h + 16.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (card_x, card_y) = super::safe_position_in(&mut rng, card_w, card_h, 60.0, vp_w * 1.3, vp_h * 1.3);

    Level24State { scenario_idx, visible_items, target_item, style, accent, card_x, card_y, card_w, prefill }
}

#[component]
pub fn Level24() -> Element {
    let mut state = use_signal(|| random_level24());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let scenario = &SCENARIOS[st.scenario_idx];
    let placeholder = scenario.placeholder;
    let visible_items: Vec<usize> = st.visible_items.clone();
    let target_item = st.target_item;
    let style = st.style;
    let accent = st.accent.clone();
    let card_x = st.card_x;
    let card_y = st.card_y;
    let card_w = st.card_w;
    let prefill = st.prefill.clone();
    drop(st);

    let item_count = visible_items.len();
    let is_wrong = wrong();

    let scenario2 = &SCENARIOS[state.read().scenario_idx];
    let target_text = scenario2.suggestions[visible_items[target_item]];
    let instruction = format!("Select \"{}\"", target_text);

    // Card styling
    let border_radius = match style { 0 => "16px", 1 => "6px", _ => "10px" };
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; \
         background: white; border-radius: {}; \
         box-shadow: 0 4px 24px rgba(0,0,0,0.3); \
         font-family: system-ui, sans-serif; box-sizing: border-box; padding: 12px;",
        card_x, card_y, card_w, border_radius
    );

    // Input styling
    let input_radius = match style { 0 => "12px", 1 => "4px", _ => "8px" };
    let input_style = format!(
        "width: 100%; padding: 10px 14px; border: 2px solid {}; border-radius: {}; \
         font-size: 14px; color: #111827; outline: none; box-sizing: border-box; \
         font-family: system-ui, sans-serif; background: #fafafa;",
        accent, input_radius
    );

    // Dropdown styling
    let dropdown_radius = match style { 0 => "12px", 1 => "4px", _ => "8px" };
    let dropdown_style = format!(
        "margin-top: 4px; border: 1px solid #e5e7eb; border-radius: {}; \
         overflow: hidden; background: white;",
        dropdown_radius
    );

    let item_radius = match style { 0 => "8px", 1 => "2px", _ => "6px" };

    // Ground truth via UINode tree
    let item_h_est = 40.0f32;
    let card_h_est = 52.0 + item_count as f32 * item_h_est + 16.0;
    let suggestion_y_start = card_y + 56.0; // after input area
    let tree = ui_node::card(
        Rect::new(card_x, card_y, card_w, card_h_est),
        vec![
            ui_node::target_button(
                target_text,
                Rect::new(card_x, suggestion_y_start + target_item as f32 * item_h_est, card_w, item_h_est),
            ),
        ],
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
                    "Level 24"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Autocomplete"
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

                // Search card
                div {
                    style: "{card_style}",

                    // Search input (read-only, shows prefilled text)
                    div {
                        style: "position: relative;",
                        div {
                            style: "{input_style}",
                            span { style: "color: #111827;", "{prefill}" }
                            span { style: "color: #9ca3af;", "{placeholder}" }
                        }
                        // Search icon
                        div {
                            style: "position: absolute; right: 12px; top: 50%; transform: translateY(-50%); color: #9ca3af; font-size: 16px;",
                            "\u{1F50D}"
                        }
                    }

                    // Dropdown suggestions
                    div {
                        style: "{dropdown_style}",

                        for di in 0..item_count {
                            {
                                let si = visible_items[di];
                                let label = scenario2.suggestions[si];
                                let accent_c = accent.clone();

                                let item_bg = if is_wrong && di == target_item {
                                    "#fecaca".to_string()
                                } else {
                                    "transparent".to_string()
                                };

                                // Highlight the matching prefix
                                let prefill_c = prefill.clone();
                                let label_lower = label.to_lowercase();
                                let match_len = if label_lower.starts_with(&prefill_c) { prefill_c.len() } else { 0 };
                                let matched = &label[..match_len];
                                let rest = &label[match_len..];

                                let item_style = format!(
                                    "display: flex; align-items: center; width: 100%; padding: 10px 14px; \
                                     background: {}; border: none; border-radius: {}; font-size: 14px; \
                                     color: #374151; cursor: pointer; text-align: left; \
                                     font-family: system-ui, sans-serif; box-sizing: border-box; \
                                     transition: background 0.1s;",
                                    item_bg, item_radius
                                );

                                rsx! {
                                    button {
                                        class: if di == target_item { "target" } else { "" },
                                        "data-label": "{label}",
                                        style: "{item_style}",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            if di == target_item {
                                                score.set(score() + 1);
                                                bg.set(random_canvas_bg());
                                                state.set(random_level24());
                                                wrong.set(false);
                                            } else {
                                                wrong.set(true);
                                                spawn(async move {
                                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                                    wrong.set(false);
                                                });
                                            }
                                        },
                                        if match_len > 0 {
                                            span {
                                                style: "font-weight: 700; color: {accent_c};",
                                                "{matched}"
                                            }
                                            span { "{rest}" }
                                        } else {
                                            span { "{label}" }
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
                target_w: card_w,
                target_h: card_h_est,
                tree: Some(tree.clone()),
            }
        }
    }
}
