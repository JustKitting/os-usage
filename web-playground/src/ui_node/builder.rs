//! Builder API — convenience constructors for UINode.
//!
//! These reduce boilerplate when levels construct their UI trees.

use super::*;

/// Simple button (not a target).
pub fn button(label: impl Into<String>, rect: Rect) -> UINode {
    UINode::Button(Visual::new(label, rect))
}

/// Button that the solver should click.
pub fn target_button(label: impl Into<String>, rect: Rect) -> UINode {
    UINode::Button(Visual::new(label, rect).target())
}

/// Toggle switch.
pub fn toggle(label: impl Into<String>, rect: Rect, is_on: bool) -> UINode {
    UINode::Toggle(Visual::new(label, rect).target(), ToggleState { is_on })
}

/// Checkbox.
pub fn checkbox(label: impl Into<String>, rect: Rect, is_checked: bool) -> UINode {
    UINode::Checkbox(Visual::new(label, rect).target(), CheckState { is_checked })
}

/// Tab header.
pub fn tab(label: impl Into<String>, rect: Rect) -> UINode {
    UINode::Tab(Visual::new(label, rect).target())
}

/// Accordion / collapsible section header.
pub fn accordion(label: impl Into<String>, rect: Rect) -> UINode {
    UINode::Accordion(Visual::new(label, rect).target())
}

/// Selectable tag chip.
pub fn tag(label: impl Into<String>, rect: Rect, is_selected: bool) -> UINode {
    UINode::Tag(Visual::new(label, rect).target(), TagState { is_selected })
}

/// Toast notification.
pub fn toast(label: impl Into<String>, rect: Rect, kind: impl Into<String>, message: impl Into<String>) -> UINode {
    UINode::Toast(
        Visual::new(label, rect).target(),
        ToastState { kind: kind.into(), message: message.into() },
    )
}

/// Star rating control.
pub fn star_rating(label: impl Into<String>, rect: Rect, current: usize, target: usize, max: usize) -> UINode {
    UINode::Star(
        Visual::new(label, rect).target(),
        StarState { current, target, max },
    )
}

/// Text input field (target).
pub fn text_input(
    label: impl Into<String>,
    rect: Rect,
    placeholder: impl Into<String>,
    target_value: impl Into<String>,
) -> UINode {
    UINode::TextInput(
        Visual::new(label, rect).target(),
        InputState {
            placeholder: placeholder.into(),
            current_value: String::new(),
            target_value: target_value.into(),
        },
    )
}

/// Slider with drag interaction (target).
pub fn slider(
    label: impl Into<String>,
    rect: Rect,
    min: i32,
    max: i32,
    step: i32,
    current: i32,
    target: i32,
    thumb_rect: Rect,
    target_thumb_rect: Rect,
) -> UINode {
    UINode::Slider(
        Visual::new(label, rect).target(),
        SliderState {
            min,
            max,
            step,
            current_val: current,
            target_val: target,
            thumb_rect,
            target_thumb_rect,
        },
    )
}

/// Draggable element.
pub fn drag_source(label: impl Into<String>, rect: Rect) -> UINode {
    UINode::DragSource(Visual::new(label, rect).target())
}

/// Drop zone.
pub fn drop_zone(label: impl Into<String>, rect: Rect) -> UINode {
    UINode::DropZone(Visual::new(label, rect))
}

/// Dropdown select (target).
pub fn dropdown(
    label: impl Into<String>,
    rect: Rect,
    options: Vec<String>,
    target_option: impl Into<String>,
) -> UINode {
    UINode::Dropdown(
        Visual::new(label, rect).target(),
        DropdownState {
            options,
            selected: None,
            target_option: target_option.into(),
            trigger_label: "Choose...".into(),
        },
    )
}

/// Dropdown with custom trigger label.
pub fn dropdown_with_trigger(
    label: impl Into<String>,
    rect: Rect,
    options: Vec<String>,
    target_option: impl Into<String>,
    trigger_label: impl Into<String>,
) -> UINode {
    UINode::Dropdown(
        Visual::new(label, rect).target(),
        DropdownState {
            options,
            selected: None,
            target_option: target_option.into(),
            trigger_label: trigger_label.into(),
        },
    )
}

/// Context menu (right-click trigger).
pub fn context_menu(
    rect: Rect,
    trigger_label: impl Into<String>,
    items: Vec<String>,
    target_item: impl Into<String>,
) -> UINode {
    let tl = trigger_label.into();
    UINode::ContextMenu(
        Visual::new(&tl, rect).target(),
        ContextMenuState {
            items,
            target_item: target_item.into(),
            trigger_label: tl,
        },
    )
}

/// Stepper (+/- buttons).
pub fn stepper(
    label: impl Into<String>,
    rect: Rect,
    min: i32,
    max: i32,
    step: i32,
    current: i32,
    target: i32,
) -> UINode {
    let l = label.into();
    UINode::Stepper(
        Visual::new(&l, rect).target(),
        StepperState {
            min,
            max,
            step,
            current_val: current,
            target_val: target,
            minus_label: format!("minus: {}", l),
            plus_label: format!("+: {}", l),
        },
    )
}

/// Radio button group.
pub fn radio_group(
    label: impl Into<String>,
    rect: Rect,
    options: Vec<String>,
    target_option: usize,
) -> UINode {
    UINode::RadioGroup(
        Visual::new(label, rect).target(),
        RadioState {
            options,
            selected: None,
            target_option,
        },
    )
}

/// Card container (no submit button).
pub fn card(rect: Rect, children: Vec<UINode>) -> UINode {
    UINode::Card(Visual::new("card", rect), children)
}

/// Form container (appends submit click after children).
pub fn form(rect: Rect, submit_label: impl Into<String>, children: Vec<UINode>) -> UINode {
    UINode::Form(
        Visual::new("form", rect),
        FormState {
            submit_label: submit_label.into(),
            cancel_label: None,
        },
        children,
    )
}
