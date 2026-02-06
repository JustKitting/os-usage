use dioxus::prelude::*;
use rand::Rng;

use crate::Route;
use crate::primitives::Position;
use super::{fresh_rng, random_canvas_bg, ordinal, describe_position};

const LEGAL_PARAGRAPHS: &[&str] = &[
    "By accessing or using this service, you acknowledge that you have read, understood, and agree to be bound by these terms and conditions. These terms constitute a legally binding agreement between you and the service provider. Any modifications to these terms will be effective upon posting.",
    "The service provider reserves the right to modify, suspend, or discontinue any aspect of the service at any time without prior notice. Continued use of the service after such modifications constitutes acceptance of the updated terms. You are encouraged to review these terms periodically.",
    "You agree not to use the service for any unlawful purpose or in any way that could damage, disable, overburden, or impair the service. You are solely responsible for all activities conducted under your account and must maintain the confidentiality of your credentials at all times.",
    "All content, features, and functionality of the service are owned by the service provider and are protected by international copyright, trademark, patent, trade secret, and other intellectual property laws. Unauthorized reproduction or distribution is strictly prohibited.",
    "The service is provided on an \"as is\" and \"as available\" basis without warranties of any kind, either express or implied, including but not limited to implied warranties of merchantability, fitness for a particular purpose, and non-infringement of third-party rights.",
    "In no event shall the service provider be liable for any indirect, incidental, special, consequential, or punitive damages, including without limitation, loss of profits, data, use, goodwill, or other intangible losses resulting from your use of or inability to use the service.",
    "You agree to indemnify and hold harmless the service provider and its affiliates, officers, agents, and employees from and against any claims, liabilities, damages, losses, and expenses arising out of or in any way connected with your access to or use of the service.",
    "The service provider may collect and process personal data in accordance with its privacy policy. By using the service, you consent to such processing and warrant that all data provided by you is accurate, current, and complete to the best of your knowledge.",
    "These terms shall be governed by and construed in accordance with the laws of the applicable jurisdiction, without regard to its conflict of law provisions. Any legal action related to these terms must be brought within one year of the cause of action arising.",
    "Any dispute arising from or relating to these terms shall be resolved through binding arbitration in accordance with the rules of the applicable arbitration association. The arbitrator's decision shall be final, binding, and enforceable in any court of competent jurisdiction.",
    "The service provider may assign or transfer these terms, in whole or in part, without restriction. You may not assign or transfer any rights or obligations under these terms without the prior written consent of the service provider, and any attempted assignment shall be void.",
    "If any provision of these terms is found to be unenforceable or invalid under applicable law, that provision shall be limited or eliminated to the minimum extent necessary so that the remaining provisions of these terms shall remain in full force and effect.",
    "The failure of the service provider to exercise or enforce any right or provision of these terms shall not constitute a waiver of such right or provision. No waiver of any term shall be deemed a further or continuing waiver of such term or any other term.",
    "You acknowledge that the service provider may establish general practices and limits concerning use of the service, including without limitation the maximum period of time that data, content, or other uploaded materials will be retained by the service.",
    "The service provider reserves the right to refuse service, terminate accounts, remove or edit content, or cancel orders at its sole discretion, including without limitation if the provider believes that your conduct violates applicable law or is harmful to the interests of other users, third parties, or the service provider.",
    "All notices and communications related to these terms shall be in writing and shall be deemed to have been duly given when received, whether delivered personally, by certified or registered mail, return receipt requested, or by recognized overnight courier service.",
];

const CHECKBOX_LABELS: &[&str] = &[
    "I have read and agree to the Terms of Service",
    "I accept the Privacy Policy",
    "I acknowledge the Data Processing Agreement",
    "I consent to receiving electronic communications",
    "I agree to the Acceptable Use Policy",
    "I confirm I am at least 18 years of age",
    "I accept the End User License Agreement",
    "I agree to the Arbitration Clause",
    "I acknowledge the Limitation of Liability",
    "I consent to data collection as described above",
    "I accept the Intellectual Property terms",
    "I agree to the Indemnification provisions",
];

const AGREEMENT_TITLES: &[&str] = &[
    "License Agreement",
    "Terms of Service",
    "End User License Agreement",
    "Terms and Conditions",
    "Privacy Policy Agreement",
    "Service Agreement",
];

struct Level14State {
    title: String,
    sections: Vec<(String, Option<String>)>, // (paragraph, optional checkbox label)
    checkbox_count: usize,
    target_checkboxes: Vec<usize>,
    mode: u8, // 0=all, 1=ordinal, 2=by label
    target_label: String,
    x: f32,
    y: f32,
    card_w: f32,
    card_h: f32,
}

fn random_level14() -> Level14State {
    let mut rng = fresh_rng();

    let title = AGREEMENT_TITLES[rng.random_range(0..AGREEMENT_TITLES.len())].to_string();
    let para_count = rng.random_range(10..=14usize);
    let cb_count = rng.random_range(3..=5usize);

    // Checkbox positions: after paragraphs 2..para_count-2 (ensure text above & below)
    let mut available: Vec<usize> = (2..para_count.saturating_sub(2)).collect();
    let mut cb_positions: Vec<usize> = Vec::new();
    for _ in 0..cb_count.min(available.len()) {
        let i = rng.random_range(0..available.len());
        cb_positions.push(available.remove(i));
    }
    cb_positions.sort();
    let cb_count = cb_positions.len();

    // Pick paragraphs (allow repeats if needed)
    let mut para_pool: Vec<usize> = (0..LEGAL_PARAGRAPHS.len()).collect();
    let mut paragraphs: Vec<String> = Vec::new();
    for _ in 0..para_count {
        if para_pool.is_empty() {
            para_pool = (0..LEGAL_PARAGRAPHS.len()).collect();
        }
        let i = rng.random_range(0..para_pool.len());
        paragraphs.push(LEGAL_PARAGRAPHS[para_pool.remove(i)].to_string());
    }

    // Pick checkbox labels
    let mut label_pool: Vec<usize> = (0..CHECKBOX_LABELS.len()).collect();
    let mut cb_labels: Vec<String> = Vec::new();
    for _ in 0..cb_count {
        let i = rng.random_range(0..label_pool.len());
        cb_labels.push(CHECKBOX_LABELS[label_pool.remove(i)].to_string());
    }

    // Build sections
    let mut sections: Vec<(String, Option<String>)> = Vec::new();
    let mut cb_idx = 0;
    for (pi, para) in paragraphs.into_iter().enumerate() {
        let cb = if cb_idx < cb_count && cb_positions[cb_idx] == pi {
            let label = cb_labels[cb_idx].clone();
            cb_idx += 1;
            Some(label)
        } else {
            None
        };
        sections.push((para, cb));
    }

    // Mode & target
    let mode = rng.random_range(0..3u8);
    let mut target_checkboxes = Vec::new();
    let mut target_label = String::new();
    match mode {
        0 => { target_checkboxes = (0..cb_count).collect(); }
        1 => {
            let idx = rng.random_range(0..cb_count);
            target_checkboxes.push(idx);
        }
        _ => {
            let idx = rng.random_range(0..cb_count);
            target_checkboxes.push(idx);
            target_label = cb_labels[idx].clone();
        }
    }

    let card_w = rng.random_range(380.0..=500.0f32);
    let card_h = rng.random_range(450.0..=600.0f32);
    let margin = 40.0;
    let x = rng.random_range(margin..(Position::VIEWPORT - card_w - margin).max(margin + 1.0));
    let y = rng.random_range(margin..(Position::VIEWPORT - card_h - margin).max(margin + 1.0));

    Level14State { title, sections, checkbox_count: cb_count, target_checkboxes, mode, target_label, x, y, card_w, card_h }
}

#[component]
pub fn Level14() -> Element {
    let mut state = use_signal(|| random_level14());
    let mut score = use_signal(|| 0u32);
    let mut bg = use_signal(|| random_canvas_bg());
    let initial_cb = state.read().checkbox_count;
    let mut checks = use_signal(move || vec![false; initial_cb]);
    let mut wrong = use_signal(|| false);

    let st = state.read();
    let title = st.title.clone();
    let sections: Vec<(String, Option<String>)> = st.sections.clone();
    let checkbox_count = st.checkbox_count;
    let target_checkboxes: Vec<usize> = st.target_checkboxes.clone();
    let mode = st.mode;
    let target_label = st.target_label.clone();
    let card_x = st.x;
    let card_y = st.y;
    let card_w = st.card_w;
    let card_h = st.card_h;
    drop(st);

    let is_wrong = wrong();
    let checks_snap: Vec<bool> = checks.read().clone();
    let section_count = sections.len();

    // Precompute checkbox index for each section
    let section_cb_idx: Vec<Option<usize>> = {
        let mut idx = 0usize;
        sections.iter().map(|(_, cb)| {
            if cb.is_some() {
                let i = idx;
                idx += 1;
                Some(i)
            } else {
                None
            }
        }).collect()
    };

    let instruction = match mode {
        0 => "Check all checkboxes and click Accept".to_string(),
        1 => format!("Check the {} checkbox and click Accept", ordinal(target_checkboxes[0] + 1)),
        _ => format!("Check \"{}\" and click Accept", target_label),
    };

    let card_style = format!(
        "position: absolute; left: {}px; top: {}px; background: white; border-radius: 12px; padding: 16px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); font-family: system-ui, sans-serif; width: {}px; height: {}px; display: flex; flex-direction: column;",
        card_x, card_y, card_w, card_h
    );
    let submit_bg = if is_wrong { "#ef4444" } else { "#4f46e5" };

    // Ground truth
    let cb_descs: Vec<String> = {
        let mut idx = 0;
        let mut descs = Vec::new();
        for (si, (_, cb)) in sections.iter().enumerate() {
            if let Some(label) = cb {
                let is_target = target_checkboxes.contains(&idx);
                let marker = if is_target { " (TARGET)" } else { "" };
                descs.push(format!("#{} after para {}: \"{}\"{}", idx + 1, si + 1, label, marker));
                idx += 1;
            }
        }
        descs
    };
    let position_desc = describe_position(card_x, card_y, card_w + 32.0, card_h);
    let description = format!(
        "\"{}\", {} paragraphs, {} checkboxes: [{}], mode: {}, at {}",
        title, section_count, checkbox_count,
        cb_descs.join(", "),
        match mode {
            0 => "check all".to_string(),
            1 => format!("ordinal ({})", ordinal(target_checkboxes[0] + 1)),
            _ => format!("by label \"{}\"", target_label),
        },
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
                    "Level 20"
                }
                span {
                    style: "color: #6b7280; font-size: 14px;",
                    "License"
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
                    style: "{card_style}",

                    h3 {
                        style: "margin: 0 0 6px 0; font-size: 15px; color: #111; font-weight: 700; flex-shrink: 0;",
                        "{title}"
                    }

                    p {
                        style: "margin: 0 0 10px 0; font-size: 13px; color: #4f46e5; font-weight: 600; flex-shrink: 0;",
                        "{instruction}"
                    }

                    // Scrollable content
                    div {
                        style: "flex: 1; overflow-y: auto; border: 1px solid #e5e7eb; border-radius: 6px; padding: 12px; margin-bottom: 10px; font-size: 12px; color: #374151; line-height: 1.6; min-height: 0;",

                        for si in 0..section_count {
                            {
                                let para = sections[si].0.clone();
                                let has_cb = sections[si].1.is_some();
                                let cb_label = sections[si].1.clone().unwrap_or_default();
                                let cb_idx = section_cb_idx[si].unwrap_or(0);
                                let is_checked = has_cb && checks_snap.get(cb_idx).copied().unwrap_or(false);

                                rsx! {
                                    div {
                                        p {
                                            style: "margin: 0 0 12px 0;",
                                            "{para}"
                                        }
                                        if has_cb {
                                            div {
                                                class: if target_checkboxes.contains(&cb_idx) { "target" } else { "" },
                                                "data-label": "{cb_label}",
                                                style: "display: flex; align-items: flex-start; gap: 8px; padding: 8px 10px; margin: 4px 0 16px 0; background: #f3f4f6; border: 1px solid #d1d5db; border-radius: 6px; cursor: pointer;",
                                                onclick: move |_| {
                                                    let mut vals = checks.write();
                                                    if let Some(v) = vals.get_mut(cb_idx) {
                                                        *v = !*v;
                                                    }
                                                },
                                                input {
                                                    r#type: "checkbox",
                                                    tabindex: "-1",
                                                    checked: is_checked,
                                                    style: "width: 16px; height: 16px; margin-top: 1px; accent-color: #4f46e5; pointer-events: none; flex-shrink: 0;",
                                                }
                                                span {
                                                    style: "font-size: 12px; color: #374151; user-select: none;",
                                                    "{cb_label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Accept button
                    button {
                        class: "target",
                        style: "width: 100%; padding: 10px; background: {submit_bg}; color: white; border: none; border-radius: 6px; font-size: 14px; font-weight: 600; font-family: system-ui, sans-serif; cursor: pointer; box-sizing: border-box; flex-shrink: 0; transition: background 0.15s;",
                        tabindex: "-1",
                        onclick: move |_| {
                            let vals = checks.read();
                            let ok = target_checkboxes.iter().all(|&i| vals.get(i).copied().unwrap_or(false));
                            drop(vals);
                            if ok {
                                score.set(score() + 1);
                                bg.set(random_canvas_bg());
                                let new_st = random_level14();
                                let count = new_st.checkbox_count;
                                state.set(new_st);
                                checks.set(vec![false; count]);
                                wrong.set(false);
                                document::eval("document.activeElement?.blur()");
                            } else {
                                wrong.set(true);
                                spawn(async move {
                                    gloo_timers::future::TimeoutFuture::new(600).await;
                                    wrong.set(false);
                                });
                            }
                        },
                        "Accept"
                    }
                }
            }

            super::GroundTruth {
                description: description,
                target_x: card_x,
                target_y: card_y,
                target_w: card_w + 32.0,
                target_h: card_h,
                steps: {
                    let mut parts: Vec<String> = target_checkboxes.iter()
                        .filter_map(|&ci| sections.iter().filter_map(|(_, opt)| opt.as_ref()).nth(ci))
                        .map(|label| format!(r#"{{"action":"click","target":"{}"}}"#, label))
                        .collect();
                    parts.push(r#"{"action":"click","target":"Accept"}"#.to_string());
                    format!("[{}]", parts.join(","))
                },
            }
        }
    }
}
