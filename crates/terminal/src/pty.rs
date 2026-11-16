use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use thiserror::Error;

use crate::screen::TerminalScreen;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("PTY creation error: {0}")]
    Creation(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct TerminalProcess {
    writer: Box<dyn Write + Send>,
    pub screen: Arc<Mutex<TerminalScreen>>,
}

impl TerminalProcess {
    pub fn spawn(rows: u16, cols: u16) -> Result<Self, PtyError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::Creation(e.to_string()))?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::Creation(e.to_string()))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::Creation(e.to_string()))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::Creation(e.to_string()))?;

        let screen = Arc::new(Mutex::new(TerminalScreen::new(rows as usize, cols as usize)));
        let screen_clone = screen.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let mut scr = screen_clone.lock();
                scr.write_bytes(&buf[..n]);
            }
        });

        Ok(Self { writer, screen })
    }

    pub fn write_input(&mut self, text: &str) -> std::io::Result<()> {
        self.writer.write_all(text.as_bytes())?;
        self.writer.flush()
    }
}
