use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub category: String,
    pub keybinding: Option<String>,
}

impl Command {
    pub fn new(id: &str, title: &str, category: &str, keybinding: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            category: category.to_string(),
            keybinding: keybinding.map(|s| s.to_string()),
        }
    }

    pub fn display_name(&self) -> String {
        format!("{}: {}", self.category, self.title)
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    order: Vec<String>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::with_default_commands()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn register(&mut self, cmd: Command) {
        let id = cmd.id.clone();
        if !self.commands.contains_key(&id) {
            self.order.push(id.clone());
        }
        self.commands.insert(id, cmd);
    }

    pub fn get(&self, id: &str) -> Option<&Command> {
        self.commands.get(id)
    }

    pub fn list(&self) -> Vec<&Command> {
        self.order
            .iter()
            .filter_map(|id| self.commands.get(id))
            .collect()
    }

    /// Fuzzy search matching command titles and IDs.
    pub fn search(&self, query: &str) -> Vec<&Command> {
        if query.is_empty() {
            return self.list();
        }

        let q = query.to_lowercase();
        let mut results: Vec<(&Command, usize)> = self
            .commands
            .values()
            .filter_map(|cmd| {
                let title_lower = cmd.title.to_lowercase();
                let cat_lower = cmd.category.to_lowercase();
                let full = format!("{} {}", cat_lower, title_lower);

                if let Some(pos) = full.find(&q) {
                    Some((cmd, pos))
                } else if q.chars().all(|c| full.contains(c)) {
                    Some((cmd, 100))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by_key(|(_, score)| *score);
        results.into_iter().map(|(cmd, _)| cmd).collect()
    }

    /// Initializes all default VS Code compatible editor commands.
    pub fn with_default_commands() -> Self {
        let mut reg = Self::new();

        // File operations
        reg.register(Command::new("file.new", "New File", "File", Some("Ctrl+N")));
        reg.register(Command::new("file.open", "Open File...", "File", Some("Ctrl+O")));
        reg.register(Command::new("file.open_folder", "Open Folder...", "File", Some("Ctrl+K Ctrl+O")));
        reg.register(Command::new("file.save", "Save", "File", Some("Ctrl+S")));
        reg.register(Command::new("file.save_as", "Save As...", "File", Some("Ctrl+Shift+S")));
        reg.register(Command::new("file.close_tab", "Close Editor", "File", Some("Ctrl+W")));

        // Edit operations
        reg.register(Command::new("edit.undo", "Undo", "Edit", Some("Ctrl+Z")));
        reg.register(Command::new("edit.redo", "Redo", "Edit", Some("Ctrl+Y")));
        reg.register(Command::new("edit.cut", "Cut", "Edit", Some("Ctrl+X")));
        reg.register(Command::new("edit.copy", "Copy", "Edit", Some("Ctrl+C")));
        reg.register(Command::new("edit.paste", "Paste", "Edit", Some("Ctrl+V")));
        reg.register(Command::new("edit.find", "Find", "Edit", Some("Ctrl+F")));
        reg.register(Command::new("edit.replace", "Replace", "Edit", Some("Ctrl+H")));
        reg.register(Command::new("edit.select_all", "Select All", "Edit", Some("Ctrl+A")));
        reg.register(Command::new("edit.add_cursor_above", "Add Cursor Above", "Edit", Some("Ctrl+Alt+Up")));
        reg.register(Command::new("edit.add_cursor_below", "Add Cursor Below", "Edit", Some("Ctrl+Alt+Down")));

        // View & Navigation
        reg.register(Command::new("view.command_palette", "Command Palette", "View", Some("Ctrl+Shift+P")));
        reg.register(Command::new("view.quick_open", "Quick Open, Go to File...", "Go", Some("Ctrl+P")));
        reg.register(Command::new("view.toggle_sidebar", "Toggle Primary Side Bar", "View", Some("Ctrl+B")));
        reg.register(Command::new("view.toggle_terminal", "Toggle Integrated Terminal", "View", Some("Ctrl+`")));
        reg.register(Command::new("view.split_editor", "Split Editor Right", "View", Some("Ctrl+\\")));
        reg.register(Command::new("view.toggle_problems", "Toggle Problems Panel", "View", Some("Ctrl+Shift+M")));
        reg.register(Command::new("view.zoom_in", "Zoom In", "View", Some("Ctrl+=")));
        reg.register(Command::new("view.zoom_out", "Zoom Out", "View", Some("Ctrl+-")));

        // Selection & Code
        reg.register(Command::new("editor.fold", "Fold", "Editor", Some("Ctrl+Shift+[")));
        reg.register(Command::new("editor.unfold", "Unfold", "Editor", Some("Ctrl+Shift+]")));
        reg.register(Command::new("editor.format", "Format Document", "Editor", Some("Shift+Alt+F")));
        reg.register(Command::new("editor.go_to_definition", "Go to Definition", "Go", Some("F12")));
        reg.register(Command::new("editor.rename", "Rename Symbol", "Editor", Some("F2")));

        reg
    }
}
