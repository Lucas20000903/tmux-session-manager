//! Scroll state management with center-locked selection behavior.

use ratatui::widgets::ListState;

/// Manages scroll state for a list with center-locked scrolling.
pub struct ScrollState {
    /// The underlying ratatui ListState
    list_state: ListState,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default(),
        }
    }

    /// Update the scroll state given the current selection and list dimensions.
    pub fn update(
        &mut self,
        selected: usize,
        total_items: usize,
        visible_height: usize,
    ) -> &mut ListState {
        self.list_state.select(Some(selected));

        let offset = Self::compute_centered_offset(selected, total_items, visible_height);
        *self.list_state.offset_mut() = offset;

        &mut self.list_state
    }

    /// Compute the scroll offset to keep selection centered.
    fn compute_centered_offset(
        selected: usize,
        total_items: usize,
        visible_height: usize,
    ) -> usize {
        if visible_height == 0 || total_items == 0 {
            return 0;
        }

        let middle = visible_height / 2;

        if selected <= middle {
            return 0;
        }

        let ideal_offset = selected.saturating_sub(middle);
        let max_offset = total_items.saturating_sub(visible_height);

        ideal_offset.min(max_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_scroll_when_at_top() {
        assert_eq!(ScrollState::compute_centered_offset(0, 20, 10), 0);
        assert_eq!(ScrollState::compute_centered_offset(3, 20, 10), 0);
        assert_eq!(ScrollState::compute_centered_offset(5, 20, 10), 0);
    }

    #[test]
    fn test_scroll_to_center_selection() {
        assert_eq!(ScrollState::compute_centered_offset(7, 20, 10), 2);
        assert_eq!(ScrollState::compute_centered_offset(10, 20, 10), 5);
    }

    #[test]
    fn test_max_scroll_at_bottom() {
        assert_eq!(ScrollState::compute_centered_offset(18, 20, 10), 10);
        assert_eq!(ScrollState::compute_centered_offset(19, 20, 10), 10);
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(ScrollState::compute_centered_offset(0, 0, 10), 0);
        assert_eq!(ScrollState::compute_centered_offset(5, 20, 0), 0);
        assert_eq!(ScrollState::compute_centered_offset(3, 5, 10), 0);
    }
}
