use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, describe_position};

struct TagScenario {
    title: &'static str,
    tags: &'static [&'static str],
}

const SCENARIOS: &[TagScenario] = &[
    TagScenario { title: "Skills", tags: &[
        "Rust", "Python", "TypeScript", "Go", "Java", "C++", "Swift", "Kotlin", "Ruby", "SQL",
    ]},
    TagScenario { title: "Interests", tags: &[
        "Photography", "Hiking", "Cooking", "Gaming", "Reading", "Travel", "Music", "Fitness", "Art", "Film",
    ]},
    TagScenario { title: "Categories", tags: &[
        "Electronics", "Clothing", "Books", "Home", "Sports", "Toys", "Garden", "Automotive", "Health", "Food",
    ]},
    TagScenario { title: "Genres", tags: &[
        "Rock", "Jazz", "Pop", "Classical", "Hip-Hop", "Electronic", "Country", "R&B", "Metal", "Folk",
    ]},
    TagScenario { title: "Allergies", tags: &[
        "Peanuts", "Gluten", "Dairy", "Shellfish", "Eggs", "Soy", "Tree Nuts", "Wheat", "Fish", "Sesame",
    ]},
    TagScenario { title: "Features", tags: &[
        "Dark Mode", "Notifications", "Auto-Save", "Sync", "Offline", "2FA", "Analytics", "Export", "API", "Webhooks",
    ]},
    TagScenario { title: "Toppings", tags: &[
        "Pepperoni", "Mushrooms", "Onions", "Olives", "Peppers", "Sausage", "Bacon", "Jalapeños", "Tomatoes", "Spinach",
    ]},
    TagScenario { title: "Languages", tags: &[
        "English", "Spanish", "French", "German", "Japanese", "Mandarin", "Korean", "Portuguese", "Italian", "Arabic",
    ]},
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

/// Mode: add means select unselected tags, remove means deselect already-selected tags
#[derive(Clone, Copy, PartialEq)]
enum TagMode {
    Add,
    Remove,
}

struct Level26State {
    scenario_idx: usize,
    available: Vec<usize>,
    initially_selected: Vec<bool>,
    target_tags: Vec<usize>,
    mode: TagMode,
    style: u8,
    accent: String,
    card_x: f32,
    card_y: f32,
    card_w: f32,
}

fn random_level26() -> Level26State {
    let mut rng = fresh_rng();
    let scenario_idx = rng.random_range(0..SCENARIOS.len());
    let scenario = &SCENARIOS[scenario_idx];

    // Pick 6-9 tags to show
    let count = rng.random_range(6..=9usize).min(scenario.tags.len());
    let mut pool: Vec<usize> = (0..scenario.tags.len()).collect();
    let mut available = Vec::new();
    for _ in 0..count {
        let idx = rng.random_range(0..pool.len());
        available.push(pool.remove(idx));
    }

    let mode = if rng.random_bool(0.5) { TagMode::Add } else { TagMode::Remove };

    // For Add mode: some tags start selected, target is among unselected
    // For Remove mode: most tags start selected, target is among selected
    let mut initially_selected = vec![false; count];
    let mut target_tags = Vec::new();

    match mode {
        TagMode::Add => {
            // 2-4 start selected
            let pre_selected = rng.random_range(2..=4usize).min(count - 2);
            let mut indices: Vec<usize> = (0..count).collect();
            for _ in 0..pre_selected {
                let idx = rng.random_range(0..indices.len());
                let i = indices.remove(idx);
                initially_selected[i] = true;
            }
            // Pick 1-3 targets from unselected
            let unselected: Vec<usize> = (0..count).filter(|i| !initially_selected[*i]).collect();
            let target_count = rng.random_range(1..=3usize).min(unselected.len());
            let mut unsel_pool = unselected;
            for _ in 0..target_count {
                let idx = rng.random_range(0..unsel_pool.len());
                target_tags.push(unsel_pool.remove(idx));
            }
        }
        TagMode::Remove => {
            // Most start selected
            for i in 0..count {
                initially_selected[i] = rng.random_bool(0.75);
            }
            // Ensure at least 3 selected
            let sel_count = initially_selected.iter().filter(|&&s| s).count();
            if sel_count < 3 {
                for i in 0..count {
                    if !initially_selected[i] {
                        initially_selected[i] = true;
                        if initially_selected.iter().filter(|&&s| s).count() >= 4 { break; }
                    }
                }
            }
            // Pick 1-2 targets from selected
            let selected_indices: Vec<usize> = (0..count).filter(|i| initially_selected[*i]).collect();
            let target_count = rng.random_range(1..=2usize).min(selected_indices.len());
            let mut sel_pool = selected_indices;
            for _ in 0..target_count {
                let idx = rng.random_range(0..sel_pool.len());
                target_tags.push(sel_pool.remove(idx));
            }
        }
    }

    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    let card_w = rng.random_range(320.0..=460.0f32);
    let card_h = 280.0;
    let margin = 60.0;
    let card_x = rng.random_range(margin..(Position::VIEWPORT - card_w - margin).max(margin + 1.0));
    let card_y = rng.random_range(margin..(Position::VIEWPORT - card_h - margin).max(margin + 1.0));

    Level26State { scenario_idx, available, initially_selected, target_tags, mode, style, accent, card_x, card_y, card_w }
}

#[component]
pub fn Level26() -> Element {
    let mut state = use_signal(|| random_level26());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let mut wrong = use_signal(|| false);
    let initial_sel = state.read().initially_selected.clone();
    let mut selected = use_signal(move || initial_sel);

    let st = state.read();
    let scenario = &SCENARIOS[st.scenario_idx];
    let title = scenario.title;
    let available: Vec<usize> = st.available.clone();
    let target_tags: Vec<usize> = st.target_tags.clone();
    let mode = st.mode;
    let style = st.style;
    let accent = st.accent.clone();
    let card_x = st.card_x;
    let card_y = st.card_y;
    let card_w = st.card_w;
    drop(st);

    let tag_count = available.len();
    let is_wrong = wrong();
    let cur_sel: Vec<bool> = selected.read().clone();

    // Build instruction
    let target_labels: Vec<&str> = target_tags.iter().map(|&ti| scenario.tags[available[ti]]).collect();
    let instruction = match mode {
        TagMode::Add => {
            let labels = target_labels.iter().map(|l| format!("\"{}\"", l)).collect::<Vec<_>>().join(", ");
            format!("Select {}", labels)
        }
        TagMode::Remove => {
            let labels = target_labels.iter().map(|l| format!("\"{}\"", l)).collect::<Vec<_>>().join(", ");
            format!("Remove {}", labels)
        }
    };

    // Check if goal is met
    let is_correct = target_tags.iter().all(|&ti| {
        match mode {
            TagMode::Add => cur_sel.get(ti).copied().unwrap_or(false),
            TagMode::Remove => !cur_sel.get(ti).copied().unwrap_or(true),
        }
    });

    let border_radius = match style { 0 => "16px", 1 => "6px", _ => "10px" };
    let chip_radius = match style { 0 => "20px", 1 => "4px", _ => "8px" };
    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; \
         background: white; border-radius: {}; \
         box-shadow: 0 4px 24px rgba(0,0,0,0.3); \
         font-family: system-ui, sans-serif; box-sizing: border-box; padding: 16px;",
        card_x, card_y, card_w, border_radius
    );

    let submit_bg = if is_wrong { "#ef4444" } else { &accent };

    // Ground truth
    let tags_desc: String = available.iter().enumerate().map(|(i, &si)| {
        let label = scenario.tags[si];
        let sel_mark = if cur_sel.get(i).copied().unwrap_or(false) { " [SEL]" } else { "" };
        let target_mark = if target_tags.contains(&i) { " (TARGET)" } else { "" };
        format!("\"{}\"{}{}",  label, sel_mark, target_mark)
    }).collect::<Vec<_>>().join(", ");
    let position_desc = describe_position(card_x, card_y, card_w, 280.0);
    let description = format!(
        "multi-select tags, title: \"{}\", mode: {}, tags: [{}], style: {}, at {}",
        title,
        match mode { TagMode::Add => "add", TagMode::Remove => "remove" },
        tags_desc,
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
                    "Level 26"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Multi-Select Tags"
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

                    h3 {
                        style: "margin: 0 0 4px 0; font-size: 16px; color: #111827; font-weight: 600;",
                        "{title}"
                    }
                    p {
                        style: "margin: 0 0 12px 0; font-size: 12px; color: #9ca3af;",
                        "Click tags to select or remove them"
                    }

                    // Tags area
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 16px;",

                        for ti in 0..tag_count {
                            {
                                let si = available[ti];
                                let label = scenario.tags[si];
                                let is_sel = cur_sel.get(ti).copied().unwrap_or(false);
                                let accent_c = accent.clone();

                                let chip_bg = if is_sel {
                                    format!("{}18", accent_c)
                                } else {
                                    "#f3f4f6".to_string()
                                };
                                let chip_border = if is_sel {
                                    format!("1.5px solid {}", accent_c)
                                } else {
                                    "1.5px solid #e5e7eb".to_string()
                                };
                                let chip_color = if is_sel {
                                    accent_c.clone()
                                } else {
                                    "#6b7280".to_string()
                                };

                                let is_target = target_tags.contains(&ti);
                                let chip_style = format!(
                                    "display: inline-flex; align-items: center; gap: 6px; \
                                     padding: 6px 14px; background: {}; border: {}; \
                                     border-radius: {}; font-size: 13px; color: {}; \
                                     cursor: pointer; font-family: system-ui, sans-serif; \
                                     font-weight: {}; transition: all 0.15s;",
                                    chip_bg, chip_border, chip_radius, chip_color,
                                    if is_sel { "600" } else { "400" }
                                );

                                rsx! {
                                    button {
                                        class: if is_target { "target" } else { "" },
                                        "data-label": "{label}",
                                        style: "{chip_style}",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            let mut s = selected.write();
                                            if let Some(val) = s.get_mut(ti) {
                                                *val = !*val;
                                            }
                                        },
                                        span { "{label}" }
                                        if is_sel {
                                            span {
                                                style: "font-size: 11px; opacity: 0.6;",
                                                "\u{2715}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: {chip_radius}; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s;",
                        tabindex: "-1",
                        onclick: move |_| {
                            if is_correct {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level26();
                                let new_sel = new_st.initially_selected.clone();
                                state.set(new_st);
                                selected.set(new_sel);
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
                target_h: 280.0,
                steps: {
                    let mut parts: Vec<String> = target_labels.iter()
                        .map(|l| format!(r#"{{"action":"click","target":"{}"}}"#, l))
                        .collect();
                    parts.push(r#"{"action":"click","target":"Submit"}"#.to_string());
                    format!("[{}]", parts.join(","))
                },
            }
        }
    }
}
