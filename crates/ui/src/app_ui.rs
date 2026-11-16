use std::collections::HashMap;
use std::path::{Path, PathBuf};
use commands::{Command, CommandRegistry};
use editor_core::Editor;
use filesystem::{FileSystem, LineEnding};
use git::{GitRepo, GitStatus, GutterMarker};
use language_server::DiagnosticStore;
use settings::{EditorSettings, Theme};
use terminal::TerminalProcess;
use workspace::Workspace;

use crate::command_palette::CommandPalette;
use crate::editor_view::EditorView;
use crate::explorer_view::ExplorerView;
use crate::git_view::GitView;
use crate::problems_view::ProblemsView;
use crate::search_view::SearchView;
use crate::status_bar::StatusBar;
use crate::tabs::TabBar;
use crate::terminal_view::TerminalView;
use crate::theme::to_egui_color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveSidebar {
    Explorer,
    Search,
    SourceControl,
    Extensions,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomPanelTab {
    Terminal,
    Problems,
    Output,
}

pub struct OxideApp {
    pub editors: Vec<Editor>,
    pub active_editor_idx: Option<usize>,
    pub workspace: Workspace,
    pub settings: EditorSettings,
    pub theme: Theme,
    pub commands: CommandRegistry,
    pub command_palette: CommandPalette,
    pub active_sidebar: Option<ActiveSidebar>,
    pub bottom_panel_open: bool,
    pub active_bottom_tab: BottomPanelTab,
    pub search_view: SearchView,
    pub git_view: GitView,
    pub git_status: GitStatus,
    pub gutter_markers: HashMap<usize, GutterMarker>,
    pub diagnostics: DiagnosticStore,
    pub terminal_process: Option<TerminalProcess>,
}

impl OxideApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load default dark modern theme
        let theme = Theme::dark_modern();
        let settings = EditorSettings::default();

        let mut workspace = Workspace::new();
        // If current dir has files, open it as workspace root
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        workspace.open_folder(current_dir.clone());

        let git_status = GitRepo::query_status(&current_dir);

        // Spawn integrated terminal
        let terminal_process = TerminalProcess::spawn(24, 80).ok();

        // Open an initial welcoming scratchpad buffer
        let welcome_code = r#"// Welcome to Oxide Code Editor
// High-Performance Rust-Native Development Environment

fn main() {
    println!("Hello from Oxide!");
    let features = vec![
        "Sub-millisecond latency",
        "Ropey B-Tree Text Buffer",
        "Incremental Syntax Highlighting",
        "LSP & DAP Ready",
        "Integrated PTY Terminal",
    ];

    for feat in features {
        println!("🚀 {}", feat);
    }
}
"#;
        let initial_editor = Editor::from_str(welcome_code, None, "rust");

        Self {
            editors: vec![initial_editor],
            active_editor_idx: Some(0),
            workspace,
            settings,
            theme,
            commands: CommandRegistry::with_default_commands(),
            command_palette: CommandPalette::new(),
            active_sidebar: Some(ActiveSidebar::Explorer),
            bottom_panel_open: false,
            active_bottom_tab: BottomPanelTab::Terminal,
            search_view: SearchView::default(),
            git_view: GitView::default(),
            git_status,
            gutter_markers: HashMap::new(),
            diagnostics: DiagnosticStore::new(),
            terminal_process,
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        // Check if already open
        if let Some(pos) = self.editors.iter().position(|e| e.file_path.as_deref() == Some(path)) {
            self.active_editor_idx = Some(pos);
            return;
        }

        if let Ok((content, _)) = FileSystem::read_file(path) {
            let lang = syntax::Language::from_path(path).id();
            let editor = Editor::from_str(&content, Some(path.to_path_buf()), lang);
            self.editors.push(editor);
            self.active_editor_idx = Some(self.editors.len() - 1);
        }
    }

    pub fn save_current_file(&mut self) {
        if let Some(idx) = self.active_editor_idx {
            if let Some(ed) = self.editors.get_mut(idx) {
                if let Some(ref path) = ed.file_path {
                    let content = ed.buffer.to_string();
                    if FileSystem::write_file_atomic(path, &content, LineEnding::Lf).is_ok() {
                        ed.buffer.mark_saved();
                    }
                }
            }
        }
    }

    pub fn execute_command(&mut self, cmd: &Command) {
        match cmd.id.as_str() {
            "file.save" => self.save_current_file(),
            "view.command_palette" => self.command_palette.open(),
            "view.toggle_sidebar" => {
                self.active_sidebar = if self.active_sidebar.is_some() {
                    None
                } else {
                    Some(ActiveSidebar::Explorer)
                };
            }
            "view.toggle_terminal" => {
                self.bottom_panel_open = !self.bottom_panel_open;
                self.active_bottom_tab = BottomPanelTab::Terminal;
            }
            "view.toggle_problems" => {
                self.bottom_panel_open = !self.bottom_panel_open;
                self.active_bottom_tab = BottomPanelTab::Problems;
            }
            "file.new" => {
                let ed = Editor::new();
                self.editors.push(ed);
                self.active_editor_idx = Some(self.editors.len() - 1);
            }
            "file.close_tab" => {
                if let Some(idx) = self.active_editor_idx {
                    self.editors.remove(idx);
                    if self.editors.is_empty() {
                        self.active_editor_idx = None;
                    } else if idx >= self.editors.len() {
                        self.active_editor_idx = Some(self.editors.len() - 1);
                    }
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for OxideApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Global Keyboard Shortcuts
        ctx.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl {
                if i.modifiers.shift && i.key_pressed(egui::Key::P) {
                    self.command_palette.open();
                } else if i.key_pressed(egui::Key::S) {
                    self.save_current_file();
                } else if i.key_pressed(egui::Key::B) {
                    self.active_sidebar = if self.active_sidebar.is_some() {
                        None
                    } else {
                        Some(ActiveSidebar::Explorer)
                    };
                }
            }
        });

        // 1. Status Bar (at bottom)
        egui::Panel::bottom("status_bar")
            .exact_size(22.0)
            .show(ui, |ui| {
                let current_editor = self.active_editor_idx.and_then(|idx| self.editors.get(idx));
                StatusBar::render(
                    ui,
                    current_editor,
                    &self.diagnostics,
                    &self.git_status.branch,
                    &self.theme,
                );
            });

        // 2. Activity Bar (narrow leftmost icons column)
        egui::Panel::left("activity_bar")
            .exact_size(48.0)
            .frame(egui::Frame::NONE.fill(to_egui_color(self.theme.tab_bg)))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    let exp_active = self.active_sidebar == Some(ActiveSidebar::Explorer);
                    if ui.selectable_label(exp_active, "📁").clicked() {
                        self.active_sidebar = if exp_active { None } else { Some(ActiveSidebar::Explorer) };
                    }
                    let srch_active = self.active_sidebar == Some(ActiveSidebar::Search);
                    if ui.selectable_label(srch_active, "🔍").clicked() {
                        self.active_sidebar = if srch_active { None } else { Some(ActiveSidebar::Search) };
                    }
                    let git_active = self.active_sidebar == Some(ActiveSidebar::SourceControl);
                    if ui.selectable_label(git_active, "🌿").clicked() {
                        self.active_sidebar = if git_active { None } else { Some(ActiveSidebar::SourceControl) };
                    }
                    let ext_active = self.active_sidebar == Some(ActiveSidebar::Extensions);
                    if ui.selectable_label(ext_active, "🧩").clicked() {
                        self.active_sidebar = if ext_active { None } else { Some(ActiveSidebar::Extensions) };
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        let set_active = self.active_sidebar == Some(ActiveSidebar::Settings);
                        if ui.selectable_label(set_active, "⚙").clicked() {
                            self.active_sidebar = if set_active { None } else { Some(ActiveSidebar::Settings) };
                        }
                    });
                });
            });

        // 3. Primary Sidebar (collapsible)
        if let Some(sidebar) = self.active_sidebar {
            egui::Panel::left("primary_sidebar")
                .resizable(true)
                .default_size(240.0)
                .frame(
                    egui::Frame::NONE
                        .fill(to_egui_color(self.theme.sidebar_bg))
                        .inner_margin(8),
                )
                .show(ui, |ui| {
                    match sidebar {
                        ActiveSidebar::Explorer => {
                            let mut opened_file: Option<PathBuf> = None;
                            ExplorerView::render(ui, &mut self.workspace.file_trees, &self.theme, |path| {
                                opened_file = Some(path.to_path_buf());
                            });
                            if let Some(p) = opened_file {
                                self.open_file(&p);
                            }
                        }
                        ActiveSidebar::Search => {
                            let mut jump_target: Option<(PathBuf, usize)> = None;
                            self.search_view.render(ui, &self.workspace.roots, &self.theme, |p, line| {
                                jump_target = Some((p.to_path_buf(), line));
                            });
                            if let Some((p, line)) = jump_target {
                                self.open_file(&p);
                                if let Some(ed) = self.active_editor_idx.and_then(|idx| self.editors.get_mut(idx)) {
                                    ed.scroll_line = line.saturating_sub(1) as f32;
                                    let char_pos = ed.buffer.line_to_char(line.saturating_sub(1));
                                    ed.selections.set_cursors(vec![editor_core::Cursor::new(char_pos)]);
                                }
                            }
                        }
                        ActiveSidebar::SourceControl => {
                            let mut diff_file: Option<PathBuf> = None;
                            self.git_view.render(ui, &self.git_status, &self.theme, |path| {
                                diff_file = Some(path.to_path_buf());
                            });
                            if let Some(p) = diff_file {
                                self.open_file(&p);
                            }
                        }
                        ActiveSidebar::Extensions => {
                            ui.heading(egui::RichText::new("EXTENSIONS").size(11.0).strong());
                            ui.label("Installed Extensions (0)");
                            ui.separator();
                            ui.label("VS Code compatible plugin engine ready.");
                        }
                        ActiveSidebar::Settings => {
                            ui.heading(egui::RichText::new("SETTINGS").size(11.0).strong());
                            ui.add(egui::Slider::new(&mut self.settings.font_size, 10.0..=28.0).text("Font Size"));
                            ui.add(egui::Slider::new(&mut self.settings.tab_size, 2..=8).text("Tab Size"));
                            ui.checkbox(&mut self.settings.line_numbers, "Line Numbers");
                            ui.checkbox(&mut self.settings.minimap_enabled, "Minimap");
                            ui.separator();
                            ui.label("Theme:");
                            if ui.selectable_label(self.theme.name == "Dark Modern", "Dark Modern").clicked() {
                                self.theme = Theme::dark_modern();
                            }
                            if ui.selectable_label(self.theme.name == "One Dark Pro", "One Dark Pro").clicked() {
                                self.theme = Theme::one_dark();
                            }
                            if ui.selectable_label(self.theme.name == "Light Modern", "Light Modern").clicked() {
                                self.theme = Theme::light_modern();
                            }
                        }
                    }
                });
        }

        // 4. Bottom Drawer Panel (Terminal, Problems, Output)
        if self.bottom_panel_open {
            egui::Panel::bottom("bottom_panel")
                .resizable(true)
                .default_size(200.0)
                .frame(
                    egui::Frame::NONE
                        .fill(to_egui_color(self.theme.background))
                        .inner_margin(6),
                )
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let term_sel = self.active_bottom_tab == BottomPanelTab::Terminal;
                        if ui.selectable_label(term_sel, "TERMINAL").clicked() {
                            self.active_bottom_tab = BottomPanelTab::Terminal;
                        }
                        let prob_sel = self.active_bottom_tab == BottomPanelTab::Problems;
                        if ui.selectable_label(prob_sel, format!("PROBLEMS ({})", self.diagnostics.total_errors() + self.diagnostics.total_warnings())).clicked() {
                            self.active_bottom_tab = BottomPanelTab::Problems;
                        }
                        let out_sel = self.active_bottom_tab == BottomPanelTab::Output;
                        if ui.selectable_label(out_sel, "OUTPUT").clicked() {
                            self.active_bottom_tab = BottomPanelTab::Output;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("×").clicked() {
                                self.bottom_panel_open = false;
                            }
                        });
                    });

                    ui.separator();

                    match self.active_bottom_tab {
                        BottomPanelTab::Terminal => {
                            if let Some(ref mut proc) = self.terminal_process {
                                TerminalView::render(ui, proc, &self.theme);
                            } else {
                                ui.label("Terminal failed to initialize PTY.");
                            }
                        }
                        BottomPanelTab::Problems => {
                            let mut jump_target: Option<(PathBuf, usize)> = None;
                            ProblemsView::render(ui, &self.diagnostics, &self.theme, |path, line| {
                                jump_target = Some((path.to_path_buf(), line));
                            });
                            if let Some((p, line)) = jump_target {
                                self.open_file(&p);
                                if let Some(ed) = self.active_editor_idx.and_then(|idx| self.editors.get_mut(idx)) {
                                    ed.scroll_line = line.saturating_sub(1) as f32;
                                    let char_pos = ed.buffer.line_to_char(line.saturating_sub(1));
                                    ed.selections.set_cursors(vec![editor_core::Cursor::new(char_pos)]);
                                }
                            }
                        }
                        BottomPanelTab::Output => {
                            ui.label("Oxide core initialized. 0 crashes, 0 errors.");
                        }
                    }
                });
        }

        // 5. Central Editor Area
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(to_egui_color(self.theme.background)))
            .show(ui, |ui| {
                // Tab bar
                let open_paths: Vec<PathBuf> = self
                    .editors
                    .iter()
                    .map(|e| e.file_path.clone().unwrap_or_else(|| PathBuf::from("Untitled")))
                    .collect();

                let is_dirty = self
                    .active_editor_idx
                    .and_then(|idx| self.editors.get(idx))
                    .map(|e| e.buffer.is_dirty())
                    .unwrap_or(false);

                let mut selected_tab: Option<usize> = None;
                let mut closed_tab: Option<usize> = None;

                TabBar::render(
                    ui,
                    &open_paths,
                    self.active_editor_idx,
                    is_dirty,
                    &self.theme,
                    &mut |idx| selected_tab = Some(idx),
                    &mut |idx| closed_tab = Some(idx),
                );

                if let Some(idx) = selected_tab {
                    self.active_editor_idx = Some(idx);
                }

                if let Some(idx) = closed_tab {
                    self.editors.remove(idx);
                    if self.editors.is_empty() {
                        self.active_editor_idx = None;
                    } else if idx >= self.editors.len() {
                        self.active_editor_idx = Some(self.editors.len() - 1);
                    }
                }

                ui.separator();

                // Active Editor Rendering
                if let Some(idx) = self.active_editor_idx {
                    if let Some(ed) = self.editors.get_mut(idx) {
                        EditorView::render(
                            ui,
                            ed,
                            &self.settings,
                            &self.theme,
                            &self.gutter_markers,
                        );
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("No files open.\nPress Ctrl+Shift+P for Command Palette, or choose a file from the Explorer.")
                                .color(to_egui_color(self.theme.gutter_fg)),
                        );
                    });
                }
            });

        // 6. Command Palette Modal Overlay
        let mut executed_cmd: Option<Command> = None;
        self.command_palette.render(&ctx, &self.commands, &self.theme, |cmd| {
            executed_cmd = Some(cmd.clone());
        });

        if let Some(cmd) = executed_cmd {
            self.execute_command(&cmd);
        }
    }
}
