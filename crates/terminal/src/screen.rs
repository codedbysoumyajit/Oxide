use vte::{Params, Parser, Perform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<(u8, u8, u8)>,
    pub bg: Option<(u8, u8, u8)>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
        }
    }
}

pub struct TerminalScreen {
    pub rows: usize,
    pub cols: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub grid: Vec<Vec<Cell>>,
    pub scrollback: Vec<Vec<Cell>>,
    parser: Parser,
    current_fg: Option<(u8, u8, u8)>,
    current_bg: Option<(u8, u8, u8)>,
}

impl TerminalScreen {
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::default(); cols]; rows];
        Self {
            rows,
            cols,
            cursor_row: 0,
            cursor_col: 0,
            grid,
            scrollback: Vec::new(),
            parser: Parser::new(),
            current_fg: None,
            current_bg: None,
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        let mut performer = ScreenPerformer { screen: self };
        parser.advance(&mut performer, bytes);
        performer.screen.parser = parser;
    }

    pub fn newline(&mut self) {
        if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        } else {
            // Scroll up
            let top_line = self.grid.remove(0);
            self.scrollback.push(top_line);
            if self.scrollback.len() > 1000 {
                self.scrollback.remove(0);
            }
            self.grid.push(vec![Cell::default(); self.cols]);
        }
        self.cursor_col = 0;
    }
}

struct ScreenPerformer<'a> {
    screen: &'a mut TerminalScreen,
}

impl<'a> Perform for ScreenPerformer<'a> {
    fn print(&mut self, c: char) {
        if self.screen.cursor_col >= self.screen.cols {
            self.screen.newline();
        }
        let r = self.screen.cursor_row;
        let c_idx = self.screen.cursor_col;
        if r < self.screen.grid.len() && c_idx < self.screen.grid[r].len() {
            self.screen.grid[r][c_idx] = Cell {
                ch: c,
                fg: self.screen.current_fg,
                bg: self.screen.current_bg,
            };
            self.screen.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.screen.newline(),
            b'\r' => self.screen.cursor_col = 0,
            0x08 => {
                // Backspace
                self.screen.cursor_col = self.screen.cursor_col.saturating_sub(1);
            }
            0x09 => {
                // Tab
                self.screen.cursor_col = (self.screen.cursor_col + 8) / 8 * 8;
                if self.screen.cursor_col >= self.screen.cols {
                    self.screen.newline();
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'J' => {
                // Clear display
                for row in &mut self.screen.grid {
                    for cell in row {
                        *cell = Cell::default();
                    }
                }
                self.screen.cursor_row = 0;
                self.screen.cursor_col = 0;
            }
            'm' => {
                // SGR Color codes
                for param in params.iter() {
                    match param.first() {
                        Some(0) => {
                            self.screen.current_fg = None;
                            self.screen.current_bg = None;
                        }
                        Some(31) => self.screen.current_fg = Some((230, 80, 80)), // Red
                        Some(32) => self.screen.current_fg = Some((80, 200, 80)), // Green
                        Some(33) => self.screen.current_fg = Some((230, 200, 80)), // Yellow
                        Some(34) => self.screen.current_fg = Some((80, 140, 230)), // Blue
                        Some(35) => self.screen.current_fg = Some((200, 80, 230)), // Magenta
                        Some(36) => self.screen.current_fg = Some((80, 220, 220)), // Cyan
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
