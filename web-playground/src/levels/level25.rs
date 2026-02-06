use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, describe_position, ordinal};

struct ListScenario {
    title: &'static str,
    items: &'static [&'static str],
}

const SCENARIOS: &[ListScenario] = &[
    ListScenario { title: "Priority Tasks", items: &[
        "Fix login bug", "Deploy to staging", "Write unit tests", "Update docs",
        "Review PR #42", "Refactor auth", "Add logging", "Setup CI",
    ]},
    ListScenario { title: "Playlist", items: &[
        "Bohemian Rhapsody", "Hotel California", "Stairway to Heaven",
        "Imagine", "Yesterday", "Hey Jude", "Wonderwall", "Creep",
    ]},
    ListScenario { title: "Shopping List", items: &[
        "Milk", "Eggs", "Bread", "Butter", "Cheese",
        "Apples", "Rice", "Chicken",
    ]},
    ListScenario { title: "Travel Itinerary", items: &[
        "Book flights", "Reserve hotel", "Rent car", "Pack bags",
        "Get passport", "Buy insurance", "Plan route", "Exchange currency",
    ]},
    ListScenario { title: "Recipe Steps", items: &[
        "Preheat oven", "Mix dry ingredients", "Beat eggs", "Combine wet and dry",
        "Pour into pan", "Bake 25 min", "Cool on rack", "Add frosting",
    ]},
    ListScenario { title: "Sprint Backlog", items: &[
        "FEAT-101", "BUG-203", "FEAT-105", "CHORE-44",
        "BUG-210", "FEAT-112", "CHORE-51", "BUG-215",
    ]},
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

struct Level25State {
    scenario_idx: usize,
    /// Indices into scenario.items, in display order
    order: Vec<usize>,
    /// Which item (by scenario index) to move
    target_item: usize,
    /// Target position (0-indexed) where it should end up
    target_pos: usize,
    style: u8,
    accent: String,
    card_x: f32,
    card_y: f32,
    card_w: f32,
}

fn random_level25() -> Level25State {
    let mut rng = fresh_rng();
    let scenario_idx = rng.random_range(0..SCENARIOS.len());
    let scenario = &SCENARIOS[scenario_idx];

    // Pick 5-7 items
    let count = rng.random_range(5..=7usize).min(scenario.items.len());
    let mut pool: Vec<usize> = (0..scenario.items.len()).collect();
    let mut order = Vec::new();
    for _ in 0..count {
        let idx = rng.random_range(0..pool.len());
        order.push(pool.remove(idx));
    }

    // Pick target item and a different target position
    let item_idx_in_order = rng.random_range(0..count);
    let target_item = order[item_idx_in_order];

    // Pick a target position that's different from current
    let mut target_pos = rng.random_range(0..count);
    while target_pos == item_idx_in_order {
        target_pos = rng.random_range(0..count);
    }

    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    let card_w = rng.random_range(280.0..=380.0f32);
    let item_h = 44.0f32;
    let card_h = count as f32 * item_h + 120.0;
    let margin = 60.0;
    let card_x = rng.random_range(margin..(Position::VIEWPORT - card_w - margin).max(margin + 1.0));
    let card_y = rng.random_range(margin..(Position::VIEWPORT - card_h - margin).max(margin + 1.0));

    Level25State { scenario_idx, order, target_item, target_pos, style, accent, card_x, card_y, card_w }
}

#[component]
pub fn Level25() -> Element {
    let mut state = use_signal(|| random_level25());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);
    let initial_order = state.read().order.clone();
    let mut order = use_signal(move || initial_order);
    let mut selected = use_signal(|| None::<usize>);

    let st = state.read();
    let scenario = &SCENARIOS[st.scenario_idx];
    let title = scenario.title;
    let target_item = st.target_item;
    let target_pos = st.target_pos;
    let style = st.style;
    let accent = st.accent.clone();
    let card_x = st.card_x;
    let card_y = st.card_y;
    let card_w = st.card_w;
    drop(st);

    let cur_order: Vec<usize> = order.read().clone();
    let item_count = cur_order.len();
    let is_wrong = wrong();
    let sel = selected();

    let target_label = scenario.items[target_item];
    let target_ord = ordinal(target_pos + 1);
    let instruction = format!("Move \"{}\" to {} position", target_label, target_ord);

    // Check if target is already in correct position
    let is_correct = cur_order.get(target_pos) == Some(&target_item);

    let border_radius = match style { 0 => "16px", 1 => "6px", _ => "10px" };
    let item_radius = match style { 0 => "10px", 1 => "4px", _ => "6px" };
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; \
         background: white; border-radius: {}; \
         box-shadow: 0 4px 24px rgba(0,0,0,0.3); \
         font-family: system-ui, sans-serif; box-sizing: border-box; padding: 16px;",
        card_x, card_y, card_w, border_radius
    );

    let submit_bg = if is_wrong { "#ef4444" } else { &accent };

    // Ground truth
    let items_desc: String = cur_order.iter().enumerate().map(|(i, &si)| {
        let label = scenario.items[si];
        let target_mark = if si == target_item { " (TARGET)" } else { "" };
        let pos_mark = if i == target_pos { " [TARGET POS]" } else { "" };
        format!("{}. \"{}\"{}{}",  i + 1, label, target_mark, pos_mark)
    }).collect::<Vec<_>>().join(", ");
    let item_h_est = 44.0f32;
    let card_h_est = item_count as f32 * item_h_est + 120.0;
    let position_desc = describe_position(card_x, card_y, card_w, card_h_est);
    let description = format!(
        "sortable list, title: \"{}\", items: [{}], goal: move \"{}\" to pos {}, style: {}, at {}",
        title, items_desc, target_label, target_pos + 1,
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
                    "Level 25"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Sortable List"
                }
                span {
                    style: "color: #22c55e; font-size: 14px; font-family: monospace;",
                    "score: {score}"
                }
            }

            div {
                id: "viewport",
                style: "width: 1024px; height: 1024px; background: {bg}; position: relative; border: 1px solid #2a2a4a; overflow: hidden; transition: background 0.4s;",

                // Instruction
                div {
                    style: "position: absolute; left: 0; right: 0; top: 16px; text-align: center; z-index: 30;",
                    div {
                        style: "display: inline-block; background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 8px; color: white; font-size: 14px; font-weight: 500;",
                        "{instruction}"
                    }
                }

                div {
                    style: "{card_style}",

                    // Title
                    h3 {
                        style: "margin: 0 0 12px 0; font-size: 16px; color: #111827; font-weight: 600;",
                        "{title}"
                    }

                    // Hint
                    p {
                        style: "margin: 0 0 12px 0; font-size: 12px; color: #9ca3af;",
                        "Click an item to select, then click another to swap"
                    }

                    // List items
                    div {
                        style: "display: flex; flex-direction: column; gap: 4px;",

                        for di in 0..item_count {
                            {
                                let si = cur_order[di];
                                let label = scenario.items[si];
                                let is_selected = sel == Some(di);
                                let is_target_item = si == target_item;
                                let is_target_pos = di == target_pos;
                                let accent_c = accent.clone();

                                let item_bg = if is_selected {
                                    format!("{}22", accent_c)
                                } else {
                                    "#f9fafb".to_string()
                                };
                                let item_border = if is_selected {
                                    format!("2px solid {}", accent_c)
                                } else if is_target_pos {
                                    "2px dashed #d1d5db".to_string()
                                } else {
                                    "2px solid transparent".to_string()
                                };
                                let font_weight = if is_target_item { "600" } else { "400" };

                                let item_style = format!(
                                    "display: flex; align-items: center; gap: 10px; \
                                     width: 100%; padding: 10px 12px; background: {}; \
                                     border: {}; border-radius: {}; font-size: 14px; \
                                     color: #374151; cursor: grab; text-align: left; \
                                     font-family: system-ui, sans-serif; box-sizing: border-box; \
                                     transition: background 0.1s, border 0.1s; font-weight: {};",
                                    item_bg, item_border, item_radius, font_weight
                                );

                                rsx! {
                                    button {
                                        class: if is_target_item { "target" } else { "" },
                                        "data-label": "{label}",
                                        style: "{item_style}",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            match sel {
                                                Some(prev) if prev != di => {
                                                    // Swap the two items
                                                    let mut o = order.write();
                                                    o.swap(prev, di);
                                                    selected.set(None);
                                                }
                                                Some(prev) if prev == di => {
                                                    // Deselect
                                                    selected.set(None);
                                                }
                                                _ => {
                                                    // Select this item
                                                    selected.set(Some(di));
                                                }
                                            }
                                        },
                                        // Grip handle
                                        span {
                                            style: "color: #d1d5db; font-size: 14px; flex-shrink: 0;",
                                            "\u{2261}"
                                        }
                                        // Position number
                                        span {
                                            style: "color: #9ca3af; font-size: 12px; width: 18px; flex-shrink: 0; font-family: monospace;",
                                            "{di + 1}."
                                        }
                                        span { "{label}" }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        "data-label": "Submit",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: {item_radius}; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 12px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            if is_correct {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level25();
                                let new_order = new_st.order.clone();
                                state.set(new_st);
                                order.set(new_order);
                                selected.set(None);
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
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: card_w,
                target_h: card_h_est,
                steps: format!(r#"[{{"action":"click","target":"{}"}},{{"action":"click","target":"Submit"}}]"#, target_label),
            }
        }
    }
}
