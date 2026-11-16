use std::path::PathBuf;
use egui::{vec2, Ui};
use settings::Theme;
use crate::theme::to_egui_color;

pub struct TabBar;

impl TabBar {
    pub fn render(
        ui: &mut Ui,
        open_files: &[PathBuf],
        active_idx: Option<usize>,
        is_dirty: bool,
        theme: &Theme,
        on_select: &mut impl FnMut(usize),
        on_close: &mut impl FnMut(usize),
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

            for (idx, path) in open_files.iter().enumerate() {
                let is_active = Some(idx) == active_idx;
                let bg_color = if is_active {
                    to_egui_color(theme.active_tab_bg)
                } else {
                    to_egui_color(theme.tab_bg)
                };

                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string());

                let display_title = if is_active && is_dirty {
                    format!("● {}", file_name)
                } else {
                    file_name
                };

                let text_color = if is_active {
                    to_egui_color(theme.foreground)
                } else {
                    to_egui_color(theme.gutter_fg)
                };

                let tab_btn = egui::Button::new(
                    egui::RichText::new(&display_title)
                        .color(text_color)
                        .size(12.0),
                )
                .fill(bg_color)
                .corner_radius(0);

                if ui.add(tab_btn).clicked() {
                    on_select(idx);
                }

                // Close button
                let close_btn = egui::Button::new(
                    egui::RichText::new("×").color(text_color).size(12.0),
                )
                .fill(bg_color)
                .corner_radius(0);

                if ui.add(close_btn).clicked() {
                    on_close(idx);
                }
            }
        });
    }
}
