use std::ops::Range;
use regex::Regex;
use ropey::{Rope, RopeSlice};

use crate::history::{EditOp, History, Transaction};
use crate::metrics::{LspPosition, Metrics};

/// Represents an in-memory high-performance text document backed by a B-tree Rope.
#[derive(Debug)]
pub struct TextBuffer {
    rope: Rope,
    history: History,
    dirty: bool,
    version: u64,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Create a new empty text buffer.
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            history: History::new(1000),
            dirty: false,
            version: 0,
        }
    }

    /// Create a text buffer from string content.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            history: History::new(1000),
            dirty: false,
            version: 0,
        }
    }

    /// Returns the length of the buffer in unicode scalar values (characters).
    #[inline]
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Returns the length of the buffer in UTF-8 bytes.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Returns the number of lines in the buffer.
    #[inline]
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Check if buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    /// Has unsaved modifications.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark buffer as saved (not dirty).
    #[inline]
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    /// Document version (increments on every edit).
    #[inline]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get raw reference to inner Rope.
    #[inline]
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Slices the buffer across character range.
    pub fn slice(&self, range: Range<usize>) -> RopeSlice<'_> {
        let start = range.start.min(self.len_chars());
        let end = range.end.min(self.len_chars());
        self.rope.slice(start..end)
    }

    /// Get a specific line as a RopeSlice.
    pub fn line(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        if line_idx < self.len_lines() {
            Some(self.rope.line(line_idx))
        } else {
            None
        }
    }

    /// Insert text at a character index without recording to history directly.
    pub fn insert_raw(&mut self, char_idx: usize, text: &str) {
        let char_idx = char_idx.min(self.len_chars());
        self.rope.insert(char_idx, text);
        self.dirty = true;
        self.version += 1;
    }

    /// Remove a character range without recording to history directly.
    pub fn remove_raw(&mut self, range: Range<usize>) -> String {
        let start = range.start.min(self.len_chars());
        let end = range.end.min(self.len_chars());
        if start >= end {
            return String::new();
        }
        let removed: String = self.rope.slice(start..end).into();
        self.rope.remove(start..end);
        self.dirty = true;
        self.version += 1;
        removed
    }

    /// Apply an edit transaction (composed of multiple inserts/deletes), recording to history.
    pub fn apply_transaction(&mut self, transaction: Transaction) {
        for op in &transaction.ops {
            match op {
                EditOp::Insert { char_idx, text } => {
                    self.insert_raw(*char_idx, text);
                }
                EditOp::Delete { char_idx, text } => {
                    let end = *char_idx + text.chars().count();
                    self.remove_raw(*char_idx..end);
                }
            }
        }
        self.history.push(transaction);
    }

    /// Undo the last transaction, returning cursor restoration coordinates if available.
    pub fn undo(&mut self) -> Option<(usize, usize)> {
        let tx = self.history.pop_undo()?;
        for op in &tx.ops {
            match op {
                EditOp::Insert { char_idx, text } => {
                    self.insert_raw(*char_idx, text);
                }
                EditOp::Delete { char_idx, text } => {
                    let end = *char_idx + text.chars().count();
                    self.remove_raw(*char_idx..end);
                }
            }
        }
        Some(tx.cursor_after)
    }

    /// Redo the last undone transaction, returning cursor restoration coordinates if available.
    pub fn redo(&mut self) -> Option<(usize, usize)> {
        let tx = self.history.pop_redo()?;
        for op in &tx.ops {
            match op {
                EditOp::Insert { char_idx, text } => {
                    self.insert_raw(*char_idx, text);
                }
                EditOp::Delete { char_idx, text } => {
                    let end = *char_idx + text.chars().count();
                    self.remove_raw(*char_idx..end);
                }
            }
        }
        Some(tx.cursor_after)
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    // --- Metric helpers ---

    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        Metrics::char_to_line_col(&self.rope, char_idx)
    }

    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        Metrics::line_col_to_char(&self.rope, line, col)
    }

    pub fn char_to_lsp_position(&self, char_idx: usize) -> LspPosition {
        Metrics::char_to_lsp_position(&self.rope, char_idx)
    }

    pub fn lsp_position_to_char(&self, pos: LspPosition) -> usize {
        Metrics::lsp_position_to_char(&self.rope, pos)
    }

    pub fn line_to_char(&self, line_idx: usize) -> usize {
        if line_idx >= self.len_lines() {
            self.len_chars()
        } else {
            self.rope.line_to_char(line_idx)
        }
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx.min(self.len_chars()))
    }

    pub fn char_to_byte(&self, char_idx: usize) -> usize {
        self.rope.char_to_byte(char_idx.min(self.len_chars()))
    }

    pub fn byte_to_char(&self, byte_idx: usize) -> usize {
        self.rope.byte_to_char(byte_idx.min(self.len_bytes()))
    }

    /// Search for literal occurrences across the buffer. Returns character ranges.
    pub fn search_literal(&self, query: &str, case_sensitive: bool) -> Vec<Range<usize>> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let query_chars: Vec<char> = if case_sensitive {
            query.chars().collect()
        } else {
            query.to_lowercase().chars().collect()
        };
        let query_len = query_chars.len();

        let full_text: String = self.rope.to_string();
        let target_text = if case_sensitive {
            full_text.clone()
        } else {
            full_text.to_lowercase()
        };

        let needle = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut byte_offset = 0;

        while let Some(pos) = target_text[byte_offset..].find(&needle) {
            let actual_byte_pos = byte_offset + pos;
            let match_char_start = self.byte_to_char(actual_byte_pos);
            let match_char_end = match_char_start + query_len;
            matches.push(match_char_start..match_char_end);
            byte_offset = actual_byte_pos + needle.len().max(1);
        }

        matches
    }

    /// Search with regular expressions. Returns character ranges.
    pub fn search_regex(&self, regex: &Regex) -> Vec<Range<usize>> {
        let mut matches = Vec::new();
        let full_text: String = self.rope.to_string();

        for mat in regex.find_iter(&full_text) {
            let start_char = self.byte_to_char(mat.start());
            let end_char = self.byte_to_char(mat.end());
            matches.push(start_char..end_char);
        }

        matches
    }

}

impl std::fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for chunk in self.rope.chunks() {
            write!(f, "{chunk}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for TextBuffer {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_basic_ops() {
        let mut buf = TextBuffer::from_str("Hello World!\nLine 2");
        assert_eq!(buf.len_lines(), 2);
        assert_eq!(buf.len_chars(), 19);

        buf.insert_raw(5, ", Rust");
        assert_eq!(buf.to_string(), "Hello, Rust World!\nLine 2");
        assert!(buf.is_dirty());

        let removed = buf.remove_raw(5..11);
        assert_eq!(removed, ", Rust");
        assert_eq!(buf.to_string(), "Hello World!\nLine 2");
    }

    #[test]
    fn test_buffer_search() {
        let buf = TextBuffer::from_str("let foo = 1;\nlet bar = foo + 2;\nlet baz = foo;");
        let matches = buf.search_literal("foo", true);
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0], 4..7);
        assert_eq!(matches[1], 23..26);
        assert_eq!(matches[2], 42..45);

        let regex = Regex::new(r"let (\w+)").unwrap();
        let regex_matches = buf.search_regex(&regex);
        assert_eq!(regex_matches.len(), 3);
    }
}
