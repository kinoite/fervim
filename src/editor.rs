use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::{self, Write};
use std::fs;
use crossterm::cursor::SetCursorStyle;
use crossterm::ExecutableCommand;

pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    scroll_offset_y: usize,
    mode: Mode,
    command_input: String,
    filename: Option<String>,
    dirty: bool,
    message: String,
}

impl Editor {
    pub fn new(filename: Option<String>) -> io::Result<Self> {
        let mut lines = Vec::new();
        let mut dirty = false;
        let mut message = String::new();

        if let Some(ref path) = filename {
            match fs::read_to_string(path) {
                Ok(content) => {
                    lines = content.lines().map(|s| s.to_string()).collect();
                    if lines.is_empty() {
                        lines.push(String::new());
                    }
                }
                Err(e) => {
                    lines.push(String::new());
                    dirty = true;
                    message = format!("Error reading file {}: {}", path, e);
                }
            }
        } else {
            lines.push(String::new());
            dirty = true;
        }

        Ok(Editor {
            lines,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset_y: 0,
            mode: Mode::Normal,
            command_input: String::new(),
            filename,
            dirty,
            message,
        })
    }

    pub fn draw(&self, terminal: &mut crate::terminal::Terminal) -> io::Result<()> {
        terminal.clear_screen()?;
        let (width, height) = terminal.size()?;

        let mut display_y = 0;
        let text_area_height = height - 2;

        for line_index in self.scroll_offset_y..self.lines.len() {
            if display_y >= text_area_height as usize {
                break;
            }
            let line = &self.lines[line_index];
            let display_line = if line.len() > width as usize {
                &line[..width as usize]
            } else {
                line
            };
            terminal.print_line(0, display_y as u16, display_line)?;
            display_y += 1;
        }

        let status_bar_y = height - 2;
        terminal.print_line(0, status_bar_y, &"─".repeat(width as usize))?;
        let mode_text = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        };
        let file_status = self.filename.as_deref().unwrap_or("[No Name]");
        let dirty_indicator = if self.dirty { " [Modified]" } else { "" };
        let status_text = format!("{} | {}{}", mode_text, file_status, dirty_indicator);
        terminal.print_line(0, status_bar_y + 1, &status_text)?;

        match self.mode {
            Mode::Command => {
                let box_width = (width as f32 * 0.6).min(80.0).max(40.0) as u16;
                let box_height = 5;

                let start_x = (width / 2).saturating_sub(box_width / 2);
                let start_y = (height / 2).saturating_sub(box_height / 2);

                for y in start_y..(start_y + box_height) {
                    terminal.cursor_position(start_x, y)?;
                    write!(std::io::stdout(), "{:width$}", "", width = box_width as usize)?;
                }

                terminal.print_line(start_x, start_y, &format!("┌{}┐", "─".repeat((box_width as usize).saturating_sub(2))))?;
                for y in (start_y + 1)..(start_y + box_height - 1) {
                    terminal.print_line(start_x, y, &format!("│{:width$}│", "", width = (box_width as usize).saturating_sub(2)))?;
                }
                terminal.print_line(start_x, start_y + box_height - 1, &format!("└{}┘", "─".repeat((box_width as usize).saturating_sub(2))))?;

                let label = " Command box ";
                let label_x = start_x + (box_width / 2).saturating_sub((label.len() / 2) as u16);
                terminal.print_line(label_x, start_y, label)?;

                let command_line_y = start_y + 2;
                let command_prompt = format!(":{}", self.command_input);
                let display_command = if command_prompt.len() > (box_width as usize).saturating_sub(2) {
                    &command_prompt[..=(box_width as usize).saturating_sub(2)]
                } else {
                    &command_prompt
                };
                terminal.print_line(start_x + 1, command_line_y, display_command)?;

                if !self.message.is_empty() {
                    let message_y = start_y + 3;
                    let display_message = if self.message.len() > (box_width as usize).saturating_sub(2) {
                        &self.message[..=(box_width as usize).saturating_sub(2)]
                    } else {
                        &self.message
                    };
                    terminal.print_line(start_x + 1, message_y, display_message)?;
                }

                std::io::stdout().execute(SetCursorStyle::BlinkingBar)?;
                terminal.cursor_position(start_x + 1 + self.command_input.len() as u16, command_line_y)?;

            }
            Mode::Normal => {
                std::io::stdout().execute(SetCursorStyle::BlinkingBlock)?;
                terminal.cursor_position(self.cursor_x as u16, (self.cursor_y - self.scroll_offset_y) as u16)?;
                if !self.message.is_empty() {
                    terminal.print_line(0, height - 3, &self.message)?;
                }
            }
            Mode::Insert => {
                std::io::stdout().execute(SetCursorStyle::BlinkingBar)?;
                terminal.cursor_position(self.cursor_x as u16, (self.cursor_y - self.scroll_offset_y) as u16)?;
                if !self.message.is_empty() {
                    terminal.print_line(0, height - 3, &self.message)?;
                }
            }
        }

        terminal.flush()?;
        Ok(())
    }

    pub fn handle_key_event(&mut self, event: KeyEvent, terminal: &mut crate::terminal::Terminal) -> io::Result<bool> {
        let (_, height) = terminal.size()?;
        let text_area_height = height - 2;

        match self.mode {
            Mode::Normal => self.handle_normal_mode_key(event, text_area_height),
            Mode::Insert => self.handle_insert_mode_key(event, text_area_height),
            Mode::Command => self.handle_command_mode_key(event),
        }
    }

    fn handle_normal_mode_key(&mut self, event: KeyEvent, text_area_height: u16) -> io::Result<bool> {
        self.message.clear();
        match event.code {
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('h') | KeyCode::Left => self.move_cursor_left(),
            KeyCode::Char('l') | KeyCode::Right => self.move_cursor_right(),
            KeyCode::Char('k') | KeyCode::Up => self.move_cursor_up(),
            KeyCode::Char('j') | KeyCode::Down => self.move_cursor_down(),
            KeyCode::Char('^') => self.cursor_x = 0,
            KeyCode::Char('$') => self.cursor_x = self.lines[self.cursor_y].len(),
            KeyCode::Char('G') => {
                self.cursor_y = self.lines.len() - 1;
                self.cursor_x = self.lines[self.cursor_y].len();
                self.adjust_scroll(text_area_height);
            },
            KeyCode::Char('g') => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.cursor_y = 0;
                    self.cursor_x = 0;
                    self.adjust_scroll(text_area_height);
                }
            },
            KeyCode::PageUp => self.page_up(text_area_height),
            KeyCode::PageDown => self.page_down(text_area_height),
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_input.clear();
            },
            KeyCode::Char('d') => {
                // Placeholder for delete line (dd)
                // Need to read next key for 'd'
            },
            KeyCode::Esc => {
                self.command_input.clear();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(true)
    }

    fn handle_insert_mode_key(&mut self, event: KeyEvent, text_area_height: u16) -> io::Result<bool> {
        self.message.clear();
        match event.code {
            KeyCode::Char(c) => {
                self.lines[self.cursor_y].insert(self.cursor_x, c);
                self.cursor_x += 1;
                self.dirty = true;
            }
            KeyCode::Backspace => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    self.lines[self.cursor_y].remove(self.cursor_x);
                    self.dirty = true;
                } else if self.cursor_y > 0 {
                    let current_line = self.lines.remove(self.cursor_y);
                    self.cursor_y -= 1;
                    self.cursor_x = self.lines[self.cursor_y].len();
                    self.lines[self.cursor_y].push_str(&current_line);
                    self.dirty = true;
                }
            }
            KeyCode::Delete => {
                if self.cursor_x < self.lines[self.cursor_y].len() {
                    self.lines[self.cursor_y].remove(self.cursor_x);
                    self.dirty = true;
                } else if self.cursor_y + 1 < self.lines.len() {
                    let next_line = self.lines.remove(self.cursor_y + 1);
                    self.lines[self.cursor_y].push_str(&next_line);
                    self.dirty = true;
                }
            }
            KeyCode::Enter => {
                let new_line = self.lines[self.cursor_y].split_off(self.cursor_x);
                self.cursor_y += 1;
                self.lines.insert(self.cursor_y, new_line);
                self.cursor_x = 0;
                self.dirty = true;
            }
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Up => self.move_cursor_up(),
            KeyCode::Down => self.move_cursor_down(),
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.adjust_cursor_to_line_end();
            }
            _ => {}
        }
        self.adjust_scroll(text_area_height);
        Ok(true)
    }

    fn handle_command_mode_key(&mut self, event: KeyEvent) -> io::Result<bool> {
        self.message.clear();
        match event.code {
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Enter => {
                let command = self.command_input.trim().to_string();
                let should_continue = self.execute_command(&command)?;
                self.command_input.clear();
                self.mode = Mode::Normal;
                return Ok(should_continue);
            }
            KeyCode::Esc => {
                self.command_input.clear();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(true)
    }

    fn execute_command(&mut self, command: &str) -> io::Result<bool> {
        match command {
            "q" => {
                if self.dirty {
                    self.message = "No write since last change (add ! to override)".to_string();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            "q!" => Ok(false),
            "w" => {
                self.save_file()?;
                self.message = "File written.".to_string();
                Ok(true)
            }
            "wq" => {
                self.save_file()?;
                Ok(false)
            }
            _ => {
                self.message = format!("Unknown command: {}", command);
                Ok(true)
            }
        }
    }

    fn save_file(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.filename {
            let content = self.lines.join("\n");
            fs::write(path, content)?;
            self.dirty = false;
            Ok(())
        } else {
            self.message = "No filename. Use :w <filename> to save.".to_string();
            Err(io::Error::new(io::ErrorKind::Other, "No filename"))
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_x < self.lines[self.cursor_y].len() {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.adjust_cursor_to_line_end();
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.adjust_cursor_to_line_end();
        }
    }

    fn page_up(&mut self, text_area_height: u16) {
        self.cursor_y = self.cursor_y.saturating_sub(text_area_height as usize);
        self.adjust_scroll(text_area_height);
        self.adjust_cursor_to_line_end();
    }

    fn page_down(&mut self, text_area_height: u16) {
        self.cursor_y = (self.cursor_y + text_area_height as usize).min(self.lines.len() - 1);
        self.adjust_scroll(text_area_height);
        self.adjust_cursor_to_line_end();
    }

    fn adjust_cursor_to_line_end(&mut self) {
        let current_line_len = self.lines[self.cursor_y].len();
        if self.cursor_x > current_line_len {
            self.cursor_x = current_line_len;
        }
    }

    pub fn adjust_scroll(&mut self, text_area_height: u16) {
        if self.cursor_y < self.scroll_offset_y {
            self.scroll_offset_y = self.cursor_y;
        }
        if self.cursor_y >= self.scroll_offset_y + text_area_height as usize {
            self.scroll_offset_y = self.cursor_y - text_area_height as usize + 1;
        }
    }
}

