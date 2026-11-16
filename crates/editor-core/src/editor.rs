use std::path::PathBuf;
use text_buffer::{EditOp, TextBuffer, Transaction};

use crate::cursor::Cursor;
use crate::folding::FoldingManager;
use crate::indentation::{IndentConfig, Indentation};
use crate::selection::SelectionSet;

/// The central Editor model holding document text, multiple cursors, selections, and view states.
#[derive(Debug)]
pub struct Editor {
    pub buffer: TextBuffer,
    pub selections: SelectionSet,
    pub folding: FoldingManager,
    pub indent_config: IndentConfig,
    pub file_path: Option<PathBuf>,
    pub language_id: String,
    pub scroll_line: f32,
    pub scroll_col: f32,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            selections: SelectionSet::new(),
            folding: FoldingManager::new(),
            indent_config: IndentConfig::default(),
            file_path: None,
            language_id: "plaintext".to_string(),
            scroll_line: 0.0,
            scroll_col: 0.0,
        }
    }

    pub fn from_str(content: &str, file_path: Option<PathBuf>, language_id: &str) -> Self {
        Self {
            buffer: TextBuffer::from_str(content),
            selections: SelectionSet::new(),
            folding: FoldingManager::new(),
            indent_config: IndentConfig::default(),
            file_path,
            language_id: language_id.to_string(),
            scroll_line: 0.0,
            scroll_col: 0.0,
        }
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    // --- High Level Editing Operations ---

    /// Inserts text at each cursor/selection with atomic transaction history.
    pub fn insert_text(&mut self, text: &str) {
        let cursor_before = (self.selections.primary().head, self.selections.primary().anchor);
        let mut ops = Vec::new();
        let mut new_cursors = Vec::new();

        // Process cursors in reverse to maintain valid character offsets
        let mut cursors_rev: Vec<Cursor> = self.selections.cursors().to_vec();
        cursors_rev.sort_by_key(|c| std::cmp::Reverse(c.start()));

        let text_char_len = text.chars().count();

        for cursor in cursors_rev {
            if cursor.is_selection() {
                // Delete selected range first
                let removed = self.buffer.slice(cursor.range()).into();
                ops.push(EditOp::Delete {
                    char_idx: cursor.start(),
                    text: removed,
                });
            }
            ops.push(EditOp::Insert {
                char_idx: cursor.start(),
                text: text.to_string(),
            });
            let new_pos = cursor.start() + text_char_len;
            new_cursors.push(Cursor::new(new_pos));
        }

        // Apply transaction
        let tx = Transaction::new(ops, cursor_before, (0, 0));
        self.buffer.apply_transaction(tx);

        new_cursors.reverse();
        self.selections.set_cursors(new_cursors);
    }

    /// Smart character insertion with auto-bracket closing and pair wrapping.
    pub fn insert_char(&mut self, ch: char) {
        // If selection is active and bracket or quote is typed: wrap selections!
        let is_wrapper = matches!(ch, '(' | '[' | '{' | '"' | '\'' | '`');
        if is_wrapper && self.selections.primary().is_selection() {
            let close_ch = match ch {
                '(' => ')',
                '[' => ']',
                '{' => '}',
                c => c,
            };
            self.wrap_selections(ch, close_ch);
            return;
        }

        // Check for stepping over existing closing bracket
        if matches!(ch, ')' | ']' | '}' | '"' | '\'' | '`') {
            let head = self.selections.primary().head;
            if head < self.buffer.len_chars() {
                let next_char = self.buffer.slice(head..head + 1).chars().next();
                if next_char == Some(ch) && !self.selections.primary().is_selection() {
                    // Step over
                    self.move_right(false);
                    return;
                }
            }
        }

        // Auto-close open bracket pairs
        let close_pair = match ch {
            '(' => Some(')'),
            '[' => Some(']'),
            '{' => Some('}'),
            '"' => Some('"'),
            '\'' => Some('\''),
            '`' => Some('`'),
            _ => None,
        };

        if let Some(close_ch) = close_pair {
            let pair_str = format!("{}{}", ch, close_ch);
            self.insert_text(&pair_str);
            // Put cursors between the brackets
            let mut updated = self.selections.cursors().to_vec();
            for c in &mut updated {
                c.head = c.head.saturating_sub(1);
                c.anchor = c.head;
            }
            self.selections.set_cursors(updated);
        } else {
            let mut s = String::new();
            s.push(ch);
            self.insert_text(&s);
        }
    }

    /// Wrap all active selections with opening and closing delimiters.
    pub fn wrap_selections(&mut self, open: char, close: char) {
        let cursor_before = (self.selections.primary().head, self.selections.primary().anchor);
        let mut ops = Vec::new();
        let mut new_cursors = Vec::new();

        let mut cursors_rev: Vec<Cursor> = self.selections.cursors().to_vec();
        cursors_rev.sort_by_key(|c| std::cmp::Reverse(c.start()));

        for cursor in cursors_rev {
            let start = cursor.start();
            let end = cursor.end();
            ops.push(EditOp::Insert {
                char_idx: end,
                text: close.to_string(),
            });
            ops.push(EditOp::Insert {
                char_idx: start,
                text: open.to_string(),
            });
            // Keep selection inside the wrapped delimiters
            new_cursors.push(Cursor::with_anchor(end + 1, start + 1));
        }

        let tx = Transaction::new(ops, cursor_before, (0, 0));
        self.buffer.apply_transaction(tx);
        new_cursors.reverse();
        self.selections.set_cursors(new_cursors);
    }

    /// Handles Backspace with empty-pair deletion and selection removal.
    pub fn backspace(&mut self) {
        let cursor_before = (self.selections.primary().head, self.selections.primary().anchor);
        let mut ops = Vec::new();
        let mut new_cursors = Vec::new();

        let mut cursors_rev: Vec<Cursor> = self.selections.cursors().to_vec();
        cursors_rev.sort_by_key(|c| std::cmp::Reverse(c.start()));

        for cursor in cursors_rev {
            if cursor.is_selection() {
                let removed = self.buffer.slice(cursor.range()).into();
                ops.push(EditOp::Delete {
                    char_idx: cursor.start(),
                    text: removed,
                });
                new_cursors.push(Cursor::new(cursor.start()));
            } else if cursor.head > 0 {
                // Check if inside bracket pair like () or {}
                let head = cursor.head;
                let prev_ch = self.buffer.slice(head - 1..head).chars().next();
                let next_ch = if head < self.buffer.len_chars() {
                    self.buffer.slice(head..head + 1).chars().next()
                } else {
                    None
                };

                let is_pair = matches!(
                    (prev_ch, next_ch),
                    (Some('('), Some(')'))
                        | (Some('['), Some(']'))
                        | (Some('{'), Some('}'))
                        | (Some('"'), Some('"'))
                        | (Some('\''), Some('\''))
                        | (Some('`'), Some('`'))
                );

                if is_pair {
                    // Delete both
                    let removed = self.buffer.slice(head - 1..head + 1).into();
                    ops.push(EditOp::Delete {
                        char_idx: head - 1,
                        text: removed,
                    });
                    new_cursors.push(Cursor::new(head - 1));
                } else {
                    // Delete single char
                    let removed = self.buffer.slice(head - 1..head).into();
                    ops.push(EditOp::Delete {
                        char_idx: head - 1,
                        text: removed,
                    });
                    new_cursors.push(Cursor::new(head - 1));
                }
            } else {
                new_cursors.push(cursor);
            }
        }

        if !ops.is_empty() {
            let tx = Transaction::new(ops, cursor_before, (0, 0));
            self.buffer.apply_transaction(tx);
            new_cursors.reverse();
            self.selections.set_cursors(new_cursors);
        }
    }

    /// Handles Forward Delete.
    pub fn delete_forward(&mut self) {
        let cursor_before = (self.selections.primary().head, self.selections.primary().anchor);
        let mut ops = Vec::new();
        let mut new_cursors = Vec::new();

        let mut cursors_rev: Vec<Cursor> = self.selections.cursors().to_vec();
        cursors_rev.sort_by_key(|c| std::cmp::Reverse(c.start()));

        for cursor in cursors_rev {
            if cursor.is_selection() {
                let removed = self.buffer.slice(cursor.range()).into();
                ops.push(EditOp::Delete {
                    char_idx: cursor.start(),
                    text: removed,
                });
                new_cursors.push(Cursor::new(cursor.start()));
            } else if cursor.head < self.buffer.len_chars() {
                let removed = self.buffer.slice(cursor.head..cursor.head + 1).into();
                ops.push(EditOp::Delete {
                    char_idx: cursor.head,
                    text: removed,
                });
                new_cursors.push(cursor);
            } else {
                new_cursors.push(cursor);
            }
        }

        if !ops.is_empty() {
            let tx = Transaction::new(ops, cursor_before, (0, 0));
            self.buffer.apply_transaction(tx);
            new_cursors.reverse();
            self.selections.set_cursors(new_cursors);
        }
    }

    /// Handles Enter with auto-indentation.
    pub fn insert_newline(&mut self) {
        let primary_head = self.selections.primary().head;
        let (line_idx, _) = self.buffer.char_to_line_col(primary_head);
        let line_content = self
            .buffer
            .line(line_idx)
            .map(|s| s.to_string())
            .unwrap_or_default();

        let indent = Indentation::compute_enter_indent(&line_content, &self.indent_config);
        let insertion = format!("\n{}", indent);
        self.insert_text(&insertion);
    }

    // --- Cursor Navigation Operations ---

    pub fn move_left(&mut self, select: bool) {
        let cursors: Vec<Cursor> = self
            .selections
            .cursors()
            .iter()
            .map(|c| {
                let new_head = if !select && c.is_selection() {
                    c.start()
                } else {
                    c.head.saturating_sub(1)
                };
                let new_anchor = if select { c.anchor } else { new_head };
                Cursor::with_anchor(new_head, new_anchor)
            })
            .collect();
        self.selections.set_cursors(cursors);
    }

    pub fn move_right(&mut self, select: bool) {
        let max_len = self.buffer.len_chars();
        let cursors: Vec<Cursor> = self
            .selections
            .cursors()
            .iter()
            .map(|c| {
                let new_head = if !select && c.is_selection() {
                    c.end()
                } else {
                    (c.head + 1).min(max_len)
                };
                let new_anchor = if select { c.anchor } else { new_head };
                Cursor::with_anchor(new_head, new_anchor)
            })
            .collect();
        self.selections.set_cursors(cursors);
    }

    pub fn move_up(&mut self, select: bool) {
        let cursors: Vec<Cursor> = self
            .selections
            .cursors()
            .iter()
            .map(|c| {
                let (line, col) = self.buffer.char_to_line_col(c.head);
                let target_col = if c.preferred_col > 0 { c.preferred_col } else { col };
                if line > 0 {
                    let new_head = self.buffer.line_col_to_char(line - 1, target_col);
                    let mut cur = Cursor::with_anchor(new_head, if select { c.anchor } else { new_head });
                    cur.preferred_col = target_col;
                    cur
                } else {
                    let mut cur = Cursor::with_anchor(0, if select { c.anchor } else { 0 });
                    cur.preferred_col = target_col;
                    cur
                }
            })
            .collect();
        self.selections.set_cursors(cursors);
    }

    pub fn move_down(&mut self, select: bool) {
        let max_lines = self.buffer.len_lines();
        let cursors: Vec<Cursor> = self
            .selections
            .cursors()
            .iter()
            .map(|c| {
                let (line, col) = self.buffer.char_to_line_col(c.head);
                let target_col = if c.preferred_col > 0 { c.preferred_col } else { col };
                if line + 1 < max_lines {
                    let new_head = self.buffer.line_col_to_char(line + 1, target_col);
                    let mut cur = Cursor::with_anchor(new_head, if select { c.anchor } else { new_head });
                    cur.preferred_col = target_col;
                    cur
                } else {
                    let end_pos = self.buffer.len_chars();
                    let mut cur = Cursor::with_anchor(end_pos, if select { c.anchor } else { end_pos });
                    cur.preferred_col = target_col;
                    cur
                }
            })
            .collect();
        self.selections.set_cursors(cursors);
    }

    pub fn move_to_line_start(&mut self, select: bool) {
        let cursors: Vec<Cursor> = self
            .selections
            .cursors()
            .iter()
            .map(|c| {
                let (line, _) = self.buffer.char_to_line_col(c.head);
                let start_char = self.buffer.line_to_char(line);
                Cursor::with_anchor(start_char, if select { c.anchor } else { start_char })
            })
            .collect();
        self.selections.set_cursors(cursors);
    }

    pub fn move_to_line_end(&mut self, select: bool) {
        let cursors: Vec<Cursor> = self
            .selections
            .cursors()
            .iter()
            .map(|c| {
                let (line, _) = self.buffer.char_to_line_col(c.head);
                let line_len = self.buffer.line(line).map(|l| l.len_chars()).unwrap_or(0);
                let line_start = self.buffer.line_to_char(line);
                // Exclude newline from line end
                let line_str = self.buffer.line(line).map(|l| l.to_string()).unwrap_or_default();
                let actual_len = if line_str.ends_with("\r\n") {
                    line_len.saturating_sub(2)
                } else if line_str.ends_with('\n') {
                    line_len.saturating_sub(1)
                } else {
                    line_len
                };
                let end_char = line_start + actual_len;
                Cursor::with_anchor(end_char, if select { c.anchor } else { end_char })
            })
            .collect();
        self.selections.set_cursors(cursors);
    }

    pub fn select_all(&mut self) {
        let len = self.buffer.len_chars();
        self.selections.set_cursors(vec![Cursor::with_anchor(len, 0)]);
    }

    pub fn add_cursor_above(&mut self) {
        let prim = *self.selections.primary();
        let (line, col) = self.buffer.char_to_line_col(prim.head);
        if line > 0 {
            let target = self.buffer.line_col_to_char(line - 1, col);
            self.selections.add_cursor(Cursor::new(target));
        }
    }

    pub fn add_cursor_below(&mut self) {
        let prim = *self.selections.primary();
        let (line, col) = self.buffer.char_to_line_col(prim.head);
        if line + 1 < self.buffer.len_lines() {
            let target = self.buffer.line_col_to_char(line + 1, col);
            self.selections.add_cursor(Cursor::new(target));
        }
    }

    pub fn undo(&mut self) {
        if let Some((head, anchor)) = self.buffer.undo() {
            self.selections.set_cursors(vec![Cursor::with_anchor(head, anchor)]);
        }
    }

    pub fn redo(&mut self) {
        if let Some((head, anchor)) = self.buffer.redo() {
            self.selections.set_cursors(vec![Cursor::with_anchor(head, anchor)]);
        }
    }

    pub fn copy_selection(&self) -> String {
        let mut out = String::new();
        for (i, cursor) in self.selections.cursors().iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if cursor.is_selection() {
                out.push_str(&self.buffer.slice(cursor.range()).to_string());
            } else {
                // Copy entire current line if no selection
                let (line, _) = self.buffer.char_to_line_col(cursor.head);
                if let Some(l) = self.buffer.line(line) {
                    out.push_str(&l.to_string());
                }
            }
        }
        out
    }

    /// Find matching bracket from position.
    pub fn find_matching_bracket(&self, char_idx: usize) -> Option<usize> {
        if char_idx >= self.buffer.len_chars() {
            return None;
        }

        let ch = self.buffer.slice(char_idx..char_idx + 1).chars().next()?;
        let (is_open, target_ch) = match ch {
            '(' => (true, ')'),
            ')' => (false, '('),
            '[' => (true, ']'),
            ']' => (false, '['),
            '{' => (true, '}'),
            '}' => (false, '{'),
            _ => return None,
        };

        let mut depth = 0;
        if is_open {
            for i in char_idx..self.buffer.len_chars() {
                if let Some(c) = self.buffer.slice(i..i + 1).chars().next() {
                    if c == ch {
                        depth += 1;
                    } else if c == target_ch {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                }
            }
        } else {
            for i in (0..=char_idx).rev() {
                if let Some(c) = self.buffer.slice(i..i + 1).chars().next() {
                    if c == ch {
                        depth += 1;
                    } else if c == target_ch {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_typing_and_undo() {
        let mut ed = Editor::new();
        ed.insert_char('(');
        // Expect auto-closed brackets "()" and cursor between them at index 1
        assert_eq!(ed.buffer.to_string(), "()");
        assert_eq!(ed.selections.primary().head, 1);

        ed.insert_text("123");
        assert_eq!(ed.buffer.to_string(), "(123)");

        // Undo
        ed.undo();
        assert_eq!(ed.buffer.to_string(), "()");

        ed.undo();
        assert_eq!(ed.buffer.to_string(), "");
    }

    #[test]
    fn test_editor_multicursor() {
        let mut ed = Editor::from_str("line1\nline2\nline3", None, "plaintext");
        ed.selections.set_cursors(vec![Cursor::new(0), Cursor::new(6), Cursor::new(12)]);

        ed.insert_text("# ");
        assert_eq!(ed.buffer.to_string(), "# line1\n# line2\n# line3");
    }

    #[test]
    fn test_bracket_matching() {
        let ed = Editor::from_str("fn main() { let x = (1 + 2); }", None, "rust");
        let open_brace = 10; // '{'
        let close_brace = ed.find_matching_bracket(open_brace);
        assert_eq!(close_brace, Some(29)); // '}'
    }
}
