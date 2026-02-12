use crate::session::ClaudeCodeStatus;

/// Strip ANSI escape sequences from text
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip CSI sequences: ESC [ ... final_byte
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() || next == 'm' {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn detect_status(content: &str) -> ClaudeCodeStatus {
    let content = &strip_ansi(content);

    if content.contains("Enter to select")
        || content.contains("↑/↓ to navigate")
        || content.contains("Esc to cancel")
        || content.contains("to edit")
    {
        return ClaudeCodeStatus::WaitingInput;
    }
    if has_input_field(content) {
        if content.contains("to interrupt") {
            return ClaudeCodeStatus::Working;
        }
        return ClaudeCodeStatus::Idle;
    }

    ClaudeCodeStatus::Unknown
}

/// Detect input field: prompt line (❯) with border directly above it.
fn has_input_field(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains('❯') {
            // Check if line above is a border
            if i > 0 && lines[i - 1].contains('─') {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working() {
        let content = "* (ctrl+c to interrupt)\n─────\n❯ hello";
        assert_eq!(detect_status(content), ClaudeCodeStatus::Working);
    }

    #[test]
    fn test_idle() {
        let content = "● Done\n─────\n❯ hello";
        assert_eq!(detect_status(content), ClaudeCodeStatus::Idle);
    }

    #[test]
    fn test_no_border_above_prompt() {
        let content = "─────\nsome text\n❯ hello";
        assert_eq!(detect_status(content), ClaudeCodeStatus::Unknown);
    }

    #[test]
    fn test_waiting_input_enter_to_select() {
        let content = "Enter to select";
        assert_eq!(detect_status(content), ClaudeCodeStatus::WaitingInput);
    }

    #[test]
    fn test_waiting_input_navigate() {
        let content = "↑/↓ to navigate";
        assert_eq!(detect_status(content), ClaudeCodeStatus::WaitingInput);
    }

    #[test]
    fn test_waiting_input_permission_prompt() {
        let content = "Do you want to proceed?\n  1. Yes\n❯ 2. No\n\nEsc to cancel";
        assert_eq!(detect_status(content), ClaudeCodeStatus::WaitingInput);
    }

    #[test]
    fn test_waiting_input_with_ansi() {
        let content = "\x1b[38;2;153;153;153mEsc\x1b[39m \x1b[38;2;153;153;153mto\x1b[39m \x1b[38;2;153;153;153mcancel\x1b[39m";
        assert_eq!(detect_status(content), ClaudeCodeStatus::WaitingInput);
    }

    #[test]
    fn test_working_with_ansi() {
        let content = "\x1b[38;2;153;153;153mesc to interrupt\x1b[39m\n\x1b[38;2;177;185;249m─────\x1b[39m\n\x1b[38;2;177;185;249m❯\x1b[39m hello";
        assert_eq!(detect_status(content), ClaudeCodeStatus::Working);
    }

    #[test]
    fn test_waiting_input_edit_plan() {
        let content = "ctrl+e to edit plan.md";
        assert_eq!(detect_status(content), ClaudeCodeStatus::WaitingInput);
    }

    #[test]
    fn test_unknown() {
        let content = "random stuff";
        assert_eq!(detect_status(content), ClaudeCodeStatus::Unknown);
    }
}
