use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub branch: String,
    pub modified: Vec<PathBuf>,
    pub untracked: Vec<PathBuf>,
    pub staged: Vec<PathBuf>,
}

pub struct GitRepo;

impl GitRepo {
    /// Detects the active git branch by reading .git/HEAD directly for near-zero latency.
    pub fn detect_branch(repo_root: &Path) -> Option<String> {
        let head_path = repo_root.join(".git").join("HEAD");
        if let Ok(content) = fs::read_to_string(&head_path) {
            let trimmed = content.trim();
            if let Some(ref_path) = trimmed.strip_prefix("ref: refs/heads/") {
                return Some(ref_path.to_string());
            } else if trimmed.len() >= 7 {
                // Detached HEAD: return short commit hash
                return Some(trimmed[..7].to_string());
            }
        }
        None
    }

    /// Queries git status using git porcelain if git is installed.
    pub fn query_status(repo_root: &Path) -> GitStatus {
        let mut status = GitStatus {
            branch: Self::detect_branch(repo_root).unwrap_or_else(|| "main".to_string()),
            modified: Vec::new(),
            untracked: Vec::new(),
            staged: Vec::new(),
        };

        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(repo_root)
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.len() < 4 {
                    continue;
                }
                let x = line.chars().next().unwrap_or(' ');
                let y = line.chars().nth(1).unwrap_or(' ');
                let rel_path = &line[3..];
                let path = repo_root.join(rel_path);

                if x == 'M' || x == 'A' || x == 'D' {
                    status.staged.push(path.clone());
                }
                if y == 'M' || y == 'D' {
                    status.modified.push(path.clone());
                }
                if x == '?' && y == '?' {
                    status.untracked.push(path);
                }
            }
        }

        status
    }
}
