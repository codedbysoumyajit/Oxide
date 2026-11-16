use std::ops::Range;

/// Represents a cursor position and selection anchor in character offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub head: usize,
    pub anchor: usize,
    pub preferred_col: usize,
}

impl Cursor {
    pub const fn new(head: usize) -> Self {
        Self {
            head,
            anchor: head,
            preferred_col: 0,
        }
    }

    pub const fn with_anchor(head: usize, anchor: usize) -> Self {
        Self {
            head,
            anchor,
            preferred_col: 0,
        }
    }

    pub fn is_selection(&self) -> bool {
        self.head != self.anchor
    }

    pub fn start(&self) -> usize {
        self.head.min(self.anchor)
    }

    pub fn end(&self) -> usize {
        self.head.max(self.anchor)
    }

    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    pub fn collapse_to_head(&mut self) {
        self.anchor = self.head;
    }

    pub fn collapse_to_start(&mut self) {
        self.head = self.start();
        self.anchor = self.head;
    }

    pub fn collapse_to_end(&mut self) {
        self.head = self.end();
        self.anchor = self.head;
    }
}
