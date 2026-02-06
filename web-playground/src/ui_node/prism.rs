//! Prism API — typed accessors and tree traversal for UINode.
//!
//! Idiomatic Rust "prisms": pattern-match accessors that return Option,
//! combined with a walk() iterator for composable tree queries.

use super::*;

impl UINode {
    /// Access the Visual properties shared by all variants.
    pub fn visual(&self) -> &Visual {
        match self {
            UINode::Button(v)
            | UINode::Toggle(v, _)
            | UINode::Checkbox(v, _)
            | UINode::Tab(v)
            | UINode::Accordion(v)
            | UINode::Tag(v, _)
            | UINode::Toast(v, _)
            | UINode::Star(v, _)
            | UINode::ModalButton(v)
            | UINode::TextInput(v, _)
            | UINode::Slider(v, _)
            | UINode::DragSource(v)
            | UINode::DropZone(v)
            | UINode::Dropdown(v, _)
            | UINode::ContextMenu(v, _)
            | UINode::Stepper(v, _)
            | UINode::RadioGroup(v, _)
            | UINode::Card(v, _)
            | UINode::Form(v, _, _) => v,
        }
    }

    /// Mutable access to the Visual properties.
    pub fn visual_mut(&mut self) -> &mut Visual {
        match self {
            UINode::Button(v)
            | UINode::Toggle(v, _)
            | UINode::Checkbox(v, _)
            | UINode::Tab(v)
            | UINode::Accordion(v)
            | UINode::Tag(v, _)
            | UINode::Toast(v, _)
            | UINode::Star(v, _)
            | UINode::ModalButton(v)
            | UINode::TextInput(v, _)
            | UINode::Slider(v, _)
            | UINode::DragSource(v)
            | UINode::DropZone(v)
            | UINode::Dropdown(v, _)
            | UINode::ContextMenu(v, _)
            | UINode::Stepper(v, _)
            | UINode::RadioGroup(v, _)
            | UINode::Card(v, _)
            | UINode::Form(v, _, _) => v,
        }
    }

    /// Children of container nodes. Returns empty slice for leaf nodes.
    pub fn children(&self) -> &[UINode] {
        match self {
            UINode::Card(_, children) | UINode::Form(_, _, children) => children,
            _ => &[],
        }
    }

    /// Pre-order depth-first traversal of the entire tree.
    pub fn walk(&self) -> WalkIter<'_> {
        WalkIter { stack: vec![self] }
    }

    // ── Typed prism accessors ───────────────────────────────────────

    pub fn as_button(&self) -> Option<&Visual> {
        match self { UINode::Button(v) => Some(v), _ => None }
    }

    pub fn as_toggle(&self) -> Option<(&Visual, &ToggleState)> {
        match self { UINode::Toggle(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_checkbox(&self) -> Option<(&Visual, &CheckState)> {
        match self { UINode::Checkbox(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_tab(&self) -> Option<&Visual> {
        match self { UINode::Tab(v) => Some(v), _ => None }
    }

    pub fn as_accordion(&self) -> Option<&Visual> {
        match self { UINode::Accordion(v) => Some(v), _ => None }
    }

    pub fn as_tag(&self) -> Option<(&Visual, &TagState)> {
        match self { UINode::Tag(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_toast(&self) -> Option<(&Visual, &ToastState)> {
        match self { UINode::Toast(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_star(&self) -> Option<(&Visual, &StarState)> {
        match self { UINode::Star(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_modal_button(&self) -> Option<&Visual> {
        match self { UINode::ModalButton(v) => Some(v), _ => None }
    }

    pub fn as_text_input(&self) -> Option<(&Visual, &InputState)> {
        match self { UINode::TextInput(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_slider(&self) -> Option<(&Visual, &SliderState)> {
        match self { UINode::Slider(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_drag_source(&self) -> Option<&Visual> {
        match self { UINode::DragSource(v) => Some(v), _ => None }
    }

    pub fn as_drop_zone(&self) -> Option<&Visual> {
        match self { UINode::DropZone(v) => Some(v), _ => None }
    }

    pub fn as_dropdown(&self) -> Option<(&Visual, &DropdownState)> {
        match self { UINode::Dropdown(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_context_menu(&self) -> Option<(&Visual, &ContextMenuState)> {
        match self { UINode::ContextMenu(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_stepper(&self) -> Option<(&Visual, &StepperState)> {
        match self { UINode::Stepper(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_radio_group(&self) -> Option<(&Visual, &RadioState)> {
        match self { UINode::RadioGroup(v, s) => Some((v, s)), _ => None }
    }

    pub fn as_card(&self) -> Option<(&Visual, &[UINode])> {
        match self { UINode::Card(v, c) => Some((v, c)), _ => None }
    }

    pub fn as_form(&self) -> Option<(&Visual, &FormState, &[UINode])> {
        match self { UINode::Form(v, f, c) => Some((v, f, c)), _ => None }
    }

    // ── Query helpers ───────────────────────────────────────────────

    /// Find all target nodes in the tree.
    pub fn targets(&self) -> Vec<&UINode> {
        self.walk().filter(|n| n.visual().is_target).collect()
    }
}

/// Pre-order DFS iterator over a UINode tree.
pub struct WalkIter<'a> {
    stack: Vec<&'a UINode>,
}

impl<'a> Iterator for WalkIter<'a> {
    type Item = &'a UINode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // Push children in reverse for left-to-right traversal
        for child in node.children().iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}
