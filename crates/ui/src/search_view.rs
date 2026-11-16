use std::path::{Path, PathBuf};
use egui::{Color32, RichText, Sense, Ui};
use search::{SearchEngine, SearchQuery, SearchResultMatch};
use settings::Theme;
use crate::theme::to_egui_color;

#[derive(Default)]
pub struct SearchView {
    pub query: String,
    pub is_regex: bool,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub results: Vec<SearchResultMatch>,
}

impl SearchView {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        roots: &[PathBuf],
        theme: &Theme,
        mut on_jump_to: impl FnMut(&Path, usize),
    ) {
        ui.vertical(|ui| {
            ui.heading(
                RichText::new("SEARCH")
                    .size(11.0)
                    .strong()
                    .color(to_egui_color(theme.gutter_fg)),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                let text_edit = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search across workspace...")
                        .desired_width(180.0),
                );

                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.perform_search(roots);
                }

                if ui.button("🔍").clicked() {
                    self.perform_search(roots);
                }
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.case_sensitive, "Aa");
                ui.checkbox(&mut self.whole_word, "W");
                ui.checkbox(&mut self.is_regex, ".*");
            });

            ui.separator();
            ui.label(
                RichText::new(format!("{} results found", self.results.len()))
                    .size(11.0)
                    .color(to_egui_color(theme.gutter_fg)),
            );

            egui::ScrollArea::vertical().show(ui, |ui| {
                for res in &self.results {
                    let file_name = res
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "File".to_string());

                    let title = format!("{}:{}", file_name, res.line_number);
                    let response = ui.horizontal(|ui| {
                        ui.label(RichText::new(title).strong().color(Color32::from_rgb(0, 122, 204)));
                        ui.label(
                            RichText::new(res.line_text.trim())
                                .color(to_egui_color(theme.foreground))
                                .size(11.0),
                        );
                    });

                    if response.response.interact(Sense::click()).clicked() {
                        on_jump_to(&res.path, res.line_number);
                    }
                }
            });
        });
    }

    fn perform_search(&mut self, roots: &[PathBuf]) {
        let q = SearchQuery {
            query: self.query.clone(),
            is_regex: self.is_regex,
            case_sensitive: self.case_sensitive,
            whole_word: self.whole_word,
            max_results: 500,
        };
        self.results = SearchEngine::search(roots, &q);
    }
}
