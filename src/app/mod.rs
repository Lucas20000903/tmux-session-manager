//! Application state and business logic

mod helpers;
mod mode;

use anyhow::Result;

use crate::scroll_state::ScrollState;
use crate::session::Session;
use crate::tmux::Tmux;

pub use mode::{Mode, NewSessionField, SessionAction};

use helpers::expand_path;

/// Main application state
pub struct App {
    /// All discovered sessions
    pub sessions: Vec<Session>,
    /// Currently selected index
    pub selected: usize,
    /// Current UI mode
    pub mode: Mode,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Name of the currently attached session (if any)
    pub current_session: Option<String>,
    /// Filter text for filtering sessions
    pub filter: String,
    /// Error message to display (clears on next action)
    pub error: Option<String>,
    /// Success message to display (clears on next action)
    pub message: Option<String>,
    /// Cached preview content for the selected session's pane
    pub preview_content: Option<String>,
    /// Available actions for the selected session
    pub available_actions: Vec<SessionAction>,
    /// Currently highlighted action in ActionMenu mode
    pub selected_action: usize,
    /// Action pending confirmation
    pub pending_action: Option<SessionAction>,
    /// Scroll state for the session list
    pub scroll_state: ScrollState,
    /// Whether to show the preview pane
    pub show_preview: bool,
}

impl App {
    // =========================================================================
    // Initialization and core lifecycle
    // =========================================================================

    /// Create a new App instance
    pub fn new() -> Result<Self> {
        let sessions = Tmux::list_sessions()?;
        let current_session = Tmux::current_session()?;

        let mut app = Self {
            sessions,
            selected: 0,
            mode: Mode::Normal,
            should_quit: false,
            current_session,
            filter: String::new(),
            error: None,
            message: None,
            preview_content: None,
            available_actions: Vec::new(),
            selected_action: 0,
            pending_action: None,
            scroll_state: ScrollState::new(),
            show_preview: true,
        };

        app.update_preview();
        Ok(app)
    }

    /// Update the preview content for the currently selected session
    pub fn update_preview(&mut self) {
        const PREVIEW_LINES: usize = 15;

        let pane_id = self.selected_session().and_then(|session| {
            session
                .claude_code_pane
                .clone()
                .or_else(|| session.panes.first().map(|p| p.id.clone()))
        });

        self.preview_content = pane_id.and_then(|id| {
            Tmux::capture_pane(&id, PREVIEW_LINES, false).ok()
        });
    }

    /// Called every tick to refresh sessions and preview
    pub fn tick(&mut self) {
        if let Ok(sessions) = Tmux::list_sessions() {
            // Preserve selected session by name
            let selected_name = self.selected_session().map(|s| s.name.clone());
            self.sessions = sessions;

            // Restore selection by name
            if let Some(ref name) = selected_name {
                let filtered = self.filtered_sessions();
                if let Some(pos) = filtered.iter().position(|s| &s.name == name) {
                    self.selected = pos;
                }
            }

            if self.selected >= self.filtered_sessions().len() && !self.sessions.is_empty() {
                self.selected = self.filtered_sessions().len().saturating_sub(1);
            }
        }
        self.update_preview();
    }

    /// Clear any displayed messages
    pub fn clear_messages(&mut self) {
        self.error = None;
        self.message = None;
    }

    /// Refresh the session list
    pub fn refresh(&mut self) {
        self.clear_messages();
        if self.refresh_sessions() {
            self.message = Some("\u{f00c} Refreshed".to_string());
        }
    }

    /// Refresh sessions without affecting messages
    fn refresh_sessions(&mut self) -> bool {
        match Tmux::list_sessions() {
            Ok(sessions) => {
                self.sessions = sessions;
                if self.selected >= self.sessions.len() && !self.sessions.is_empty() {
                    self.selected = self.sessions.len() - 1;
                }
                self.update_preview();
                true
            }
            Err(e) => {
                self.error = Some(format!("\u{f00d} Failed to refresh: {}", e));
                false
            }
        }
    }

    // =========================================================================
    // Session selection and navigation
    // =========================================================================

    /// Get filtered sessions based on current filter, sorted by creation time
    pub fn filtered_sessions(&self) -> Vec<&Session> {
        let mut result: Vec<&Session> = if self.filter.is_empty() {
            self.sessions.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.sessions
                .iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&filter_lower)
                        || s.display_path().to_lowercase().contains(&filter_lower)
                })
                .collect()
        };
        result.sort_by_key(|s| s.created);
        result
    }

    /// Get filtered sessions grouped by working directory.
    /// Returns Vec of (group_path, Vec<(flat_index, &Session)>).
    pub fn grouped_sessions(&self) -> Vec<(String, Vec<(usize, &Session)>)> {
        let filtered = self.filtered_sessions();
        let mut groups: Vec<(String, Vec<(usize, &Session)>)> = Vec::new();

        for (i, session) in filtered.iter().enumerate() {
            let path = session.display_path();
            if let Some(group) = groups.iter_mut().find(|(p, _)| p == &path) {
                group.1.push((i, session));
            } else {
                groups.push((path, vec![(i, session)]));
            }
        }

        groups
    }

    /// Get the currently selected session
    pub fn selected_session(&self) -> Option<&Session> {
        let filtered = self.filtered_sessions();
        filtered.get(self.selected).copied()
    }

    /// Get the visual order of flat indices from grouped sessions
    fn visual_order(&self) -> Vec<usize> {
        self.grouped_sessions()
            .into_iter()
            .flat_map(|(_, sessions)| sessions.into_iter().map(|(i, _)| i))
            .collect()
    }

    /// Move selection up (in visual/grouped order)
    pub fn select_prev(&mut self) {
        let order = self.visual_order();
        if let Some(pos) = order.iter().position(|&i| i == self.selected) {
            if pos > 0 {
                self.selected = order[pos - 1];
                self.update_preview();
            }
        }
    }

    /// Move selection down (in visual/grouped order)
    pub fn select_next(&mut self) {
        let order = self.visual_order();
        if let Some(pos) = order.iter().position(|&i| i == self.selected) {
            if pos < order.len() - 1 {
                self.selected = order[pos + 1];
                self.update_preview();
            }
        }
    }

    /// Switch to the selected session and quit
    pub fn switch_to_selected(&mut self) {
        self.clear_messages();
        if let Some(session) = self.selected_session() {
            let name = session.name.clone();
            match Tmux::switch_to_session(&name) {
                Ok(_) => {
                    self.should_quit = true;
                }
                Err(e) => {
                    self.error = Some(format!("\u{f00d} Failed to switch: {}", e));
                }
            }
        }
    }

    /// Switch to the selected session but keep tsm open
    pub fn switch_to_selected_stay(&mut self) {
        self.clear_messages();
        if let Some(session) = self.selected_session() {
            let name = session.name.clone();
            match Tmux::switch_to_session(&name) {
                Ok(_) => {
                    self.message = Some(format!("\u{f00c} Switched to '{}'", name));
                }
                Err(e) => {
                    self.error = Some(format!("\u{f00d} Failed to switch: {}", e));
                }
            }
        }
    }

    // =========================================================================
    // Action menu
    // =========================================================================

    /// Enter the action menu for the selected session
    pub fn enter_action_menu(&mut self) {
        self.clear_messages();
        if self.selected_session().is_some() {
            self.compute_actions();
            self.mode = Mode::ActionMenu;
        }
    }

    /// Move to next action in the action menu
    pub fn select_next_action(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_action = (self.selected_action + 1) % self.available_actions.len();
        }
    }

    /// Move to previous action in the action menu
    pub fn select_prev_action(&mut self) {
        if !self.available_actions.is_empty() {
            if self.selected_action == 0 {
                self.selected_action = self.available_actions.len() - 1;
            } else {
                self.selected_action -= 1;
            }
        }
    }

    /// Execute the currently selected action from the action menu
    pub fn execute_selected_action(&mut self) {
        if let Some(action) = self.available_actions.get(self.selected_action).cloned() {
            if action.requires_confirmation() {
                self.pending_action = Some(action);
                self.mode = Mode::ConfirmAction;
            } else {
                self.execute_action(action);
            }
        }
    }

    /// Compute available actions for the selected session
    fn compute_actions(&mut self) {
        if self.selected_session().is_none() {
            self.available_actions = vec![];
            return;
        }

        let actions = vec![
            SessionAction::SwitchTo,
            SessionAction::Rename,
            SessionAction::Kill,
        ];

        self.available_actions = actions;
        self.selected_action = 0;
    }

    // =========================================================================
    // Action execution
    // =========================================================================

    /// Start the kill confirmation flow
    pub fn start_kill(&mut self) {
        self.clear_messages();
        if self.selected_session().is_some() {
            self.pending_action = Some(SessionAction::Kill);
            self.mode = Mode::ConfirmAction;
        }
    }

    /// Confirm and execute the pending action
    pub fn confirm_action(&mut self) {
        if let Some(action) = self.pending_action.take() {
            self.execute_action(action);
        }
        self.mode = Mode::Normal;
    }

    /// Execute an action on the selected session
    fn execute_action(&mut self, action: SessionAction) {
        let Some(session) = self.selected_session() else {
            self.mode = Mode::Normal;
            return;
        };
        let session_name = session.name.clone();

        match action {
            SessionAction::SwitchTo => {
                match Tmux::switch_to_session(&session_name) {
                    Ok(_) => self.should_quit = true,
                    Err(e) => self.error = Some(format!("\u{f00d} Failed to switch: {}", e)),
                }
                self.mode = Mode::Normal;
            }
            SessionAction::Rename => {
                self.mode = Mode::Rename {
                    old_name: session_name.clone(),
                    new_name: session_name,
                };
            }
            SessionAction::Kill => {
                match Tmux::kill_session(&session_name) {
                    Ok(_) => {
                        self.refresh_sessions();
                        self.message = Some(format!("\u{f00c} Killed session '{}'", session_name));
                    }
                    Err(e) => self.error = Some(format!("\u{f00d} Failed to kill: {}", e)),
                }
                self.mode = Mode::Normal;
            }
        }
    }

    // =========================================================================
    // Dialog flows: Rename
    // =========================================================================

    /// Start the rename flow
    pub fn start_rename(&mut self) {
        self.clear_messages();
        if let Some(session) = self.selected_session() {
            self.mode = Mode::Rename {
                old_name: session.name.clone(),
                new_name: session.name.clone(),
            };
        }
    }

    /// Confirm and execute session rename
    pub fn confirm_rename(&mut self) {
        if let Mode::Rename {
            ref old_name,
            ref new_name,
        } = self.mode
        {
            let old = old_name.clone();
            let new = new_name.clone();

            if old == new {
                self.mode = Mode::Normal;
                return;
            }

            match Tmux::rename_session(&old, &new) {
                Ok(_) => {
                    self.refresh_sessions();
                    self.message = Some(format!("\u{f00c} Renamed '{}' to '{}'", old, new));
                }
                Err(e) => {
                    self.error = Some(format!("\u{f00d} Failed to rename: {}", e));
                }
            }
        }
        self.mode = Mode::Normal;
    }

    // =========================================================================
    // Dialog flows: New Session
    // =========================================================================

    /// Generate a default session name like claude_A1B2C3D4 or shell_A1B2C3D4
    pub fn generate_session_name(start_claude: bool) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let prefix = if start_claude { "claude" } else { "shell" };
        format!("{}_{:08X}", prefix, (seed & 0xFFFFFFFF) as u32)
    }

    /// Start the new session flow, defaulting to the selected session's directory
    pub fn start_new_session(&mut self) {
        self.clear_messages();
        let default_path = self
            .selected_session()
            .map(|s| s.display_path())
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "~".to_string())
            });

        let completion = crate::completion::complete_path(&default_path);

        self.mode = Mode::NewSession {
            name: Self::generate_session_name(true),
            path: default_path,
            field: NewSessionField::StartWith,
            path_suggestions: completion.suggestions,
            path_selected: None,
            start_claude: true,
        };
    }

    /// Create the new session
    pub fn confirm_new_session(&mut self) {
        if let Mode::NewSession {
            ref name,
            ref path,
            start_claude,
            ..
        } = self.mode
        {
            if name.is_empty() {
                self.error = Some("\u{f00d} Session name cannot be empty".to_string());
                self.mode = Mode::Normal;
                return;
            }

            let session_name = name.clone();
            let session_path = expand_path(path);

            match Tmux::new_session(&session_name, &session_path, start_claude) {
                Ok(_) => {
                    self.refresh_sessions();
                    self.message = Some(format!("\u{f00c} Created session '{}'", session_name));
                }
                Err(e) => {
                    self.error = Some(format!("\u{f00d} Failed to create session: {}", e));
                }
            }
        }
        self.mode = Mode::Normal;
    }

    // =========================================================================
    // Filter mode
    // =========================================================================

    /// Start filter mode
    pub fn start_filter(&mut self) {
        self.clear_messages();
        self.mode = Mode::Filter {
            input: self.filter.clone(),
        };
    }

    /// Apply filter and return to normal mode
    pub fn apply_filter(&mut self) {
        if let Mode::Filter { ref input } = self.mode {
            self.filter = input.clone();
            self.selected = 0;
        }
        self.mode = Mode::Normal;
        self.update_preview();
    }

    /// Clear the filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.selected = 0;
    }

    /// Show help
    pub fn show_help(&mut self) {
        self.clear_messages();
        self.mode = Mode::Help;
    }

    /// Cancel current mode and return to normal
    pub fn cancel(&mut self) {
        self.pending_action = None;
        self.mode = Mode::Normal;
    }

    // =========================================================================
    // Status and statistics
    // =========================================================================

    /// Count sessions by status
    pub fn status_counts(&self) -> (usize, usize, usize) {
        use crate::session::ClaudeCodeStatus;

        let mut working = 0;
        let mut waiting = 0;
        let mut idle = 0;

        for session in &self.sessions {
            match session.claude_code_status {
                ClaudeCodeStatus::Working => working += 1,
                ClaudeCodeStatus::WaitingInput => waiting += 1,
                ClaudeCodeStatus::Idle => idle += 1,
                ClaudeCodeStatus::Unknown => {}
            }
        }

        (working, waiting, idle)
    }

    // =========================================================================
    // Path completion methods
    // =========================================================================

    /// Update path suggestions for NewSession mode
    pub fn update_new_session_path_suggestions(&mut self) {
        if let Mode::NewSession {
            ref path,
            ref mut path_suggestions,
            ref mut path_selected,
            ..
        } = self.mode
        {
            let completion = crate::completion::complete_path(path);
            *path_suggestions = completion.suggestions;
            if let Some(idx) = *path_selected {
                if idx >= path_suggestions.len() {
                    *path_selected = if path_suggestions.is_empty() {
                        None
                    } else {
                        Some(path_suggestions.len() - 1)
                    };
                }
            }
        }
    }

    /// Select previous path suggestion in NewSession mode
    pub fn select_prev_new_session_path(&mut self) {
        if let Mode::NewSession {
            ref path_suggestions,
            ref mut path_selected,
            ..
        } = self.mode
        {
            if path_suggestions.is_empty() {
                return;
            }
            *path_selected = Some(
                path_selected
                    .map(|i| {
                        if i == 0 {
                            path_suggestions.len() - 1
                        } else {
                            i - 1
                        }
                    })
                    .unwrap_or(path_suggestions.len() - 1),
            );
        }
    }

    /// Select next path suggestion in NewSession mode
    pub fn select_next_new_session_path(&mut self) {
        if let Mode::NewSession {
            ref path_suggestions,
            ref mut path_selected,
            ..
        } = self.mode
        {
            if path_suggestions.is_empty() {
                return;
            }
            *path_selected = Some(
                path_selected
                    .map(|i| (i + 1) % path_suggestions.len())
                    .unwrap_or(0),
            );
        }
    }

    /// Accept the current path completion in NewSession mode
    pub fn accept_new_session_path_completion(&mut self) {
        if let Mode::NewSession {
            ref mut path,
            ref path_suggestions,
            ref mut path_selected,
            ..
        } = self.mode
        {
            if let Some(idx) = *path_selected {
                if let Some(suggestion) = path_suggestions.get(idx) {
                    *path = suggestion.clone();
                    *path_selected = None;
                }
            } else if let Some(first) = path_suggestions.first() {
                *path = first.clone();
            }
        }
        self.update_new_session_path_suggestions();
    }

    // =========================================================================
    // Scroll/list computation
    // =========================================================================

    /// Count the number of group headers shown before a given session index.
    fn group_headers_before(&self, session_index: usize) -> usize {
        let groups = self.grouped_sessions();
        let mut headers = 0;
        for (_, sessions) in &groups {
            if let Some(&(first_idx, _)) = sessions.first() {
                if first_idx > session_index {
                    break;
                }
                headers += 1;
            }
        }
        headers
    }

    /// Total number of group headers in the list
    fn group_header_count(&self) -> usize {
        self.grouped_sessions().len()
    }

    /// Compute the flat list index for the current selection.
    pub fn compute_flat_list_index(&self) -> usize {
        let filtered_count = self.filtered_sessions().len();
        if filtered_count == 0 {
            return 0;
        }

        let group_offset = self.group_headers_before(self.selected);

        match self.mode {
            Mode::ActionMenu => {
                let mut index = self.selected + group_offset;

                index += 1; // selected session row itself
                index += 1; // metadata row
                index += 1; // separator
                index += self.selected_action;

                index
            }
            _ => self.selected + group_offset,
        }
    }

    /// Compute the total number of items in the rendered list.
    pub fn compute_total_list_items(&self) -> usize {
        let filtered_count = self.filtered_sessions().len();
        if filtered_count == 0 {
            return 0;
        }

        let group_headers = self.group_header_count();

        match self.mode {
            Mode::ActionMenu => {
                let mut total = filtered_count + group_headers;

                total += 1; // metadata row
                total += 1; // separator
                total += self.available_actions.len();
                total += 1; // end separator

                total
            }
            _ => filtered_count + group_headers,
        }
    }
}
