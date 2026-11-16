use std::collections::HashSet;
use text_buffer::TextBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct FoldingManager {
    folded_lines: HashSet<usize>, // start lines that are folded
}

impl FoldingManager {
    pub fn new() -> Self {
        Self {
            folded_lines: HashSet::new(),
        }
    }

    pub fn is_folded(&self, start_line: usize) -> bool {
        self.folded_lines.contains(&start_line)
    }

    pub fn toggle_fold(&mut self, start_line: usize) {
        if self.folded_lines.contains(&start_line) {
            self.folded_lines.remove(&start_line);
        } else {
            self.folded_lines.insert(start_line);
        }
    }

    pub fn fold(&mut self, start_line: usize) {
        self.folded_lines.insert(start_line);
    }

    pub fn unfold(&mut self, start_line: usize) {
        self.folded_lines.remove(&start_line);
    }

    pub fn unfold_all(&mut self) {
        self.folded_lines.clear();
    }

    /// Compute foldable regions in buffer by scanning for matching braces `{ ... }` or indentation.
    pub fn compute_fold_ranges(buffer: &TextBuffer) -> Vec<FoldRange> {
        let mut ranges = Vec::new();
        let mut brace_stack = Vec::new();

        let num_lines = buffer.len_lines();
        for line_idx in 0..num_lines {
            if let Some(line) = buffer.line(line_idx) {
                let line_str: String = line.into();
                for ch in line_str.chars() {
                    if ch == '{' {
                        brace_stack.push(line_idx);
                    } else if ch == '}' {
                        if let Some(start_line) = brace_stack.pop() {
                            if start_line < line_idx {
                                ranges.push(FoldRange {
                                    start_line,
                                    end_line: line_idx,
                                });
                            }
                        }
                    }
                }
            }
        }

        ranges
    }

    /// Check if a given line is hidden because an enclosing region is folded.
    pub fn is_line_hidden(&self, line: usize, fold_ranges: &[FoldRange]) -> bool {
        for range in fold_ranges {
            if self.is_folded(range.start_line) && line > range.start_line && line <= range.end_line {
                return true;
            }
        }
        false
    }
}
