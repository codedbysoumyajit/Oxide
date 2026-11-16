use std::path::Path;
use egui::{Color32, RichText, Sense, Ui};
use language_server::{DiagnosticSeverity, DiagnosticStore};
use settings::Theme;
use crate::theme::to_egui_color;

pub struct ProblemsView;

impl ProblemsView {
    pub fn render(
        ui: &mut Ui,
        diagnostics: &DiagnosticStore,
        theme: &Theme,
        mut on_jump_to: impl FnMut(&Path, usize),
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("❌ {} Errors", diagnostics.total_errors()))
                        .strong()
                        .color(Color32::from_rgb(244, 67, 54)),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("⚠ {} Warnings", diagnostics.total_warnings()))
                        .strong()
                        .color(Color32::from_rgb(255, 193, 7)),
                );
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let all = diagnostics.all_diagnostics();
                if all.is_empty() {
                    ui.label(
                        RichText::new("No problems have been detected in the workspace.")
                            .italics()
                            .color(to_egui_color(theme.gutter_fg)),
                    );
                }

                for (path, diag) in all {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "File".to_string());

                    let (icon, color) = match diag.severity {
                        Some(DiagnosticSeverity::Error) => ("❌", Color32::from_rgb(244, 67, 54)),
                        Some(DiagnosticSeverity::Warning) => ("⚠", Color32::from_rgb(255, 193, 7)),
                        _ => ("ℹ", Color32::from_rgb(33, 150, 243)),
                    };

                    let line_num = diag.range.start.line as usize + 1;
                    let title = format!("{} [{}:{}]", file_name, line_num, diag.range.start.character + 1);

                    let item = ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).color(color));
                        ui.label(RichText::new(title).strong().color(Color32::from_rgb(0, 122, 204)));
                        ui.label(RichText::new(&diag.message).color(to_egui_color(theme.foreground)));
                    });

                    if item.response.interact(Sense::click()).clicked() {
                        on_jump_to(path, line_num);
                    }
                }
            });
        });
    }
}
