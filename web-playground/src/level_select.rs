use dioxus::prelude::*;
use crate::Route;

struct LevelInfo {
    name: &'static str,
    desc: &'static str,
    route: Route,
}

const LEVELS: &[LevelInfo] = &[
    // --- Basic single controls ---
    LevelInfo { name: "Level 1",  desc: "Click the button",         route: Route::Level1 {} },
    LevelInfo { name: "Level 2",  desc: "Toggle the switch",        route: Route::Level2 {} },
    LevelInfo { name: "Level 3",  desc: "Type the word",            route: Route::Level3 {} },
    LevelInfo { name: "Level 4",  desc: "Select the right option",  route: Route::Level4 {} },
    LevelInfo { name: "Level 5",  desc: "Radio buttons",            route: Route::Level17 {} },
    LevelInfo { name: "Level 6",  desc: "Slider",                   route: Route::Level16 {} },
    LevelInfo { name: "Level 7",  desc: "Number stepper",           route: Route::Level18 {} },
    LevelInfo { name: "Level 8",  desc: "Star rating",              route: Route::Level19 {} },
    LevelInfo { name: "Level 9",  desc: "Tabs",                     route: Route::Level20 {} },
    // --- Scrolling ---
    LevelInfo { name: "Level 10", desc: "Scroll & click",           route: Route::LevelScroll {} },
    // --- Targeted identification ---
    LevelInfo { name: "Level 11", desc: "Find the right button",    route: Route::Level5 {} },
    LevelInfo { name: "Level 12", desc: "Click the right toggle",   route: Route::Level6 {} },
    LevelInfo { name: "Level 13", desc: "Type into the right input", route: Route::Level7 {} },
    LevelInfo { name: "Level 14", desc: "Accordion",                route: Route::Level21 {} },
    // --- Multi-element compound ---
    LevelInfo { name: "Level 15", desc: "Multi-dropdown",           route: Route::Level8 {} },
    LevelInfo { name: "Level 16", desc: "Mixed inputs",             route: Route::Level9 {} },
    LevelInfo { name: "Level 17", desc: "Form submission",          route: Route::Level10 {} },
    // --- Complex compound ---
    LevelInfo { name: "Level 18", desc: "Carousel reading",         route: Route::Level11 {} },
    LevelInfo { name: "Level 19", desc: "Grid form",                route: Route::Level12 {} },
    LevelInfo { name: "Level 20", desc: "Table input",              route: Route::Level13 {} },
    LevelInfo { name: "Level 21", desc: "License agreement",        route: Route::Level14 {} },
    LevelInfo { name: "Level 22", desc: "Drag & drop",              route: Route::Level15 {} },
    LevelInfo { name: "Level 23", desc: "Modal dialog",             route: Route::Level22 {} },
    LevelInfo { name: "Level 24", desc: "Context menu",             route: Route::Level23 {} },
    LevelInfo { name: "Level 25", desc: "Search autocomplete",      route: Route::Level24 {} },
    LevelInfo { name: "Level 26", desc: "Sortable list",            route: Route::Level25 {} },
    LevelInfo { name: "Level 27", desc: "Multi-select tags",        route: Route::Level26 {} },
    LevelInfo { name: "Level 28", desc: "Toast dismiss",            route: Route::Level27 {} },
];

const COLS: usize = 4;
const ROWS: usize = 5;
const PER_PAGE: usize = COLS * ROWS;

static LEVEL_PAGE: GlobalSignal<usize> = Signal::global(|| 0);

/// Total number of slots (levels + locked placeholders) to fill pages evenly
fn total_slots() -> usize {
    let count = LEVELS.len().max(PER_PAGE);
    // Round up to next multiple of PER_PAGE
    ((count + PER_PAGE - 1) / PER_PAGE) * PER_PAGE
}

fn total_pages() -> usize {
    (total_slots() + PER_PAGE - 1) / PER_PAGE
}

fn set_page(page: &mut Signal<usize>, val: usize) {
    page.set(val);
    *LEVEL_PAGE.write() = val;
}

#[component]
pub fn LevelSelect() -> Element {
    let initial = *LEVEL_PAGE.read();
    let mut page = use_signal(move || initial);
    let pages = total_pages();
    let slots = total_slots();

    let start = page() * PER_PAGE;
    let end = (start + PER_PAGE).min(slots);

    rsx! {
        div {
            style: "min-height: 100vh; background: #0f0f1a; display: flex; flex-direction: column; align-items: center; padding: 40px 20px; font-family: system-ui, sans-serif;",

            // Header
            div {
                style: "display: flex; gap: 16px; align-items: center; margin-bottom: 40px;",
                Link {
                    to: Route::Landing {},
                    style: "color: #6b7280; text-decoration: none; font-size: 14px;",
                    "\u{2190} Home"
                }
                h1 {
                    style: "color: #e5e7eb; margin: 0; font-size: 32px; font-weight: 700;",
                    "Levels"
                }
            }

            // Level cards grid — fixed 4 columns
            div {
                style: "display: grid; grid-template-columns: repeat(4, 180px); gap: 16px;",

                for idx in start..end {
                    if idx < LEVELS.len() {
                        {
                            let level = &LEVELS[idx];
                            rsx! {
                                Link {
                                    to: level.route.clone(),
                                    style: "background: #1a1a2e; border: 1px solid #2a2a4a; border-radius: 10px; padding: 24px; text-decoration: none; transition: border-color 0.2s;",
                                    div {
                                        style: "color: #6366f1; font-size: 13px; font-weight: 600; margin-bottom: 8px; font-family: monospace;",
                                        "{idx + 1}"
                                    }
                                    h3 {
                                        style: "color: #e5e7eb; font-size: 18px; margin: 0 0 8px 0;",
                                        "{level.name}"
                                    }
                                    p {
                                        style: "color: #6b7280; font-size: 14px; margin: 0;",
                                        "{level.desc}"
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            style: "background: #12121f; border: 1px solid #1f1f35; border-radius: 10px; padding: 24px; opacity: 0.4;",
                            div {
                                style: "color: #4b5563; font-size: 13px; font-weight: 600; margin-bottom: 8px; font-family: monospace;",
                                "{idx + 1}"
                            }
                            h3 {
                                style: "color: #4b5563; font-size: 18px; margin: 0 0 8px 0;",
                                "Coming soon"
                            }
                            p {
                                style: "color: #374151; font-size: 14px; margin: 0;",
                                "..."
                            }
                        }
                    }
                }
            }

            // Page selector
            if pages > 1 {
                {
                    let cur = page();
                    let prev_bg = if cur == 0 { "#1a1a2e" } else { "#2a2a4a" };
                    let prev_color = if cur == 0 { "#4b5563" } else { "#e5e7eb" };
                    let next_bg = if cur == pages - 1 { "#1a1a2e" } else { "#2a2a4a" };
                    let next_color = if cur == pages - 1 { "#4b5563" } else { "#e5e7eb" };
                    rsx! {
                        div {
                            style: "display: flex; gap: 8px; align-items: center; margin-top: 32px;",

                            // Previous
                            button {
                                style: "padding: 8px 14px; background: {prev_bg}; color: {prev_color}; border: 1px solid #2a2a4a; border-radius: 6px; font-size: 14px; cursor: pointer; font-family: system-ui, sans-serif;",
                                disabled: cur == 0,
                                onclick: move |_| { let v = page().saturating_sub(1); set_page(&mut page, v); },
                                "\u{2190}"
                            }

                            // Page numbers
                            for p in 0..pages {
                                {
                                    let is_current = p == cur;
                                    let bg = if is_current { "#6366f1" } else { "#1a1a2e" };
                                    let color = if is_current { "white" } else { "#9ca3af" };
                                    let border = if is_current { "#6366f1" } else { "#2a2a4a" };
                                    rsx! {
                                        button {
                                            style: "width: 36px; height: 36px; background: {bg}; color: {color}; border: 1px solid {border}; border-radius: 6px; font-size: 14px; cursor: pointer; font-family: monospace; font-weight: 600;",
                                            onclick: move |_| set_page(&mut page, p),
                                            "{p + 1}"
                                        }
                                    }
                                }
                            }

                            // Next
                            button {
                                style: "padding: 8px 14px; background: {next_bg}; color: {next_color}; border: 1px solid #2a2a4a; border-radius: 6px; font-size: 14px; cursor: pointer; font-family: system-ui, sans-serif;",
                                disabled: cur == pages - 1,
                                onclick: move |_| { let v = (page() + 1).min(pages - 1); set_page(&mut page, v); },
                                "\u{2192}"
                            }
                        }
                    }
                }
            }
        }
    }
}
