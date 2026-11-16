use smallvec::{smallvec, SmallVec};
use crate::cursor::Cursor;

/// Manages multiple cursors and selection ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    cursors: SmallVec<[Cursor; 4]>,
    primary_idx: usize,
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            cursors: smallvec![Cursor::new(0)],
            primary_idx: 0,
        }
    }

    pub fn single(head: usize) -> Self {
        Self {
            cursors: smallvec![Cursor::new(head)],
            primary_idx: 0,
        }
    }

    pub fn single_range(head: usize, anchor: usize) -> Self {
        Self {
            cursors: smallvec![Cursor::with_anchor(head, anchor)],
            primary_idx: 0,
        }
    }

    #[inline]
    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }

    #[inline]
    pub fn cursors_mut(&mut self) -> &mut [Cursor] {
        &mut self.cursors
    }

    #[inline]
    pub fn primary(&self) -> &Cursor {
        &self.cursors[self.primary_idx.min(self.cursors.len().saturating_sub(1))]
    }

    #[inline]
    pub fn primary_mut(&mut self) -> &mut Cursor {
        let idx = self.primary_idx.min(self.cursors.len().saturating_sub(1));
        &mut self.cursors[idx]
    }

    pub fn add_cursor(&mut self, cursor: Cursor) {
        self.cursors.push(cursor);
        self.normalize();
    }

    pub fn set_cursors(&mut self, cursors: Vec<Cursor>) {
        if cursors.is_empty() {
            self.cursors = smallvec![Cursor::new(0)];
            self.primary_idx = 0;
        } else {
            self.cursors = SmallVec::from_vec(cursors);
            self.primary_idx = self.cursors.len() - 1;
            self.normalize();
        }
    }

    pub fn clear_secondary(&mut self) {
        let prim = *self.primary();
        self.cursors = smallvec![prim];
        self.primary_idx = 0;
    }

    /// Sorts cursors by start position and merges any that overlap.
    pub fn normalize(&mut self) {
        if self.cursors.len() <= 1 {
            return;
        }

        // Sort by start position
        self.cursors.sort_by_key(|c| c.start());

        let mut merged: SmallVec<[Cursor; 4]> = SmallVec::new();
        for cursor in self.cursors.drain(..) {
            if let Some(last) = merged.last_mut() {
                if cursor.start() <= last.end() {
                    // Overlap or touching: merge into single span
                    let new_start = last.start().min(cursor.start());
                    let new_end = last.end().max(cursor.end());
                    // Keep the head direction of the later cursor if it had a selection
                    let is_forward = cursor.head >= cursor.anchor;
                    let (new_head, new_anchor) = if is_forward {
                        (new_end, new_start)
                    } else {
                        (new_start, new_end)
                    };
                    last.head = new_head;
                    last.anchor = new_anchor;
                    continue;
                }
            }
            merged.push(cursor);
        }

        self.cursors = merged;
        if self.primary_idx >= self.cursors.len() {
            self.primary_idx = self.cursors.len().saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_merging() {
        let mut sel = SelectionSet::new();
        sel.set_cursors(vec![
            Cursor::with_anchor(5, 0),
            Cursor::with_anchor(8, 4),
            Cursor::with_anchor(20, 15),
        ]);

        assert_eq!(sel.cursors().len(), 2);
        assert_eq!(sel.cursors()[0].start(), 0);
        assert_eq!(sel.cursors()[0].end(), 8);
        assert_eq!(sel.cursors()[1].start(), 15);
        assert_eq!(sel.cursors()[1].end(), 20);
    }
}
