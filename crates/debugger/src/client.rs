use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::protocol::{Breakpoint, DebugState, StackFrame, Variable};

pub struct DebugSession {
    pub state: DebugState,
    pub breakpoints: HashMap<PathBuf, Vec<Breakpoint>>,
    pub stack_frames: Vec<StackFrame>,
    pub variables: Vec<Variable>,
}

impl Default for DebugSession {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugSession {
    pub fn new() -> Self {
        Self {
            state: DebugState::Inactive,
            breakpoints: HashMap::new(),
            stack_frames: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn toggle_breakpoint(&mut self, path: &Path, line: usize) {
        let bps = self.breakpoints.entry(path.to_path_buf()).or_default();
        if let Some(pos) = bps.iter().position(|b| b.line == line) {
            bps.remove(pos);
        } else {
            bps.push(Breakpoint {
                id: None,
                verified: true,
                line,
                file_path: path.to_path_buf(),
            });
        }
    }

    pub fn has_breakpoint(&self, path: &Path, line: usize) -> bool {
        self.breakpoints
            .get(path)
            .map(|list| list.iter().any(|b| b.line == line))
            .unwrap_or(false)
    }

    pub fn clear_all_breakpoints(&mut self) {
        self.breakpoints.clear();
    }
}
