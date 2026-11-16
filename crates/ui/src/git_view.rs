use std::path::Path;
use egui::{Color32, RichText, Sense, Ui};
use git::GitStatus;
use settings::Theme;
use crate::theme::to_egui_color;

#[derive(Default)]
pub struct GitView {
    pub commit_message: String,
}

impl GitView {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        status: &GitStatus,
        theme: &Theme,
        mut on_open_diff: impl FnMut(&Path),
    ) {
        ui.vertical(|ui| {
            ui.heading(
                RichText::new("SOURCE CONTROL")
                    .size(11.0)
                    .strong()
                    .color(to_egui_color(theme.gutter_fg)),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("🌿").size(12.0));
                ui.label(
                    RichText::new(&status.branch)
                        .strong()
                        .color(to_egui_color(theme.foreground)),
                );
            });

            ui.add_space(6.0);
            ui.add(
                egui::TextEdit::multiline(&mut self.commit_message)
                    .hint_text("Message (Ctrl+Enter to commit)")
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );

            if ui.button("✔ Commit").clicked() {
                self.commit_message.clear();
            }

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if !status.modified.is_empty() {
                    ui.label(
                        RichText::new(format!("MODIFIED ({})", status.modified.len()))
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(220, 180, 80)),
                    );

                    for path in &status.modified {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "File".to_string());

                        let response = ui.horizontal(|ui| {
                            ui.label(RichText::new("M").color(Color32::from_rgb(220, 180, 80)));
                            ui.label(RichText::new(name).color(to_egui_color(theme.foreground)));
                        });

                        if response.response.interact(Sense::click()).clicked() {
                            on_open_diff(path);
                        }
                    }
                }

                if !status.untracked.is_empty() {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("UNTRACKED ({})", status.untracked.len()))
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(80, 200, 80)),
                    );

                    for path in &status.untracked {
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "File".to_string());

                        let response = ui.horizontal(|ui| {
                            ui.label(RichText::new("U").color(Color32::from_rgb(80, 200, 80)));
                            ui.label(RichText::new(name).color(to_egui_color(theme.foreground)));
                        });

                        if response.response.interact(Sense::click()).clicked() {
                            on_open_diff(path);
                        }
                    }
                }
            });
        });
    }
}
