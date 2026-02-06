//! Completion checking — validate current state against target state.
//!
//! The UINode tree is rebuilt each render with live values, so check()
//! compares the internal current vs target state without external input.
//! Click-based elements (Button, Tab, etc.) return `Complete` on click —
//! those are event-driven and checked by the caller, not by state comparison.

use super::*;

/// How complete is a task?
#[derive(Debug, Clone, PartialEq)]
pub enum Completion {
    /// No progress yet (or element has no checkable state).
    NotStarted,
    /// Some children are correct but not all.
    Partial { done: usize, total: usize },
    /// All target conditions are met.
    Complete,
    /// At least one value is wrong (not just incomplete — actively incorrect
    /// for elements where wrong is distinguishable from incomplete).
    Wrong,
}

impl Completion {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Self::Partial { .. })
    }

    /// Fraction of completion: 0.0 to 1.0.
    pub fn progress(&self) -> f32 {
        match self {
            Self::NotStarted => 0.0,
            Self::Partial { done, total } => {
                if *total == 0 { 0.0 } else { *done as f32 / *total as f32 }
            }
            Self::Complete => 1.0,
            Self::Wrong => 0.0,
        }
    }
}

impl UINode {
    /// Check how complete this node (or tree) is by comparing current vs target state.
    ///
    /// For leaf nodes with state (slider, input, dropdown, etc.), compares
    /// the current value against the target value.
    ///
    /// For click-only nodes (button, tab, etc.), always returns `NotStarted` —
    /// clicks are events, not state. The caller handles those via event handlers.
    ///
    /// For containers (Card, Form), aggregates children that are targets
    /// and returns Partial/Complete based on how many are done.
    pub fn check(&self) -> Completion {
        match self {
            // ── Click-only: no state to check ───────────────────
            UINode::Button(_)
            | UINode::Tab(_)
            | UINode::Accordion(_)
            | UINode::ModalButton(_)
            | UINode::DragSource(_)
            | UINode::DropZone(_) => Completion::NotStarted,

            // ── Toggle / Checkbox ───────────────────────────────
            UINode::Toggle(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                // Target is always to flip the toggle
                if state.is_on {
                    // If it's on now and we want it off (or vice versa),
                    // the task is to click it. We can't know if it's been
                    // clicked yet from state alone — caller handles this.
                    Completion::NotStarted
                } else {
                    Completion::NotStarted
                }
            }

            UINode::Checkbox(v, _state) => {
                if !v.is_target { return Completion::NotStarted; }
                // Same as toggle — click-driven
                Completion::NotStarted
            }

            UINode::Tag(v, _state) => {
                if !v.is_target { return Completion::NotStarted; }
                Completion::NotStarted
            }

            UINode::Toast(v, _state) => {
                if !v.is_target { return Completion::NotStarted; }
                Completion::NotStarted
            }

            UINode::Star(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                if state.current == state.target {
                    Completion::Complete
                } else {
                    Completion::NotStarted
                }
            }

            // ── Text input ──────────────────────────────────────
            UINode::TextInput(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                if state.current_value == state.target_value {
                    Completion::Complete
                } else if state.current_value.is_empty() {
                    Completion::NotStarted
                } else if state.target_value.starts_with(&state.current_value) {
                    // Partially typed the correct value
                    Completion::Partial {
                        done: state.current_value.len(),
                        total: state.target_value.len(),
                    }
                } else {
                    Completion::Wrong
                }
            }

            // ── Slider ──────────────────────────────────────────
            UINode::Slider(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                if state.current_val == state.target_val {
                    Completion::Complete
                } else {
                    let range = (state.max - state.min).max(1) as f32;
                    let distance = (state.current_val - state.target_val).abs() as f32;
                    let closeness = 1.0 - (distance / range);
                    // If they've moved the slider at all from its initial position,
                    // report partial progress based on how close they are
                    Completion::Partial {
                        done: (closeness * 100.0) as usize,
                        total: 100,
                    }
                }
            }

            // ── Dropdown ────────────────────────────────────────
            UINode::Dropdown(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                match &state.selected {
                    Some(sel) if sel == &state.target_option => Completion::Complete,
                    Some(_) => Completion::Wrong,
                    None => Completion::NotStarted,
                }
            }

            // ── Context menu ────────────────────────────────────
            UINode::ContextMenu(v, _state) => {
                if !v.is_target { return Completion::NotStarted; }
                // Event-driven, not state-checkable
                Completion::NotStarted
            }

            // ── Stepper ─────────────────────────────────────────
            UINode::Stepper(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                if state.current_val == state.target_val {
                    Completion::Complete
                } else {
                    let total_steps = ((state.target_val - state.current_val).abs() / state.step.max(1)) as usize;
                    let remaining = ((state.current_val - state.target_val).abs() / state.step.max(1)) as usize;
                    let done = total_steps.saturating_sub(remaining);
                    Completion::Partial { done, total: total_steps }
                }
            }

            // ── Radio group ─────────────────────────────────────
            UINode::RadioGroup(v, state) => {
                if !v.is_target { return Completion::NotStarted; }
                match state.selected {
                    Some(sel) if sel == state.target_option => Completion::Complete,
                    Some(_) => Completion::Wrong,
                    None => Completion::NotStarted,
                }
            }

            // ── Containers: aggregate children ──────────────────
            UINode::Card(_, children) | UINode::Form(_, _, children) => {
                let mut done = 0usize;
                let mut total = 0usize;
                let mut any_wrong = false;

                for child in children {
                    if !child.visual().is_target {
                        continue;
                    }
                    total += 1;
                    match child.check() {
                        Completion::Complete => done += 1,
                        Completion::Wrong => any_wrong = true,
                        _ => {}
                    }
                }

                if total == 0 {
                    Completion::NotStarted
                } else if done == total {
                    Completion::Complete
                } else if any_wrong {
                    Completion::Wrong
                } else if done > 0 {
                    Completion::Partial { done, total }
                } else {
                    Completion::NotStarted
                }
            }
        }
    }
}
