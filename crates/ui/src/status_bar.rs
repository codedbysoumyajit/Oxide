use egui::{Margin, RichText, Ui};
use editor_core::Editor;
use language_server::DiagnosticStore;
use settings::Theme;
use crate::theme::to_egui_color;

pub struct StatusBar;

impl StatusBar {
    pub fn render(
        ui: &mut Ui,
        editor: Option<&Editor>,
        diagnostics: &DiagnosticStore,
        git_branch: &str,
        theme: &Theme,
    ) {
        let (bg, fg) = (to_egui_color(theme.statusbar_bg), to_egui_color(theme.statusbar_fg));

        egui::Frame::NONE
            .fill(bg)
            .inner_margin(Margin {
                left: 8,
                right: 8,
                top: 3,
                bottom: 3,
            })
            .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left items
                ui.label(RichText::new(format!("🌿 {}", git_branch)).color(fg).size(11.0));
                ui.add_space(8.0);

                let errs = diagnostics.total_errors();
                let warns = diagnostics.total_warnings();
                ui.label(
                    RichText::new(format!("❌ {} ⚠ {}", errs, warns))
                        .color(fg)
                        .size(11.0),
                );

                // Right items
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ed) = editor {
                        ui.label(RichText::new(&ed.language_id).color(fg).size(11.0));
                        ui.add_space(8.0);
                        ui.label(RichText::new("UTF-8").color(fg).size(11.0));
                        ui.add_space(8.0);
                        ui.label(RichText::new("LF").color(fg).size(11.0));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("Spaces: {}", ed.indent_config.tab_size))
                                .color(fg)
                                .size(11.0),
                        );
                        ui.add_space(8.0);

                        let (line, col) = ed
                            .buffer
                            .char_to_line_col(ed.selections.primary().head);
                        ui.label(
                            RichText::new(format!("Ln {}, Col {}", line + 1, col + 1))
                                .color(fg)
                                .size(11.0),
                        );
                    }
                });
            });
        });
    }
}
