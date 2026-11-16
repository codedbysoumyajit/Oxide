use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;

use crate::encoding::LineEnding;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("File decoding error")]
    DecodeError,
    #[error("Path error: {0}")]
    PathError(String),
}

pub struct FileSystem;

impl FileSystem {
    /// Read a file from disk with automatic character encoding detection and line ending detection.
    pub fn read_file(path: &Path) -> Result<(String, LineEnding), FsError> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Detect BOM or encoding
        let (cow, _encoding_used, had_errors) = encoding_rs::UTF_8.decode(&bytes);
        let content = if !had_errors {
            cow.into_owned()
        } else {
            // Fallback: detect with WINDOWS_1252 or replacement
            let (fallback_cow, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
            fallback_cow.into_owned()
        };

        let line_ending = LineEnding::detect(&content);
        let normalized = LineEnding::normalize_to_lf(&content);

        Ok((normalized, line_ending))
    }

    /// Atomically write a file to disk via a temporary file replacement to prevent data corruption.
    pub fn write_file_atomic(
        path: &Path,
        content: &str,
        line_ending: LineEnding,
    ) -> Result<(), FsError> {
        let parent = path
            .parent()
            .ok_or_else(|| FsError::PathError("Invalid parent directory".into()))?;

        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = parent.join(format!(
            ".{}.tmp",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "editor_save".to_string())
        ));

        let encoded_text = LineEnding::apply(content, line_ending);

        {
            let mut temp_file = File::create(&temp_path)?;
            temp_file.write_all(encoded_text.as_bytes())?;
            temp_file.flush()?;
            temp_file.sync_data()?;
        }

        // Atomic rename
        fs::rename(&temp_path, path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_write_and_read() {
        let dir = std::env::temp_dir().join("oxide_fs_test");
        let file_path = dir.join("test_doc.rs");

        let content = "fn test() {\n    println!(\"hello\");\n}\n";
        FileSystem::write_file_atomic(&file_path, content, LineEnding::Lf).unwrap();

        let (read_back, ending) = FileSystem::read_file(&file_path).unwrap();
        assert_eq!(read_back, content);
        assert_eq!(ending, LineEnding::Lf);

        let _ = fs::remove_dir_all(&dir);
    }
}
