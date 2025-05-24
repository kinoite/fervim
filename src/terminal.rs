use crossterm::{
    event::{self, Event}, // Removed unused KeyCode, KeyEvent, KeyEventKind
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        Clear, ClearType, size,
    },
    cursor::{Hide, Show, MoveTo},
    ExecutableCommand,
};
use std::io::{self, stdout, Write};

pub struct Terminal;

impl Terminal {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(Hide)?;
        Ok(Terminal)
    }

    pub fn clear_screen(&self) -> io::Result<()> {
        stdout().execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn cursor_position(&self, x: u16, y: u16) -> io::Result<()> {
        stdout().execute(MoveTo(x, y))?;
        Ok(())
    }

    pub fn print_line(&self, x: u16, y: u16, text: &str) -> io::Result<()> {
        self.cursor_position(x, y)?;
        write!(stdout(), "{}", text)?;
        Ok(())
    }

    pub fn flush(&self) -> io::Result<()> {
        stdout().flush()?;
        Ok(())
    }

    pub fn read_event(&self) -> io::Result<Event> {
        event::read()
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        size()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = stdout().execute(Show);
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

