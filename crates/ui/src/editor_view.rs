use std::collections::HashMap;
use egui::{
    pos2, vec2, Color32, Event, FontId, Key, Modifiers, Rect, Response,
    Sense, Ui,
};
use editor_core::{Cursor, Editor};
use git::GutterMarker;
use settings::{EditorSettings, Theme};
use syntax::{Highlighter, Language};

use crate::theme::{get_token_color, to_egui_color};

pub struct EditorView;

impl EditorView {
    pub fn render(
        ui: &mut Ui,
        editor: &mut Editor,
        settings: &EditorSettings,
        theme: &Theme,
        gutter_markers: &HashMap<usize, GutterMarker>,
    ) -> Response {
        let available_size = ui.available_size();
        let (response, painter) =
            ui.allocate_painter(available_size, Sense::click_and_drag());

        let rect = response.rect;
        let font_id = FontId::monospace(settings.font_size);
        let row_height = settings.font_size * settings.line_height;

        // Measure a representative character width
        let char_width = painter
            .layout_no_wrap("M".to_string(), font_id.clone(), Color32::WHITE)
            .size()
            .x
            .max(1.0);

        let gutter_width = if settings.line_numbers {
            // Dynamic gutter width based on number of digits in line count
            let digits = editor.buffer.len_lines().to_string().len().max(3);
            (digits as f32 + 2.5) * char_width
        } else {
            0.0
        };

        let minimap_width = if settings.minimap_enabled { 80.0 } else { 0.0 };
        let text_area_rect = Rect::from_min_max(
            pos2(rect.min.x + gutter_width, rect.min.y),
            pos2(rect.max.x - minimap_width, rect.max.y),
        );

        // Fill backgrounds
        painter.rect_filled(rect, 0.0, to_egui_color(theme.background));
        if settings.line_numbers {
            let gutter_rect = Rect::from_min_max(
                rect.min,
                pos2(rect.min.x + gutter_width, rect.max.y),
            );
            painter.rect_filled(gutter_rect, 0.0, to_egui_color(theme.gutter_bg));
        }

        // Virtualized line calculations
        let total_lines = editor.buffer.len_lines().max(1);
        let first_visible_line = (editor.scroll_line.floor() as usize).min(total_lines.saturating_sub(1));
        let visible_line_count = (rect.height() / row_height).ceil() as usize + 2;
        let last_visible_line = (first_visible_line + visible_line_count).min(total_lines);

        let language = Language::from_id(&editor.language_id);
        let primary_cursor = *editor.selections.primary();
        let (primary_line, _) = editor.buffer.char_to_line_col(primary_cursor.head);

        // Render visible lines
        for line_idx in first_visible_line..last_visible_line {
            let line_y = rect.min.y + (line_idx as f32 - editor.scroll_line) * row_height;
            let line_rect = Rect::from_min_size(
                pos2(text_area_rect.min.x, line_y),
                vec2(text_area_rect.width(), row_height),
            );

            // Active line highlight
            if line_idx == primary_line {
                painter.rect_filled(line_rect, 0.0, to_egui_color(theme.active_line_bg));
            }

            // Gutter: Line number and Git change marker
            if settings.line_numbers {
                let line_num_str = (line_idx + 1).to_string();
                let text_pos = pos2(
                    rect.min.x + gutter_width - (line_num_str.len() as f32 + 1.0) * char_width,
                    line_y + (row_height - settings.font_size) * 0.5,
                );
                let fg_color = if line_idx == primary_line {
                    to_egui_color(theme.foreground)
                } else {
                    to_egui_color(theme.gutter_fg)
                };
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_TOP,
                    &line_num_str,
                    font_id.clone(),
                    fg_color,
                );

                // Git marker on gutter edge
                if let Some(marker) = gutter_markers.get(&line_idx) {
                    let marker_color = match marker {
                        GutterMarker::Added => Color32::from_rgb(75, 175, 80),
                        GutterMarker::Modified => Color32::from_rgb(33, 150, 243),
                        GutterMarker::Deleted => Color32::from_rgb(244, 67, 54),
                    };
                    let marker_rect = Rect::from_min_max(
                        pos2(rect.min.x + gutter_width - 3.0, line_y),
                        pos2(rect.min.x + gutter_width, line_y + row_height),
                    );
                    painter.rect_filled(marker_rect, 0.0, marker_color);
                }
            }

            // Line content rendering
            if let Some(line_slice) = editor.buffer.line(line_idx) {
                let line_str = line_slice.to_string();
                let line_start_char = editor.buffer.line_to_char(line_idx);

                // Render selections on this line
                for cursor in editor.selections.cursors() {
                    if cursor.is_selection() {
                        let sel_start = cursor.start();
                        let sel_end = cursor.end();
                        let line_char_end = line_start_char + line_slice.len_chars();

                        if sel_start < line_char_end && sel_end > line_start_char {
                            let col_start = sel_start.saturating_sub(line_start_char);
                            let col_end = (sel_end - line_start_char).min(line_slice.len_chars());
                            let x_start = text_area_rect.min.x + col_start as f32 * char_width - editor.scroll_col;
                            let x_end = text_area_rect.min.x + col_end as f32 * char_width - editor.scroll_col;
                            let sel_rect = Rect::from_min_max(
                                pos2(x_start.max(text_area_rect.min.x), line_y),
                                pos2(x_end.min(text_area_rect.max.x), line_y + row_height),
                            );
                            painter.rect_filled(sel_rect, 0.0, to_egui_color(theme.selection_bg));
                        }
                    }
                }

                // Render syntax tokens
                let tokens = Highlighter::highlight_line(&line_str, language);
                for token in tokens {
                    let token_x = text_area_rect.min.x + token.range.start as f32 * char_width - editor.scroll_col;
                    let token_text: String = line_str.chars().skip(token.range.start).take(token.range.end - token.range.start).collect();
                    let token_pos = pos2(token_x, line_y + (row_height - settings.font_size) * 0.5);

                    if token_x < text_area_rect.max.x && token_x + (token_text.len() as f32 * char_width) > text_area_rect.min.x {
                        painter.text(
                            token_pos,
                            egui::Align2::LEFT_TOP,
                            token_text,
                            font_id.clone(),
                            get_token_color(theme, token.kind),
                        );
                    }
                }

                // Render Cursors on this line
                for cursor in editor.selections.cursors() {
                    let (c_line, c_col) = editor.buffer.char_to_line_col(cursor.head);
                    if c_line == line_idx {
                        let cursor_x = text_area_rect.min.x + c_col as f32 * char_width - editor.scroll_col;
                        let cursor_rect = Rect::from_min_size(
                            pos2(cursor_x, line_y),
                            vec2(2.0, row_height),
                        );
                        painter.rect_filled(cursor_rect, 0.0, to_egui_color(theme.cursor_color));
                    }
                }
            }
        }

        // Minimap rendering
        if settings.minimap_enabled {
            let minimap_rect = Rect::from_min_max(
                pos2(rect.max.x - minimap_width, rect.min.y),
                rect.max,
            );
            painter.rect_filled(minimap_rect, 0.0, to_egui_color(theme.gutter_bg));
            let mm_line_height = 2.0;
            let mm_max_lines = (minimap_rect.height() / mm_line_height) as usize;
            for idx in 0..total_lines.min(mm_max_lines) {
                if let Some(l) = editor.buffer.line(idx) {
                    let len = l.len_chars().min(60);
                    let y = minimap_rect.min.y + idx as f32 * mm_line_height;
                    let line_bar = Rect::from_min_size(
                        pos2(minimap_rect.min.x + 4.0, y),
                        vec2(len as f32 * 1.0, 1.0),
                    );
                    painter.rect_filled(line_bar, 0.0, to_egui_color(theme.gutter_fg));
                }
            }
        }

        // Handle Mouse Events (Click & Drag)
        if response.clicked() {
            if let Some(mouse_pos) = response.hover_pos() {
                if text_area_rect.contains(mouse_pos) {
                    let click_rel_y = mouse_pos.y - text_area_rect.min.y;
                    let clicked_line = (editor.scroll_line + (click_rel_y / row_height)).floor() as usize;
                    let click_rel_x = mouse_pos.x - text_area_rect.min.x + editor.scroll_col;
                    let clicked_col = (click_rel_x / char_width).round().max(0.0) as usize;

                    let char_idx = editor.buffer.line_col_to_char(clicked_line, clicked_col);
                    editor.selections.set_cursors(vec![Cursor::new(char_idx)]);
                }
            }
        }

        if response.dragged() {
            if let Some(mouse_pos) = response.hover_pos() {
                if text_area_rect.contains(mouse_pos) {
                    let click_rel_y = mouse_pos.y - text_area_rect.min.y;
                    let clicked_line = (editor.scroll_line + (click_rel_y / row_height)).floor() as usize;
                    let click_rel_x = mouse_pos.x - text_area_rect.min.x + editor.scroll_col;
                    let clicked_col = (click_rel_x / char_width).round().max(0.0) as usize;

                    let char_idx = editor.buffer.line_col_to_char(clicked_line, clicked_col);
                    let anchor = editor.selections.primary().anchor;
                    editor.selections.set_cursors(vec![Cursor::with_anchor(char_idx, anchor)]);
                }
            }
        }

        // Handle Scroll Wheel
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
        if scroll_delta.y != 0.0 {
            editor.scroll_line = (editor.scroll_line - (scroll_delta.y / row_height)).clamp(
                0.0,
                total_lines.saturating_sub(1) as f32,
            );
        }
        if scroll_delta.x != 0.0 {
            editor.scroll_col = (editor.scroll_col - scroll_delta.x).max(0.0);
        }

        // Handle Keyboard Events
        ui.input(|i| {
            for event in &i.events {
                match event {
                    Event::Text(text) => {
                        for ch in text.chars() {
                            if !ch.is_control() {
                                editor.insert_char(ch);
                            }
                        }
                    }
                    Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        Self::handle_key(editor, *key, *modifiers);
                    }
                    _ => {}
                }
            }
        });

        response
    }

    fn handle_key(editor: &mut Editor, key: Key, modifiers: Modifiers) {
        let select = modifiers.shift;

        if modifiers.command || modifiers.ctrl {
            match key {
                Key::A => editor.select_all(),
                Key::Z => {
                    if modifiers.shift {
                        editor.redo();
                    } else {
                        editor.undo();
                    }
                }
                Key::Y => editor.redo(),
                Key::ArrowUp => editor.add_cursor_above(),
                Key::ArrowDown => editor.add_cursor_below(),
                Key::ArrowLeft => editor.move_to_line_start(select),
                Key::ArrowRight => editor.move_to_line_end(select),
                _ => {}
            }
            return;
        }

        match key {
            Key::ArrowLeft => editor.move_left(select),
            Key::ArrowRight => editor.move_right(select),
            Key::ArrowUp => editor.move_up(select),
            Key::ArrowDown => editor.move_down(select),
            Key::Backspace => editor.backspace(),
            Key::Delete => editor.delete_forward(),
            Key::Enter => editor.insert_newline(),
            Key::Home => editor.move_to_line_start(select),
            Key::End => editor.move_to_line_end(select),
            Key::Tab => {
                editor.insert_text("    ");
            }
            _ => {}
        }
    }
}
