use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::tree::FileNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub roots: Vec<PathBuf>,
    pub recent_files: Vec<PathBuf>,
}

pub struct Workspace {
    pub roots: Vec<PathBuf>,
    pub file_trees: Vec<FileNode>,
    pub open_files: Vec<PathBuf>,
    pub active_file_idx: Option<usize>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            file_trees: Vec::new(),
            open_files: Vec::new(),
            active_file_idx: None,
        }
    }

    pub fn open_folder(&mut self, path: PathBuf) {
        if !self.roots.contains(&path) {
            let mut node = FileNode::new(path.clone(), true);
            node.is_expanded = true;
            node.populate_children();
            self.file_trees.push(node);
            self.roots.push(path);
        }
    }

    pub fn open_file(&mut self, path: PathBuf) -> usize {
        if let Some(pos) = self.open_files.iter().position(|p| p == &path) {
            self.active_file_idx = Some(pos);
            pos
        } else {
            self.open_files.push(path);
            let idx = self.open_files.len() - 1;
            self.active_file_idx = Some(idx);
            idx
        }
    }

    pub fn close_file(&mut self, idx: usize) {
        if idx < self.open_files.len() {
            self.open_files.remove(idx);
            if self.open_files.is_empty() {
                self.active_file_idx = None;
            } else if let Some(active) = self.active_file_idx {
                if active >= self.open_files.len() {
                    self.active_file_idx = Some(self.open_files.len() - 1);
                }
            }
        }
    }

    pub fn active_path(&self) -> Option<&Path> {
        self.active_file_idx
            .and_then(|idx| self.open_files.get(idx))
            .map(|p| p.as_path())
    }
}
