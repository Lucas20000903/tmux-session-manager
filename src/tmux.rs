use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

use crate::detection::detect_status;
use crate::session::{Pane, Session};

/// Wrapper for tmux command execution
pub struct Tmux;

impl Tmux {
    /// List all tmux sessions with their metadata
    pub fn list_sessions() -> Result<Vec<Session>> {
        let output = Command::new("tmux")
            .args([
                "list-sessions",
                "-F",
                "#{session_name}\t#{session_created}\t#{session_attached}\t#{session_windows}",
            ])
            .output()
            .context("Failed to execute tmux list-sessions")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // No sessions is not an error for us
            if stderr.contains("no server running") || stderr.contains("no sessions") {
                return Ok(Vec::new());
            }
            anyhow::bail!("tmux list-sessions failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut sessions = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let created = parts[1].parse().unwrap_or(0);
                let attached = parts[2] == "1";
                let window_count = parts[3].parse().unwrap_or(1);

                // Get panes for this session
                let panes = Self::list_panes(&name).unwrap_or_default();

                // Find Claude Code pane and detect status
                let (claude_code_pane, claude_code_status, working_directory, pane_title) =
                    Self::find_claude_code_pane(&name, &panes);

                // Use the Claude Code pane's path, or fall back to first pane's path
                let working_directory = working_directory.unwrap_or_else(|| {
                    panes
                        .first()
                        .map(|p| p.current_path.clone())
                        .unwrap_or_default()
                });

                sessions.push(Session {
                    name,
                    created,
                    attached,
                    working_directory,
                    window_count,
                    panes,
                    claude_code_pane,
                    claude_code_status,
                    pane_title,
                });
            }
        }

        // Sort by attached status first, then by name
        sessions.sort_by(|a, b| {
            b.attached
                .cmp(&a.attached)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(sessions)
    }

    /// List all panes in a session
    fn list_panes(session: &str) -> Result<Vec<Pane>> {
        let output = Command::new("tmux")
            .args([
                "list-panes",
                "-t",
                session,
                "-F",
                "#{pane_id}\t#{pane_current_command}\t#{pane_current_path}\t#{pane_pid}\t#{pane_title}",
            ])
            .output()
            .context("Failed to execute tmux list-panes")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut panes = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 5 {
                panes.push(Pane {
                    id: parts[0].to_string(),
                    current_command: parts[1].to_string(),
                    current_path: PathBuf::from(parts[2]),
                    pid: parts[3].parse().unwrap_or(0),
                    title: parts[4].to_string(),
                });
            }
        }

        Ok(panes)
    }

    /// Check if a process or its parent is claude
    fn is_claude_process(pane: &Pane) -> bool {
        // Check pane_current_command directly
        let cmd = &pane.current_command;
        if cmd == "claude" || cmd.contains("claude") {
            return true;
        }

        // Check actual process name via pid (pane_current_command can show version number)
        if pane.pid > 0 {
            if let Ok(output) = Command::new("ps")
                .args(["-o", "command=", "-p", &pane.pid.to_string()])
                .output()
            {
                let ps_cmd = String::from_utf8_lossy(&output.stdout);
                if ps_cmd.trim() == "claude" || ps_cmd.contains("claude") {
                    return true;
                }
            }
        }

        false
    }

    /// Find the pane running Claude Code and detect its status
    /// Returns (pane_id, status, working_directory, pane_title)
    fn find_claude_code_pane(
        session_name: &str,
        panes: &[Pane],
    ) -> (
        Option<String>,
        crate::session::ClaudeCodeStatus,
        Option<PathBuf>,
        String,
    ) {
        use crate::session::ClaudeCodeStatus;

        // First: check each pane's process
        for pane in panes {
            if Self::is_claude_process(pane) {
                let status = Self::capture_pane(&pane.id, 4, true)
                    .map(|content| detect_status(&content))
                    .unwrap_or(ClaudeCodeStatus::Unknown);

                return (
                    Some(pane.id.clone()),
                    status,
                    Some(pane.current_path.clone()),
                    pane.title.clone(),
                );
            }
        }

        // Fallback: session name starts with "claude" (e.g. claude_284B7F7A)
        if session_name.starts_with("claude") {
            if let Some(pane) = panes.first() {
                let status = Self::capture_pane(&pane.id, 4, true)
                    .map(|content| detect_status(&content))
                    .unwrap_or(ClaudeCodeStatus::Unknown);

                return (
                    Some(pane.id.clone()),
                    status,
                    Some(pane.current_path.clone()),
                    pane.title.clone(),
                );
            }
        }

        let title = panes.first().map(|p| p.title.clone()).unwrap_or_default();
        (None, ClaudeCodeStatus::Unknown, None, title)
    }

    /// Capture the last N lines of a pane's content
    pub fn capture_pane(pane_id: &str, lines: usize, strip_empty: bool) -> Result<String> {
        let output = Command::new("tmux")
            .args([
                "capture-pane",
                "-t",
                pane_id,
                "-p", // Print to stdout
                "-J", // Join wrapped lines
                "-e", // Include escape sequences
            ])
            .output()
            .context("Failed to capture pane")?;

        if !output.status.success() {
            anyhow::bail!("Failed to capture pane {}", pane_id);
        }

        let content = String::from_utf8_lossy(&output.stdout);

        if strip_empty {
            // Filter out empty lines, then get last N (for status detection)
            let non_empty: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            let start = non_empty.len().saturating_sub(lines);
            let last_lines = &non_empty[start..];
            Ok(last_lines.join("\n"))
        } else {
            // Preserve internal empty lines but trim trailing ones (for preview display)
            let all_lines: Vec<&str> = content.lines().collect();

            // Find last non-empty line
            let last_non_empty = all_lines
                .iter()
                .rposition(|l| !l.trim().is_empty())
                .map(|i| i + 1)
                .unwrap_or(0);

            let trimmed = &all_lines[..last_non_empty];
            let start = trimmed.len().saturating_sub(lines);
            let last_lines = &trimmed[start..];
            Ok(last_lines.join("\n"))
        }
    }

    /// Switch to the specified session.
    /// Uses switch-client inside tmux, attach-session outside.
    pub fn switch_to_session(session: &str) -> Result<()> {
        if Self::is_inside_tmux() {
            let status = Command::new("tmux")
                .args(["switch-client", "-t", session])
                .status()
                .context("Failed to switch session")?;

            if !status.success() {
                anyhow::bail!("Failed to switch to session {}", session);
            }
        } else {
            let status = Command::new("tmux")
                .args(["attach-session", "-t", session])
                .status()
                .context("Failed to attach session")?;

            if !status.success() {
                anyhow::bail!("Failed to attach to session {}", session);
            }
        }

        Ok(())
    }

    /// Check if we're running inside a tmux session
    fn is_inside_tmux() -> bool {
        std::env::var("TMUX").is_ok()
    }

    /// Create a new tmux session
    pub fn new_session(name: &str, path: &std::path::Path, start_claude: bool) -> Result<()> {
        let path_str = path.to_string_lossy();

        let status = Command::new("tmux")
            .args(["new-session", "-d", "-s", name, "-c", &path_str])
            .status()
            .context("Failed to create new session")?;

        if !status.success() {
            anyhow::bail!("Failed to create session {}", name);
        }

        if start_claude {
            // Send claude command to the new session
            let _ = Command::new("tmux")
                .args(["send-keys", "-t", name, "claude", "Enter"])
                .status();
        }

        Ok(())
    }

    /// Kill a tmux session
    pub fn kill_session(session: &str) -> Result<()> {
        let status = Command::new("tmux")
            .args(["kill-session", "-t", session])
            .status()
            .context("Failed to kill session")?;

        if !status.success() {
            anyhow::bail!("Failed to kill session {}", session);
        }

        Ok(())
    }

    /// Rename a tmux session
    pub fn rename_session(old_name: &str, new_name: &str) -> Result<()> {
        let status = Command::new("tmux")
            .args(["rename-session", "-t", old_name, new_name])
            .status()
            .context("Failed to rename session")?;

        if !status.success() {
            anyhow::bail!("Failed to rename session {} to {}", old_name, new_name);
        }

        Ok(())
    }

    /// Get the name of the currently attached session
    pub fn current_session() -> Result<Option<String>> {
        let output = Command::new("tmux")
            .args(["display-message", "-p", "#{session_name}"])
            .output()
            .context("Failed to get current session")?;

        if !output.status.success() {
            return Ok(None);
        }

        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            Ok(None)
        } else {
            Ok(Some(name))
        }
    }
}
