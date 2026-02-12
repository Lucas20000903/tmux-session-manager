//! Application mode and action types

/// The current mode/state of the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Normal browsing mode
    Normal,
    /// Viewing actions for selected session
    ActionMenu,
    /// Filtering sessions with search input
    Filter { input: String },
    /// Confirming an action (kill, etc.)
    ConfirmAction,
    /// Creating a new session
    NewSession {
        name: String,
        path: String,
        field: NewSessionField,
        /// Path completion suggestions
        path_suggestions: Vec<String>,
        /// Currently selected path suggestion index
        path_selected: Option<usize>,
    },
    /// Renaming a session
    Rename { old_name: String, new_name: String },
    /// Showing help
    Help,
}

/// An action that can be performed on a session
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionAction {
    /// Switch to this session
    SwitchTo,
    /// Rename this session
    Rename,
    /// Kill this session
    Kill,
}

impl SessionAction {
    /// Returns the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            Self::SwitchTo => "Switch to session",
            Self::Rename => "Rename session",
            Self::Kill => "Kill session",
        }
    }

    /// Whether this action requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Kill)
    }
}

/// Which field is active in the new session dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewSessionField {
    Name,
    Path,
}
