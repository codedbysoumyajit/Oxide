use commands::{Command, CommandRegistry};
use egui::{vec2, Align2, Color32, Key, RichText, Sense, Window};
use settings::Theme;
use crate::theme::to_egui_color;

pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub selected_idx: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_idx: 0,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_idx = 0;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        registry: &CommandRegistry,
        theme: &Theme,
        mut on_execute: impl FnMut(&Command),
    ) {
        if !self.is_open {
            return;
        }

        let mut execute_cmd: Option<Command> = None;
        let mut should_close = false;

        let results = registry.search(&self.query);

        Window::new("Command Palette")
            .anchor(Align2::CENTER_TOP, vec2(0.0, 50.0))
            .fixed_size(vec2(540.0, 360.0))
            .collapsible(false)
            .title_bar(false)
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(to_egui_color(theme.sidebar_bg))
                    .inner_margin(8),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(">").strong().color(Color32::from_rgb(0, 122, 204)));
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Type a command or search...")
                            .desired_width(f32::INFINITY),
                    );
                    response.request_focus();
                });

                ui.separator();

                // Handle arrow keys
                if ctx.input(|i| i.key_pressed(Key::ArrowDown)) && !results.is_empty() {
                    self.selected_idx = (self.selected_idx + 1) % results.len();
                }
                if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && !results.is_empty() {
                    self.selected_idx = (self.selected_idx + results.len() - 1) % results.len();
                }
                if ctx.input(|i| i.key_pressed(Key::Escape)) {
                    should_close = true;
                }
                if ctx.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some(cmd) = results.get(self.selected_idx) {
                        execute_cmd = Some((*cmd).clone());
                        should_close = true;
                    }
                }

                // Render list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, cmd) in results.iter().enumerate() {
                        let is_selected = i == self.selected_idx;
                        let item_bg = if is_selected {
                            to_egui_color(theme.selection_bg)
                        } else {
                            Color32::TRANSPARENT
                        };

                        let item_response = egui::Frame::NONE.fill(item_bg).inner_margin(4).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let text = format!("{}: {}", cmd.category, cmd.title);
                                ui.label(RichText::new(text).color(to_egui_color(theme.foreground)));

                                if let Some(ref kb) = cmd.keybinding {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(kb)
                                                .size(11.0)
                                                .color(to_egui_color(theme.gutter_fg)),
                                        );
                                    });
                                }
                            });
                        });

                        if item_response.response.interact(Sense::click()).clicked() {
                            execute_cmd = Some((*cmd).clone());
                            should_close = true;
                        }
                    }
                });
            });

        if let Some(cmd) = execute_cmd {
            on_execute(&cmd);
        }

        if should_close {
            self.close();
        }
    }
}
