//! ElementKind - the categories of UI elements

use std::fmt;

/// The closed set of element categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Button,
    Input,
    Dropdown,
    Checkbox,
    Radio,
    Toggle,
    Link,
    Text,
}

impl ElementKind {
    pub const ALL: &[Self] = &[
        Self::Button,
        Self::Input,
        Self::Dropdown,
        Self::Checkbox,
        Self::Radio,
        Self::Toggle,
        Self::Link,
        Self::Text,
    ];

    pub fn describe(&self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Input => "text input",
            Self::Dropdown => "dropdown",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio button",
            Self::Toggle => "toggle switch",
            Self::Link => "link",
            Self::Text => "text",
        }
    }

    /// What interaction the model should perform
    pub fn default_action(&self) -> &'static str {
        match self {
            Self::Button => "click",
            Self::Input => "type into",
            Self::Dropdown => "select from",
            Self::Checkbox => "check",
            Self::Radio => "select",
            Self::Toggle => "toggle",
            Self::Link => "click",
            Self::Text => "read",
        }
    }
}

impl fmt::Display for ElementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}
