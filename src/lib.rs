//! SigmaOS Clipboard — cross-process clipboard management.
//!
//! Principles: Single Responsibility, Encapsulation, Information Hiding,
//! Command-Query Separation, Defensive Programming, Fail-Fast.

use std::collections::VecDeque;
use std::fmt;

const MAX_HISTORY: usize = 50;

/// Supported clipboard content types.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardContent {
    Text(String),
    Image { width: u32, height: u32, pixels: Vec<u8> },
    Empty,
}

impl fmt::Display for ClipboardContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardContent::Text(t) => write!(f, "Text({} chars)", t.len()),
            ClipboardContent::Image { width, height, .. } => {
                write!(f, "Image({width}x{height})")
            }
            ClipboardContent::Empty => write!(f, "Empty"),
        }
    }
}

#[derive(Debug)]
pub enum ClipboardError {
    HistoryEmpty,
    IndexOutOfRange(usize),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::HistoryEmpty => write!(f, "clipboard history is empty"),
            ClipboardError::IndexOutOfRange(i) => write!(f, "index {i} out of range"),
        }
    }
}

/// Clipboard manager with ring-buffer history.
/// Composition over inheritance: uses VecDeque internally.
#[derive(Debug)]
pub struct ClipboardManager {
    current: ClipboardContent,
    history: VecDeque<ClipboardContent>,
    max_history: usize,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            current: ClipboardContent::Empty,
            history: VecDeque::new(),
            max_history: MAX_HISTORY,
        }
    }

    pub fn with_max_history(max_history: usize) -> Self {
        assert!(max_history > 0, "max_history must be positive");
        Self {
            current: ClipboardContent::Empty,
            history: VecDeque::new(),
            max_history,
        }
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    /// Get current clipboard content.
    pub fn current(&self) -> &ClipboardContent {
        &self.current
    }

    /// Get history slice (most recent last).
    pub fn history(&self) -> impl Iterator<Item = &ClipboardContent> {
        self.history.iter()
    }

    /// Number of items in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Peek at a specific history entry.
    pub fn history_at(&self, index: usize) -> Result<&ClipboardContent, ClipboardError> {
        self.history.get(index).ok_or(ClipboardError::IndexOutOfRange(index))
    }

    // ── Commands ─────────────────────────────────────────────────────────────

    /// Copy text to clipboard.
    pub fn copy_text(&mut self, text: impl Into<String>) {
        let new = ClipboardContent::Text(text.into());
        self.push(new);
    }

    /// Copy image data to clipboard.
    pub fn copy_image(&mut self, width: u32, height: u32, pixels: Vec<u8>) {
        let new = ClipboardContent::Image { width, height, pixels };
        self.push(new);
    }

    /// Clear the current clipboard content.
    pub fn clear(&mut self) {
        self.push(ClipboardContent::Empty);
    }

    /// Restore from history at given index.
    pub fn restore(&mut self, index: usize) -> Result<(), ClipboardError> {
        let item = self.history.get(index)
            .ok_or(ClipboardError::IndexOutOfRange(index))?
            .clone();
        self.current = item;
        Ok(())
    }

    /// Clear all history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    // ── Private ──────────────────────────────────────────────────────────────

    fn push(&mut self, content: ClipboardContent) {
        let old = std::mem::replace(&mut self.current, content);
        if !matches!(old, ClipboardContent::Empty) {
            if self.history.len() >= self.max_history {
                self.history.pop_front();
            }
            self.history.push_back(old);
        }
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_text_updates_current() {
        // Arrange
        let mut cm = ClipboardManager::new();
        // Act
        cm.copy_text("hello world");
        // Assert
        assert_eq!(cm.current(), &ClipboardContent::Text("hello world".to_string()));
    }

    #[test]
    fn test_history_grows_on_successive_copies() {
        // Arrange
        let mut cm = ClipboardManager::new();
        // Act
        cm.copy_text("first");
        cm.copy_text("second");
        cm.copy_text("third");
        // Assert
        assert_eq!(cm.history_len(), 2); // first and second pushed to history
    }

    #[test]
    fn test_history_bounded_by_max() {
        // Arrange
        let mut cm = ClipboardManager::with_max_history(3);
        // Act
        for i in 0..10 {
            cm.copy_text(format!("item {i}"));
        }
        // Assert
        assert!(cm.history_len() <= 3);
    }

    #[test]
    fn test_restore_from_history() {
        // Arrange
        let mut cm = ClipboardManager::new();
        cm.copy_text("original");
        cm.copy_text("new");
        // Act
        let result = cm.restore(0);
        // Assert
        assert!(result.is_ok());
        assert_eq!(cm.current(), &ClipboardContent::Text("original".to_string()));
    }

    #[test]
    fn test_restore_invalid_index_returns_error() {
        // Arrange
        let mut cm = ClipboardManager::new();
        // Act
        let result = cm.restore(99);
        // Assert
        assert!(matches!(result, Err(ClipboardError::IndexOutOfRange(99))));
    }

    #[test]
    fn test_clear_sets_empty() {
        // Arrange
        let mut cm = ClipboardManager::new();
        cm.copy_text("something");
        // Act
        cm.clear();
        // Assert
        assert_eq!(cm.current(), &ClipboardContent::Empty);
    }
}
