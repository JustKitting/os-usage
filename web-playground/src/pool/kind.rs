//! ElementKind - the categories of UI elements

use std::fmt;

/// The closed set of element categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Button,
    Input,
    Dropdown,
    Checkbox,
    Toggle,
    Link,
}

impl ElementKind {
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Input => "text input",
            Self::Dropdown => "dropdown",
            Self::Checkbox => "checkbox",
            Self::Toggle => "toggle switch",
            Self::Link => "link",
        }
    }
}

impl fmt::Display for ElementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}
