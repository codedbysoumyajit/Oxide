use egui::{pos2, vec2, Color32, Event, FontId, Key, Rect, Sense, Ui};
use settings::Theme;
use terminal::TerminalProcess;

pub struct TerminalView;

impl TerminalView {
    pub fn render(
        ui: &mut Ui,
        process: &mut TerminalProcess,
        _theme: &Theme,
    ) {
        let available_size = ui.available_size();
        let (response, painter) =
            ui.allocate_painter(available_size, Sense::click_and_drag());

        let rect = response.rect;
        let font_id = FontId::monospace(13.0);
        let char_width = 7.8;
        let row_height = 16.0;

        // Background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(15, 15, 15));

        let screen = process.screen.lock();
        let max_visible_rows = (rect.height() / row_height).floor() as usize;

        for (r_idx, row) in screen.grid.iter().take(max_visible_rows).enumerate() {
            let y = rect.min.y + r_idx as f32 * row_height;
            for (c_idx, cell) in row.iter().enumerate() {
                if cell.ch == ' ' && cell.bg.is_none() {
                    continue;
                }
                let x = rect.min.x + c_idx as f32 * char_width;
                let cell_rect = Rect::from_min_size(pos2(x, y), vec2(char_width, row_height));

                if let Some((r, g, b)) = cell.bg {
                    painter.rect_filled(cell_rect, 0.0, Color32::from_rgb(r, g, b));
                }

                let fg_color = if let Some((r, g, b)) = cell.fg {
                    Color32::from_rgb(r, g, b)
                } else {
                    Color32::from_rgb(220, 220, 220)
                };

                painter.text(
                    pos2(x, y),
                    egui::Align2::LEFT_TOP,
                    cell.ch.to_string(),
                    font_id.clone(),
                    fg_color,
                );
            }
        }

        // Draw cursor
        let cursor_x = rect.min.x + screen.cursor_col as f32 * char_width;
        let cursor_y = rect.min.y + screen.cursor_row as f32 * row_height;
        let cursor_rect = Rect::from_min_size(pos2(cursor_x, cursor_y), vec2(char_width, row_height));
        painter.rect_filled(cursor_rect, 0.0, Color32::from_rgba_premultiplied(255, 255, 255, 120));

        drop(screen);

        // Forward keyboard input into PTY
        ui.input(|i| {
            for event in &i.events {
                match event {
                    Event::Text(text) => {
                        let _ = process.write_input(text);
                    }
                    Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        if modifiers.ctrl && *key == Key::C {
                            let _ = process.write_input("\x03");
                        } else {
                            match key {
                                Key::Enter => {
                                    let _ = process.write_input("\r");
                                }
                                Key::Backspace => {
                                    let _ = process.write_input("\x08");
                                }
                                Key::Tab => {
                                    let _ = process.write_input("\t");
                                }
                                Key::ArrowUp => {
                                    let _ = process.write_input("\x1b[A");
                                }
                                Key::ArrowDown => {
                                    let _ = process.write_input("\x1b[B");
                                }
                                Key::ArrowRight => {
                                    let _ = process.write_input("\x1b[C");
                                }
                                Key::ArrowLeft => {
                                    let _ = process.write_input("\x1b[D");
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
