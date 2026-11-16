use std::path::Path;
use egui::{RichText, Sense, Ui};
use settings::Theme;
use workspace::FileNode;
use crate::theme::to_egui_color;

pub struct ExplorerView;

impl ExplorerView {
    pub fn render(
        ui: &mut Ui,
        roots: &mut [FileNode],
        theme: &Theme,
        mut on_open_file: impl FnMut(&Path),
    ) {
        ui.vertical(|ui| {
            ui.heading(
                RichText::new("EXPLORER")
                    .size(11.0)
                    .strong()
                    .color(to_egui_color(theme.gutter_fg)),
            );
            ui.add_space(4.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for node in roots.iter_mut() {
                    Self::render_node(ui, node, 0, theme, &mut on_open_file);
                }
            });
        });
    }

    fn render_node(
        ui: &mut Ui,
        node: &mut FileNode,
        depth: usize,
        theme: &Theme,
        on_open_file: &mut impl FnMut(&Path),
    ) {
        let indent = depth as f32 * 14.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);
            if node.is_dir {
                let icon = if node.is_expanded { "▼ 📁" } else { "▶ 📁" };
                let label = format!("{} {}", icon, node.name);
                let response = ui.add(
                    egui::Label::new(
                        RichText::new(label)
                            .color(to_egui_color(theme.foreground))
                            .size(12.0),
                    )
                    .sense(Sense::click()),
                );

                if response.clicked() {
                    node.is_expanded = !node.is_expanded;
                    if node.is_expanded && node.children.is_empty() {
                        node.populate_children();
                    }
                }
            } else {
                let label = format!("  📄 {}", node.name);
                let response = ui.add(
                    egui::Label::new(
                        RichText::new(label)
                            .color(to_egui_color(theme.foreground))
                            .size(12.0),
                    )
                    .sense(Sense::click()),
                );

                if response.clicked() {
                    on_open_file(&node.path);
                }
            }
        });

        if node.is_dir && node.is_expanded {
            for child in &mut node.children {
                Self::render_node(ui, child, depth + 1, theme, on_open_file);
            }
        }
    }
}
