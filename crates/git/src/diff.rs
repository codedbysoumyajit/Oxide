use std::collections::HashMap;
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GutterMarker {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
    Equal,
    Insert,
    Delete,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffKind,
    pub content: String,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
}

pub struct GitDiff;

impl GitDiff {
    /// Computes gutter change markers for each line in the current buffer compared against original.
    pub fn compute_gutter_markers(original: &str, current: &str) -> HashMap<usize, GutterMarker> {
        let mut markers = HashMap::new();
        let diff = TextDiff::from_lines(original, current);

        let mut current_line = 0;
        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    current_line += 1;
                }
                ChangeTag::Insert => {
                    markers.insert(current_line, GutterMarker::Added);
                    current_line += 1;
                }
                ChangeTag::Delete => {
                    // Mark the current line with a deleted marker
                    markers.insert(current_line, GutterMarker::Deleted);
                }
            }
        }

        markers
    }

    /// Computes line-by-line diff for the diff viewer panel.
    pub fn compute_unified_diff(original: &str, current: &str) -> Vec<DiffLine> {
        let mut lines = Vec::new();
        let diff = TextDiff::from_lines(original, current);

        let mut old_idx = 1;
        let mut new_idx = 1;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    lines.push(DiffLine {
                        kind: DiffKind::Equal,
                        content: change.to_string(),
                        old_line: Some(old_idx),
                        new_line: Some(new_idx),
                    });
                    old_idx += 1;
                    new_idx += 1;
                }
                ChangeTag::Insert => {
                    lines.push(DiffLine {
                        kind: DiffKind::Insert,
                        content: change.to_string(),
                        old_line: None,
                        new_line: Some(new_idx),
                    });
                    new_idx += 1;
                }
                ChangeTag::Delete => {
                    lines.push(DiffLine {
                        kind: DiffKind::Delete,
                        content: change.to_string(),
                        old_line: Some(old_idx),
                        new_line: None,
                    });
                    old_idx += 1;
                }
            }
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gutter_markers() {
        let old = "line 1\nline 2\nline 3\n";
        let new = "line 1\nline 2 modified\nline 3\nline 4 added\n";

        let markers = GitDiff::compute_gutter_markers(old, new);
        assert!(markers.contains_key(&1)); // line 2 was changed/inserted
        assert_eq!(markers.get(&3), Some(&GutterMarker::Added)); // line 4 added
    }
}
