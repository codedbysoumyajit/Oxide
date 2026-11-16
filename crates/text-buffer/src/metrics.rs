use ropey::Rope;

/// Position within a text buffer in lines and UTF-16 code units (as required by LSP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

impl LspPosition {
    pub const fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Helper conversions between char offsets, byte offsets, lines, and LSP UTF-16 positions.
pub struct Metrics;

impl Metrics {
    /// Convert char offset to (line, col_char).
    pub fn char_to_line_col(rope: &Rope, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(rope.len_chars());
        let line_idx = rope.char_to_line(char_idx);
        let line_start_char = rope.line_to_char(line_idx);
        let col = char_idx.saturating_sub(line_start_char);
        (line_idx, col)
    }

    /// Convert line and char column to absolute char offset.
    pub fn line_col_to_char(rope: &Rope, line: usize, col: usize) -> usize {
        if line >= rope.len_lines() {
            return rope.len_chars();
        }
        let line_start = rope.line_to_char(line);
        let line_slice = rope.line(line);
        let line_len = line_slice.len_chars();
        line_start + col.min(line_len)
    }

    /// Convert char offset to LSP position (line, UTF-16 code unit offset).
    pub fn char_to_lsp_position(rope: &Rope, char_idx: usize) -> LspPosition {
        let (line_idx, col_chars) = Self::char_to_line_col(rope, char_idx);
        if line_idx >= rope.len_lines() {
            return LspPosition::new(line_idx as u32, 0);
        }
        let line_slice = rope.line(line_idx);
        let mut utf16_col = 0u32;
        for ch in line_slice.chars().take(col_chars) {
            utf16_col += ch.len_utf16() as u32;
        }
        LspPosition::new(line_idx as u32, utf16_col)
    }

    /// Convert LSP position (line, UTF-16 code unit offset) to char offset.
    pub fn lsp_position_to_char(rope: &Rope, pos: LspPosition) -> usize {
        let line_idx = (pos.line as usize).min(rope.len_lines().saturating_sub(1));
        let line_start = rope.line_to_char(line_idx);
        let line_slice = rope.line(line_idx);

        let mut current_utf16 = 0u32;
        let mut char_count = 0usize;

        for ch in line_slice.chars() {
            if current_utf16 >= pos.character {
                break;
            }
            current_utf16 += ch.len_utf16() as u32;
            char_count += 1;
        }

        line_start + char_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_conversions() {
        let text = "Hello\nWorld 🦀!\nAnother line";
        let rope = Rope::from_str(text);

        let (line, col) = Metrics::char_to_line_col(&rope, 0);
        assert_eq!((line, col), (0, 0));

        let (line, col) = Metrics::char_to_line_col(&rope, 6);
        assert_eq!((line, col), (1, 0));

        // Emoji 🦀 takes 1 char in Ropey, but 2 code units in UTF-16
        // "World " is 6 chars, so 🦀 is at char offset 6 + 6 = 12
        let lsp_pos = Metrics::char_to_lsp_position(&rope, 13);
        // line 1, "World 🦀" -> 6 + 2 = 8 utf16 units
        assert_eq!(lsp_pos.line, 1);
        assert_eq!(lsp_pos.character, 8);

        let back_char = Metrics::lsp_position_to_char(&rope, lsp_pos);
        assert_eq!(back_char, 13);
    }
}
