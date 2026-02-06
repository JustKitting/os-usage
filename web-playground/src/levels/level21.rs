use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::ui_node::{self, UINode, Visual, Rect};
use super::{fresh_rng, random_canvas_bg, ordinal};

const SECTION_LABELS: &[&str] = &[
    "Personal Information", "Payment Details", "Shipping Address",
    "Order Summary", "Account Settings", "Notifications",
    "Privacy Policy", "Terms of Service", "FAQ",
    "Contact Us", "Return Policy", "Warranty Info",
    "Technical Specs", "Customer Reviews", "Product Description",
];

const SECTION_CONTENTS: &[&str] = &[
    "Please provide your full name, date of birth, and contact information. All fields marked with an asterisk are required.",
    "We accept Visa, Mastercard, American Express, and PayPal. Your payment information is encrypted and stored securely.",
    "Enter your shipping address including street, city, state, and ZIP code. We offer free shipping on orders over $50.",
    "Review your selected items, quantities, and total price before completing your purchase. Taxes calculated at checkout.",
    "Manage your account preferences, change your password, and update your email notification settings here.",
    "Choose which notifications you'd like to receive. You can opt out of marketing emails at any time.",
    "We value your privacy. Read our full privacy policy to understand how we collect and use your data.",
    "By using our service, you agree to these terms. Please read them carefully before proceeding.",
    "Find answers to commonly asked questions about our products, shipping, returns, and account management.",
    "Reach our support team via email, phone, or live chat. Our hours of operation are Monday through Friday, 9am to 5pm.",
    "Items may be returned within 30 days of purchase. Items must be in original condition with tags attached.",
    "All products come with a one-year limited warranty covering manufacturing defects. See full terms for details.",
    "Dimensions: 10 x 8 x 3 inches. Weight: 2.5 lbs. Material: aluminum alloy. Battery life: up to 12 hours.",
    "Rated 4.5 out of 5 stars based on 1,247 reviews. Customers love the build quality and ease of use.",
    "A versatile and durable product designed for everyday use. Features premium materials and modern design.",
];

const ACCENT_COLORS: &[&str] = &[
    "#4f46e5", "#2563eb", "#0891b2", "#059669", "#d97706",
    "#dc2626", "#7c3aed", "#db2777", "#0d9488", "#ea580c",
];

#[derive(Clone)]
struct SectionInfo {
    label: String,
    content: String,
}

struct Level21State {
    sections: Vec<SectionInfo>,
    target_section: usize,
    initially_open: Vec<bool>,
    mode: u8,
    style: u8,
    accent: String,
    x: f32,
    y: f32,
    card_w: f32,
}

fn random_level21() -> Level21State {
    let mut rng = fresh_rng();
    let count = rng.random_range(3..=6usize);

    let mut label_pool: Vec<usize> = (0..SECTION_LABELS.len()).collect();
    let mut content_pool: Vec<usize> = (0..SECTION_CONTENTS.len()).collect();
    let mut sections = Vec::new();

    for _ in 0..count {
        let li = rng.random_range(0..label_pool.len());
        let label = SECTION_LABELS[label_pool.remove(li)].to_string();

        let ci = rng.random_range(0..content_pool.len());
        let content = SECTION_CONTENTS[content_pool.remove(ci)].to_string();

        sections.push(SectionInfo { label, content });
    }

    let target_section = rng.random_range(0..count);

    // Some sections may start open, but never the target
    let initially_open: Vec<bool> = (0..count)
        .map(|i| if i == target_section { false } else { rng.random_bool(0.25) })
        .collect();

    let mode = rng.random_range(0..2u8);
    let style = rng.random_range(0..3u8);
    let accent = ACCENT_COLORS[rng.random_range(0..ACCENT_COLORS.len())].to_string();

    let card_w = rng.random_range(340.0..=480.0f32);
    // Estimate height: header ~44px each, open content ~80px
    let open_count = initially_open.iter().filter(|&&o| o).count();
    let card_h = count as f32 * 48.0 + open_count as f32 * 80.0 + 120.0;
    let margin = 50.0;
    let (vp_w, vp_h) = crate::primitives::viewport_size();
    let (x, y) = super::safe_position_in(&mut rng, card_w, card_h, margin, vp_w * 1.3, vp_h * 1.3);

    Level21State { sections, target_section, initially_open, mode, style, accent, x, y, card_w }
}

#[component]
pub fn Level21() -> Element {
    let mut state = use_signal(|| random_level21());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_open: Vec<bool> = state.read().initially_open.clone();
    let mut open = use_signal(move || initial_open);
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let sections: Vec<SectionInfo> = st.sections.clone();
    let target_section = st.target_section;
    let mode = st.mode;
    let style = st.style;
    let accent = st.accent.clone();
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    drop(st);

    let section_count = sections.len();
    let is_wrong = wrong();
    let cur_open: Vec<bool> = open.read().clone();

    let target_label = sections[target_section].label.clone();

    let instruction = match mode {
        1 => {
            let ord = ordinal(target_section + 1);
            format!("Expand the {} section", ord)
        }
        _ => {
            format!("Expand \"{}\"", target_label)
        }
    };

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; box-sizing: border-box;",
        card_x, card_y, card_w
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth — build UINode tree
    let open_count = cur_open.iter().filter(|&&o| o).count();
    let est_h = section_count as f32 * 48.0 + open_count as f32 * 80.0 + 120.0;
    let card_rect = Rect::new(card_x, card_y, card_w, est_h);
    let children: Vec<UINode> = sections.iter().enumerate().map(|(i, s)| {
        let sec_rect = Rect::new(card_x, card_y, card_w, est_h);
        if i == target_section {
            ui_node::accordion(&s.label, sec_rect)
        } else {
            // Non-target accordion section
            UINode::Accordion(Visual::new(&s.label, sec_rect))
        }
    }).collect();
    let tree = ui_node::form(card_rect, "Submit", children);
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
                    "Level 13"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "Accordion"
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
                        style: "margin: 0 0 12px 0; font-size: 14px; color: #374151; font-weight: 500;",
                        "{instruction}"
                    }

                    for si in 0..section_count {
                        {
                            let s = sections[si].clone();
                            let is_open = cur_open.get(si).copied().unwrap_or(false);
                            let is_last = si == section_count - 1;
                            let accent_c = accent.clone();

                            let chevron = if is_open { "\u{25B2}" } else { "\u{25BC}" };

                            let (wrapper_style, header_style, content_style, icon_str) = match style {
                                // Style 0: divider lines
                                0 => {
                                    let border = if is_last { "none" } else { "1px solid #e5e7eb" };
                                    let ws = format!("border-bottom: {};", border);
                                    let hs = format!("display: flex; justify-content: space-between; align-items: center; padding: 12px 0; cursor: pointer; user-select: none; background: none; border: none; width: 100%; text-align: left; font-family: system-ui, sans-serif;");
                                    let cs = "padding: 0 0 12px 0; font-size: 13px; color: #6b7280; line-height: 1.5;".to_string();
                                    (ws, hs, cs, chevron.to_string())
                                }
                                // Style 1: card sections with gap
                                1 => {
                                    let mb = if is_last { "0" } else { "8px" };
                                    let bg = if is_open { "#f9fafb" } else { "#ffffff" };
                                    let ws = format!("background: {}; border: 1px solid #e5e7eb; border-radius: 8px; margin-bottom: {}; overflow: hidden;", bg, mb);
                                    let hs = "display: flex; justify-content: space-between; align-items: center; padding: 12px; cursor: pointer; user-select: none; background: none; border: none; width: 100%; text-align: left; font-family: system-ui, sans-serif; box-sizing: border-box;".to_string();
                                    let cs = "padding: 0 12px 12px 12px; font-size: 13px; color: #6b7280; line-height: 1.5;".to_string();
                                    let icon = if is_open { "\u{2212}" } else { "+" };
                                    (ws, hs, cs, icon.to_string())
                                }
                                // Style 2: minimal
                                _ => {
                                    let mb = if is_last { "0" } else { "4px" };
                                    let ws = format!("margin-bottom: {};", mb);
                                    let hs = "display: flex; gap: 8px; align-items: center; padding: 8px 0; cursor: pointer; user-select: none; background: none; border: none; width: 100%; text-align: left; font-family: system-ui, sans-serif;".to_string();
                                    let cs = "padding: 0 0 8px 20px; font-size: 13px; color: #6b7280; line-height: 1.5;".to_string();
                                    let icon = if is_open { "\u{25B8}" } else { "\u{25B8}" };
                                    (ws, hs, cs, icon.to_string())
                                }
                            };

                            let label_color = if is_open { accent_c.clone() } else { "#111827".to_string() };
                            let icon_color = if is_open { accent_c } else { "#9ca3af".to_string() };
                            let icon_transform = if style == 2 && is_open { "display: inline-block; transform: rotate(90deg); transition: transform 0.15s;" } else if style == 2 { "display: inline-block; transition: transform 0.15s;" } else { "" };

                            rsx! {
                                div {
                                    style: "{wrapper_style}",

                                    button {
                                        class: if si == target_section { "target" } else { "" },
                                        "data-label": "{s.label}",
                                        style: "{header_style}",
                                        tabindex: "-1",
                                        onclick: move |_| {
                                            let mut o = open.write();
                                            if let Some(val) = o.get_mut(si) {
                                                *val = !*val;
                                            }
                                        },

                                        span {
                                            style: "font-size: 14px; font-weight: 500; color: {label_color};",
                                            "{s.label}"
                                        }

                                        span {
                                            style: "font-size: 12px; color: {icon_color}; {icon_transform}",
                                            "{icon_str}"
                                        }
                                    }

                                    if is_open {
                                        div {
                                            style: "{content_style}",
                                            "{s.content}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Submit
                    button {
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; transition: background 0.15s; margin-top: 16px;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let is_target_open = open.read().get(target_section).copied().unwrap_or(false);
                            if is_target_open {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level21();
                                let new_open = new_st.initially_open.clone();
                                state.set(new_st);
                                open.set(new_open);
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
                target_h: est_h,
                tree: Some(tree.clone()),
            }
        }
    }
}
