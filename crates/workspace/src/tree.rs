use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub is_expanded: bool,
}

impl FileNode {
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Self {
            name,
            path,
            is_dir,
            children: Vec::new(),
            is_expanded: false,
        }
    }

    /// Read directory entries one level deep for lazy-loading.
    pub fn populate_children(&mut self) {
        if !self.is_dir {
            return;
        }

        let mut entries = Vec::new();
        if let Ok(dir) = fs::read_dir(&self.path) {
            for entry in dir.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                // Filter out heavy/hidden dirs by default
                if file_name == ".git" || file_name == "target" || file_name == "node_modules" {
                    continue;
                }
                let path = entry.path();
                let is_dir = path.is_dir();
                entries.push(FileNode::new(path, is_dir));
            }
        }

        // Sort directories first, then alphabetical
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.children = entries;
    }
}
