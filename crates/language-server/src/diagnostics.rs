use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::protocol::{Diagnostic, DiagnosticSeverity};

#[derive(Debug, Clone, Default)]
pub struct DiagnosticStore {
    diagnostics_by_file: HashMap<PathBuf, Vec<Diagnostic>>,
}

impl DiagnosticStore {
    pub fn new() -> Self {
        Self {
            diagnostics_by_file: HashMap::new(),
        }
    }

    pub fn set_diagnostics(&mut self, path: PathBuf, diagnostics: Vec<Diagnostic>) {
        if diagnostics.is_empty() {
            self.diagnostics_by_file.remove(&path);
        } else {
            self.diagnostics_by_file.insert(path, diagnostics);
        }
    }

    pub fn clear_file(&mut self, path: &Path) {
        self.diagnostics_by_file.remove(path);
    }

    pub fn get_for_file(&self, path: &Path) -> &[Diagnostic] {
        self.diagnostics_by_file
            .get(path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_for_line(&self, path: &Path, line: usize) -> Vec<&Diagnostic> {
        let line_u32 = line as u32;
        self.get_for_file(path)
            .iter()
            .filter(|d| d.range.start.line <= line_u32 && d.range.end.line >= line_u32)
            .collect()
    }

    pub fn total_errors(&self) -> usize {
        self.diagnostics_by_file
            .values()
            .flat_map(|list| list.iter())
            .filter(|d| matches!(d.severity, Some(DiagnosticSeverity::Error)))
            .count()
    }

    pub fn total_warnings(&self) -> usize {
        self.diagnostics_by_file
            .values()
            .flat_map(|list| list.iter())
            .filter(|d| matches!(d.severity, Some(DiagnosticSeverity::Warning)))
            .count()
    }

    pub fn all_diagnostics(&self) -> Vec<(&PathBuf, &Diagnostic)> {
        let mut list = Vec::new();
        for (path, diags) in &self.diagnostics_by_file {
            for diag in diags {
                list.push((path, diag));
            }
        }
        list
    }
}
