use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    Insert { char_idx: usize, text: String },
    Delete { char_idx: usize, text: String },
}

impl EditOp {
    pub fn inverted(&self) -> EditOp {
        match self {
            EditOp::Insert { char_idx, text } => EditOp::Delete {
                char_idx: *char_idx,
                text: text.clone(),
            },
            EditOp::Delete { char_idx, text } => EditOp::Insert {
                char_idx: *char_idx,
                text: text.clone(),
            },
        }
    }
}

/// A transactional group of edits with cursor restore state.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub ops: Vec<EditOp>,
    pub timestamp: Instant,
    pub cursor_before: (usize, usize), // (primary_cursor_char, primary_anchor_char)
    pub cursor_after: (usize, usize),
}

impl Transaction {
    pub fn new(
        ops: Vec<EditOp>,
        cursor_before: (usize, usize),
        cursor_after: (usize, usize),
    ) -> Self {
        Self {
            ops,
            timestamp: Instant::now(),
            cursor_before,
            cursor_after,
        }
    }

    /// Invert transaction for undo.
    pub fn inverted(&self) -> Self {
        let inverted_ops: Vec<EditOp> = self.ops.iter().rev().map(|op| op.inverted()).collect();
        Self {
            ops: inverted_ops,
            timestamp: Instant::now(),
            cursor_before: self.cursor_after,
            cursor_after: self.cursor_before,
        }
    }
}

/// History stack managing Undo and Redo with transactional grouping.
#[derive(Debug)]
pub struct History {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
    max_depth: usize,
}

impl History {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub fn push(&mut self, transaction: Transaction) {
        if transaction.ops.is_empty() {
            return;
        }

        self.redo_stack.clear();

        // Check for coalescing simple typing
        if let Some(last) = self.undo_stack.last_mut() {
            if last.ops.len() == 1
                && transaction.ops.len() == 1
                && last.timestamp.elapsed().as_millis() < 500
            {
                if let (
                    EditOp::Insert { char_idx: idx1, text: text1 },
                    EditOp::Insert { char_idx: idx2, text: text2 },
                ) = (&mut last.ops[0], &transaction.ops[0])
                {
                    if *idx1 + text1.chars().count() == *idx2
                        && !text1.ends_with('\n')
                        && !text2.ends_with('\n')
                        && !text2.starts_with(' ')
                    {
                        text1.push_str(text2);
                        last.cursor_after = transaction.cursor_after;
                        last.timestamp = Instant::now();
                        return;
                    }
                }
            }
        }

        self.undo_stack.push(transaction);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn pop_undo(&mut self) -> Option<Transaction> {
        let tx = self.undo_stack.pop()?;
        self.redo_stack.push(tx.clone());
        Some(tx.inverted())
    }

    pub fn pop_redo(&mut self) -> Option<Transaction> {
        let tx = self.redo_stack.pop()?;
        self.undo_stack.push(tx.clone());
        Some(tx)
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_transactions() {
        let mut history = History::new(100);

        let tx1 = Transaction::new(
            vec![EditOp::Insert {
                char_idx: 0,
                text: "hello".to_string(),
            }],
            (0, 0),
            (5, 5),
        );
        history.push(tx1);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undo_tx = history.pop_undo().unwrap();
        assert_eq!(
            undo_tx.ops,
            vec![EditOp::Delete {
                char_idx: 0,
                text: "hello".to_string(),
            }]
        );
        assert_eq!(undo_tx.cursor_after, (0, 0));

        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redo_tx = history.pop_redo().unwrap();
        assert_eq!(
            redo_tx.ops,
            vec![EditOp::Insert {
                char_idx: 0,
                text: "hello".to_string(),
            }]
        );
        assert_eq!(redo_tx.cursor_after, (5, 5));
    }
}
