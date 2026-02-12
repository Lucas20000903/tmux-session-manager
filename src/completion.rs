//! Path completion utilities

use std::path::{Path, PathBuf};

/// Result of path completion operation
#[derive(Debug, Default)]
pub struct PathCompletion {
    /// All matching path suggestions
    pub suggestions: Vec<String>,
    /// Ghost text suffix to display after current input
    pub ghost_text: Option<String>,
}

/// Complete a partial path string
pub fn complete_path(partial: &str) -> PathCompletion {
    let partial = partial.trim();

    if partial.is_empty() {
        return complete_in_directory(Path::new("."), "", true);
    }

    let (expanded, uses_tilde) = expand_for_completion(partial);
    let expanded_path = Path::new(&expanded);

    if partial.ends_with('/') {
        if expanded_path.is_dir() {
            return complete_in_directory(expanded_path, "", uses_tilde);
        } else {
            return PathCompletion::default();
        }
    }

    let (dir, prefix) = if let Some(parent) = expanded_path.parent() {
        let filename = expanded_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        (parent.to_path_buf(), filename.to_string())
    } else {
        (PathBuf::from("."), expanded.clone())
    };

    if !dir.exists() {
        return PathCompletion::default();
    }

    complete_in_directory(&dir, &prefix, uses_tilde)
}

/// Complete entries in a directory matching a prefix
fn complete_in_directory(dir: &Path, prefix: &str, uses_tilde: bool) -> PathCompletion {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return PathCompletion::default(),
    };

    let prefix_lower = prefix.to_lowercase();
    let home_dir = dirs::home_dir();

    let mut matches: Vec<(String, bool)> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();

            if !prefix.is_empty() && !name_lower.starts_with(&prefix_lower) {
                return None;
            }

            if name.starts_with('.') && !prefix.starts_with('.') {
                return None;
            }

            let full_path = dir.join(&name);
            let is_dir = full_path.is_dir();

            let display_path = format_display_path(&full_path, uses_tilde, &home_dir, is_dir);

            Some((display_path, is_dir))
        })
        .collect();

    matches.sort_by(|a, b| match (a.1, b.1) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
    });

    let suggestions: Vec<String> = matches.into_iter().map(|(path, _)| path).collect();
    let ghost_text = calculate_ghost_text(prefix, &suggestions);

    PathCompletion {
        suggestions,
        ghost_text,
    }
}

/// Calculate ghost text suffix based on current input and suggestions
fn calculate_ghost_text(prefix: &str, suggestions: &[String]) -> Option<String> {
    if suggestions.is_empty() {
        return None;
    }

    let first = &suggestions[0];
    let first_lower = first.to_lowercase();
    let prefix_lower = prefix.to_lowercase();

    if let Some(last_sep) = first.rfind('/') {
        let filename = &first[last_sep + 1..];
        let filename_lower = filename.to_lowercase();

        if filename_lower.starts_with(&prefix_lower) {
            let suffix = &filename[prefix.len()..];
            if !suffix.is_empty() {
                return Some(suffix.to_string());
            }
        }
    } else if first_lower.starts_with(&prefix_lower) {
        let suffix = &first[prefix.len()..];
        if !suffix.is_empty() {
            return Some(suffix.to_string());
        }
    }

    None
}

/// Format a path for display, using ~ for home directory
fn format_display_path(
    path: &Path,
    uses_tilde: bool,
    home_dir: &Option<PathBuf>,
    is_dir: bool,
) -> String {
    let mut display = if uses_tilde {
        if let Some(home) = home_dir {
            if let Ok(stripped) = path.strip_prefix(home) {
                format!("~/{}", stripped.display())
            } else {
                path.display().to_string()
            }
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    };

    if is_dir && !display.ends_with('/') {
        display.push('/');
    }

    display
}

/// Expand ~ to home directory, returning (expanded_path, used_tilde)
fn expand_for_completion(path: &str) -> (String, bool) {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return (format!("{}/{}", home.display(), stripped), true);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return (home.display().to_string(), true);
        }
    }
    (path.to_string(), path.starts_with('~'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_for_completion() {
        let (expanded, uses_tilde) = expand_for_completion("~/test");
        assert!(uses_tilde);
        assert!(expanded.contains("test"));

        let (expanded, uses_tilde) = expand_for_completion("/absolute/path");
        assert!(!uses_tilde);
        assert_eq!(expanded, "/absolute/path");
    }
}
