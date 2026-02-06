//! Ground truth resolution — recursive traversal of UINode tree.
//!
//! Each node variant contributes its piece of the description, action steps,
//! thinking chain, and target bounding boxes. Containers recurse into children.

use super::*;

/// Complete ground truth output from resolving a UINode tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedGroundTruth {
    /// Human-readable description of the UI state.
    pub description: String,
    /// Ordered action sequence for the solver.
    pub steps: Vec<Action>,
    /// VLM reasoning chain.
    pub thinking: String,
    /// All labeled bounding boxes: (label, rect) pairs.
    pub targets: Vec<(String, Rect)>,
}

impl ResolvedGroundTruth {
    /// Serialize the steps to the JSON format expected by the solver/GroundTruth component.
    pub fn steps_json(&self) -> String {
        actions_to_json(&self.steps)
    }
}

impl UINode {
    /// Resolve this node tree into complete ground truth (viewport-local coords).
    pub fn resolve(&self) -> ResolvedGroundTruth {
        self.resolve_with(&ViewportTransform::identity())
    }

    /// Resolve with a viewport transform — coordinates in thinking/description
    /// will be in window space.
    pub fn resolve_with(&self, vt: &ViewportTransform) -> ResolvedGroundTruth {
        let mut desc_parts = Vec::new();
        let mut steps = Vec::new();
        let mut think_parts = Vec::new();
        let mut targets = Vec::new();

        self.resolve_inner(&mut desc_parts, &mut steps, &mut think_parts, &mut targets, None, vt);

        ResolvedGroundTruth {
            description: desc_parts.join(", "),
            steps,
            thinking: think_parts.join(" "),
            targets,
        }
    }

    fn resolve_inner(
        &self,
        desc: &mut Vec<String>,
        steps: &mut Vec<Action>,
        think: &mut Vec<String>,
        targets: &mut Vec<(String, Rect)>,
        parent: Option<(&str, &Rect)>,
        vt: &ViewportTransform,
    ) {
        let v = self.visual();
        // Region is relative to parent (or viewport), coords are window-absolute
        let (wx, wy, ww, wh) = vt.apply(&v.rect);
        let pos = match parent {
            Some((parent_label, parent_rect)) => format!(
                "near the {} of the {} ({},{} {}x{})",
                v.rect.region_within(parent_rect), parent_label,
                wx, wy, ww, wh,
            ),
            None => format!(
                "near the {} ({},{} {}x{})",
                v.rect.region(), wx, wy, ww, wh,
            ),
        };
        let color_str = v.color.as_deref().unwrap_or("");

        match self {
            // ── Simple click targets ────────────────────────────────

            UINode::Button(v) => {
                let color_desc = color_prefix(color_str);
                desc.push(format!("{}button \"{}\" at {}", color_desc, v.label, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a {}button labeled \"{}\", located at {}. I should click it.",
                        color_desc, v.label, pos,
                    ));
                }
            }

            UINode::Toggle(v, state) => {
                let state_str = if state.is_on { "on" } else { "off" };
                desc.push(format!("toggle \"{}\" ({}) at {}", v.label, state_str, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a toggle labeled \"{}\", currently {}, located {}. I need to click it to switch it.",
                        v.label, state_str, pos,
                    ));
                }
            }

            UINode::Checkbox(v, state) => {
                let state_str = if state.is_checked { "checked" } else { "unchecked" };
                desc.push(format!("checkbox \"{}\" ({}) at {}", v.label, state_str, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a checkbox labeled \"{}\", currently {}, located {}. I need to click it.",
                        v.label, state_str, pos,
                    ));
                }
            }

            UINode::Tab(v) => {
                desc.push(format!("tab \"{}\" at {}", v.label, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a tab labeled \"{}\", located {}. I need to click it to switch to that tab.",
                        v.label, pos,
                    ));
                }
            }

            UINode::Accordion(v) => {
                desc.push(format!("accordion \"{}\" at {}", v.label, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a collapsible section labeled \"{}\", located {}. I need to click it to expand it.",
                        v.label, pos,
                    ));
                }
            }

            UINode::Tag(v, state) => {
                let state_str = if state.is_selected { "selected" } else { "unselected" };
                desc.push(format!("tag \"{}\" ({}) at {}", v.label, state_str, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a tag chip labeled \"{}\", currently {}, located {}. I need to click it.",
                        v.label, state_str, pos,
                    ));
                }
            }

            UINode::Toast(v, state) => {
                let dismiss_label = format!("dismiss: {}", state.message);
                desc.push(format!(
                    "toast ({}) \"{}\" at {}", state.kind, state.message, pos,
                ));
                targets.push((dismiss_label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&dismiss_label));
                    think.push(format!(
                        "I see a {} toast notification saying \"{}\", located {}. I need to dismiss it.",
                        state.kind, state.message, pos,
                    ));
                }
            }

            UINode::Star(v, state) => {
                desc.push(format!(
                    "star rating \"{}\" {}/{} target={} at {}",
                    v.label, state.current, state.max, state.target, pos,
                ));
                let star_label = format!("star {} of {}", state.target, v.label);
                targets.push((star_label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&star_label));
                    think.push(format!(
                        "I see a star rating for \"{}\" currently at {}/{}, located {}. I need to click star {} to set it to {}.",
                        v.label, state.current, state.max, pos, state.target, state.target,
                    ));
                }
            }

            UINode::ModalButton(v) => {
                let color_desc = color_prefix(color_str);
                desc.push(format!("{}modal button \"{}\" at {}", color_desc, v.label, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&v.label));
                    think.push(format!(
                        "I see a {}button labeled \"{}\" in the dialog, located {}. I should click it.",
                        color_desc, v.label, pos,
                    ));
                }
            }

            // ── Text input ──────────────────────────────────────────

            UINode::TextInput(v, state) => {
                desc.push(format!(
                    "text input \"{}\" placeholder=\"{}\" at {}",
                    v.label, state.placeholder, pos,
                ));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::type_text(&v.label, &state.target_value));
                    think.push(format!(
                        "I see a text input labeled \"{}\", located {}. I need to type \"{}\" into it.",
                        v.label, pos, state.target_value,
                    ));
                }
            }

            // ── Slider (drag) ───────────────────────────────────────

            UINode::Slider(v, state) => {
                let color_desc = color_prefix(color_str);
                desc.push(format!(
                    "{}slider \"{}\" range {}-{} step {} current={} target={} at {}",
                    color_desc, v.label, state.min, state.max, state.step,
                    state.current_val, state.target_val, pos,
                ));
                let from_label = format!("drag-from: {}", v.label);
                let to_label = format!("drag-to: {}", v.label);
                targets.push((from_label.clone(), state.thumb_rect));
                targets.push((to_label.clone(), state.target_thumb_rect));
                if v.is_target {
                    steps.push(Action::drag(&from_label, &to_label));
                    let direction = if state.target_val > state.current_val { "right" } else { "left" };
                    let delta = (state.target_val - state.current_val).abs();
                    think.push(format!(
                        "I see a {}slider labeled \"{}\" currently at {}, located {}. I need to drag it {} by {} to reach {}.",
                        color_desc, v.label, state.current_val, pos, direction, delta, state.target_val,
                    ));
                }
            }

            // ── Drag source / drop zone ─────────────────────────────

            UINode::DragSource(v) => {
                desc.push(format!("draggable \"{}\" at {}", v.label, pos));
                targets.push((v.label.clone(), v.rect));
                if v.is_target {
                    // Drag steps are typically constructed at the parent level
                    // since they need to reference the drop zone label.
                    think.push(format!(
                        "I see a draggable element labeled \"{}\", located {}. I need to drag it to the drop zone.",
                        v.label, pos,
                    ));
                }
            }

            UINode::DropZone(v) => {
                desc.push(format!("drop zone \"{}\" at {}", v.label, pos));
                targets.push((v.label.clone(), v.rect));
            }

            // ── Composite (multi-step) ──────────────────────────────

            UINode::Dropdown(v, state) => {
                let opts_str = state.options.iter()
                    .map(|o| format!("\"{}\"", o))
                    .collect::<Vec<_>>().join(", ");
                desc.push(format!(
                    "dropdown \"{}\" options=[{}] target=\"{}\" at {}",
                    v.label, opts_str, state.target_option, pos,
                ));
                targets.push((state.trigger_label.clone(), v.rect));
                if v.is_target {
                    steps.push(Action::click(&state.trigger_label));
                    steps.push(Action::click(&state.target_option));
                    think.push(format!(
                        "I see a dropdown labeled \"{}\", located {}. I need to click \"{}\" to open it, then select \"{}\".",
                        v.label, pos, state.trigger_label, state.target_option,
                    ));
                }
            }

            UINode::ContextMenu(v, state) => {
                let items_str = state.items.iter()
                    .map(|i| format!("\"{}\"", i))
                    .collect::<Vec<_>>().join(", ");
                desc.push(format!(
                    "context menu trigger=\"{}\" items=[{}] target=\"{}\" at {}",
                    state.trigger_label, items_str, state.target_item, pos,
                ));
                targets.push(("trigger".to_string(), v.rect));
                if v.is_target {
                    steps.push(Action::right_click(&state.trigger_label));
                    steps.push(Action::click(&state.target_item));
                    think.push(format!(
                        "I see an element I need to right-click, located {}. I'll right-click \"{}\", then select \"{}\" from the menu.",
                        pos, state.trigger_label, state.target_item,
                    ));
                }
            }

            UINode::Stepper(v, state) => {
                desc.push(format!(
                    "stepper \"{}\" range {}-{} step {} current={} target={} at {}",
                    v.label, state.min, state.max, state.step,
                    state.current_val, state.target_val, pos,
                ));
                targets.push((state.minus_label.clone(), v.rect));
                targets.push((state.plus_label.clone(), v.rect));
                if v.is_target {
                    let diff = state.target_val - state.current_val;
                    let n_clicks = (diff.abs() / state.step.max(1)) as usize;
                    let btn_label = if diff > 0 {
                        &state.plus_label
                    } else {
                        &state.minus_label
                    };
                    for _ in 0..n_clicks {
                        steps.push(Action::click(btn_label));
                    }
                    let direction = if diff > 0 { "increment" } else { "decrement" };
                    think.push(format!(
                        "I see a stepper labeled \"{}\" currently at {}, located {}. I need to {} it {} times to reach {}.",
                        v.label, state.current_val, pos, direction, n_clicks, state.target_val,
                    ));
                }
            }

            UINode::RadioGroup(v, state) => {
                let opts_str = state.options.iter().enumerate()
                    .map(|(i, o)| {
                        if i == state.target_option {
                            format!("\"{}\" (TARGET)", o)
                        } else {
                            format!("\"{}\"", o)
                        }
                    })
                    .collect::<Vec<_>>().join(", ");
                desc.push(format!(
                    "radio group \"{}\" options=[{}] at {}",
                    v.label, opts_str, pos,
                ));
                if v.is_target {
                    let target_name = &state.options[state.target_option];
                    steps.push(Action::click(target_name));
                    targets.push((target_name.clone(), v.rect));
                    think.push(format!(
                        "I see a radio group labeled \"{}\", located {}. I need to select the \"{}\" option.",
                        v.label, pos, target_name,
                    ));
                }
            }

            // ── Containers ──────────────────────────────────────────

            UINode::Card(_v, children) => {
                desc.push(format!("card at {}", pos));
                think.push(format!("I see a card {}.", pos));
                let ctx = Some(("card", &_v.rect));
                for child in children {
                    child.resolve_inner(desc, steps, think, targets, ctx, vt);
                }
                // Auto-detect DragSource+DropZone pairs and emit drag step
                emit_drag_pairs(children, steps);
            }

            UINode::Form(v, form_state, children) => {
                desc.push(format!("form at {}", pos));
                think.push(format!("I see a form {}.", pos));
                let ctx = Some(("form", &v.rect));
                for child in children {
                    child.resolve_inner(desc, steps, think, targets, ctx, vt);
                }
                emit_drag_pairs(children, steps);
                // Forms end with the submit click
                steps.push(Action::click(&form_state.submit_label));
                targets.push((form_state.submit_label.clone(), v.rect));
                let (sx, sy, sw, sh) = vt.apply(&v.rect);
                think.push(format!(
                    "After completing the form, I click \"{}\", located near the bottom of the form ({},{} {}x{}).",
                    form_state.submit_label, sx, sy, sw, sh,
                ));
            }
        }
    }
}

/// When a container has target DragSource(s) and DropZone(s), emit drag steps.
fn emit_drag_pairs(children: &[UINode], steps: &mut Vec<Action>) {
    let mut drop_label = None;
    for child in children {
        if let UINode::DropZone(v) = child {
            drop_label = Some(v.label.clone());
            break;
        }
    }
    if let Some(ref to) = drop_label {
        for child in children {
            if let UINode::DragSource(v) = child {
                if v.is_target {
                    steps.push(Action::drag(&v.label, to));
                }
            }
        }
    }
}

/// Helper: turns a color string into a prefix like "green " or empty string.
/// Accepts either english names ("green") or hex codes ("#4f46e5").
fn color_prefix(color: &str) -> String {
    if color.is_empty() {
        String::new()
    } else if color.starts_with('#') {
        // Map common hex codes to english names
        let name = match color {
            "#4f46e5" | "#7c3aed" => "indigo ",
            "#2563eb" => "blue ",
            "#0891b2" | "#0d9488" => "teal ",
            "#059669" => "green ",
            "#d97706" | "#ea580c" => "orange ",
            "#dc2626" | "#ef4444" => "red ",
            "#db2777" => "pink ",
            _ => "",
        };
        name.to_string()
    } else {
        format!("{} ", color)
    }
}
