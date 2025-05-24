use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io::{self, Write};
use std::fs;
use crossterm::cursor::SetCursorStyle;
use crossterm::ExecutableCommand;
use crossterm::style::{SetForegroundColor, SetBackgroundColor, ResetColor, Color};

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
    config: crate::config::Config,
}

impl Editor {
    pub fn new(filename: Option<String>, config: crate::config::Config) -> io::Result<Self> {
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
            config,
        })
    }

    fn parse_color(color_str: &Option<String>) -> Option<Color> {
        color_str.as_ref().and_then(|s| {
            if s.starts_with('#') && s.len() == 7 {
                let r = u8::from_str_radix(&s[1..3], 16).ok()?;
                let g = u8::from_str_radix(&s[3..5], 16).ok()?;
                let b = u8::from_str_radix(&s[5..7], 16).ok()?;
                Some(Color::Rgb { r, g, b })
            } else {
                match s.to_lowercase().as_str() {
                    "black" => Some(Color::Black),
                    "red" => Some(Color::Red),
                    "green" => Some(Color::Green),
                    "yellow" => Some(Color::Yellow),
                    "blue" => Some(Color::Blue),
                    "magenta" => Some(Color::Magenta),
                    "cyan" => Some(Color::Cyan),
                    "white" => Some(Color::White),
                    "dark_grey" => Some(Color::DarkGrey),
                    "dark_red" => Some(Color::DarkRed),
                    "dark_green" => Some(Color::DarkGreen),
                    "dark_yellow" => Some(Color::DarkYellow),
                    "dark_blue" => Some(Color::DarkBlue),
                    "dark_magenta" => Some(Color::DarkMagenta),
                    "dark_cyan" => Some(Color::DarkCyan),
                    "grey" => Some(Color::Grey),
                    _ => None,
                }
            }
        })
    }

    fn lerp_color(color1: Color, color2: Color, t: f32) -> Color {
        let (r1, g1, b1) = match color1 {
            Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
            _ => (0.0, 0.0, 0.0),
        };
        let (r2, g2, b2) = match color2 {
            Color::Rgb { r, g, b } => (r as f32, g as f32, b as f32),
            _ => (0.0, 0.0, 0.0),
        };

        let r = (r1 + (r2 - r1) * t) as u8;
        let g = (g1 + (g2 - g1) * t) as u8;
        let b = (b1 + (b2 - b1) * t) as u8;

        Color::Rgb { r, g, b }
    }


    pub fn draw(&self, terminal: &mut crate::terminal::Terminal) -> io::Result<()> {
        terminal.clear_screen()?;
        let (width, height) = terminal.size()?;

        let text_bg_color = Self::parse_color(&self.config.colors.background);
        let text_fg_color = Self::parse_color(&self.config.colors.text);

        if let Some(bg) = text_bg_color {
            std::io::stdout().execute(SetBackgroundColor(bg))?;
        }
        if let Some(fg) = text_fg_color {
            std::io::stdout().execute(SetForegroundColor(fg))?;
        }

        let mut display_y = 0;
        let mode_bar_height = self.config.mode_bar.height.unwrap_or(2);
        let text_area_height = height.saturating_sub(mode_bar_height);

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
        std::io::stdout().execute(ResetColor)?;

        let status_bar_y = height.saturating_sub(mode_bar_height);
        let status_bar_start_color = Self::parse_color(&self.config.mode_bar.primary_color)
            .or(Self::parse_color(&self.config.colors.status_bar_background));
        let status_bar_end_color = Self::parse_color(&self.config.mode_bar.secondary_color);
        let status_bar_fg_color = Self::parse_color(&self.config.mode_bar.text_color)
            .or(Self::parse_color(&self.config.colors.status_bar_text));

        let gradient_effective_width = self.config.mode_bar.width.unwrap_or(width);

        for y_offset in 0..mode_bar_height {
            for x in 0..width {
                let current_bg_color = if let (Some(start_c), Some(end_c)) = (status_bar_start_color, status_bar_end_color) {
                    let t = (x as f32 / gradient_effective_width as f32).min(1.0);
                    Self::lerp_color(start_c, end_c, t)
                } else {
                    status_bar_start_color.unwrap_or(Color::Reset)
                };
                std::io::stdout().execute(SetBackgroundColor(current_bg_color))?;
                terminal.cursor_position(x, status_bar_y + y_offset)?;
                write!(std::io::stdout(), " ")?;
            }
        }
        std::io::stdout().execute(ResetColor)?;

        if let Some(fg) = status_bar_fg_color {
            std::io::stdout().execute(SetForegroundColor(fg))?;
        }
        if let Some(bg) = status_bar_start_color {
             std::io::stdout().execute(SetBackgroundColor(bg))?;
        }
        terminal.print_line(0, status_bar_y, &"─".repeat(width as usize))?;
        std::io::stdout().execute(ResetColor)?;

        let mut status_parts = Vec::new();
        if self.config.mode_bar.show_mode.unwrap_or(true) {
            status_parts.push(match self.mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Command => "COMMAND",
            }.to_string());
        }
        if self.config.mode_bar.show_filename.unwrap_or(true) {
            status_parts.push(self.filename.as_deref().unwrap_or("[No Name]").to_string());
        }
        if self.config.mode_bar.show_dirty_indicator.unwrap_or(true) && self.dirty {
            status_parts.push("[Modified]".to_string());
        }
        let status_text = status_parts.join(" | ");

        if let Some(fg) = status_bar_fg_color {
            std::io::stdout().execute(SetForegroundColor(fg))?;
        }
        if let Some(bg) = status_bar_start_color {
            std::io::stdout().execute(SetBackgroundColor(bg))?;
        }
        terminal.print_line(0, status_bar_y + mode_bar_height - 1, &status_text)?;
        std::io::stdout().execute(ResetColor)?;


        match self.mode {
            Mode::Command => {
                let box_width = self.config.command_box.width.unwrap_or((width as f32 * 0.6).min(80.0).max(40.0) as u16);
                let box_height = self.config.command_box.height.unwrap_or(5);

                let start_x = (width / 2).saturating_sub(box_width / 2);
                let start_y = (height / 2).saturating_sub(box_height / 2);

                let box_bg_color = Self::parse_color(&self.config.command_box.primary_color)
                    .or(Self::parse_color(&self.config.colors.command_box_background));
                let box_fg_color = Self::parse_color(&self.config.command_box.text_color)
                    .or(Self::parse_color(&self.config.colors.command_box_text));
                let box_border_color = Self::parse_color(&self.config.command_box.secondary_color)
                    .or(Self::parse_color(&self.config.colors.command_box_border));

                if let Some(bg) = box_bg_color {
                    std::io::stdout().execute(SetBackgroundColor(bg))?;
                }
                if let Some(fg) = box_fg_color {
                    std::io::stdout().execute(SetForegroundColor(fg))?;
                }

                for y in start_y..(start_y + box_height) {
                    terminal.cursor_position(start_x, y)?;
                    write!(std::io::stdout(), "{:width$}", "", width = box_width as usize)?;
                }

                if let Some(border_color) = box_border_color {
                    std::io::stdout().execute(SetForegroundColor(border_color))?;
                }
                terminal.print_line(start_x, start_y, &format!("┌{}┐", "─".repeat((box_width as usize).saturating_sub(2))))?;
                for y in (start_y + 1)..(start_y + box_height - 1) {
                    // FIX: Removed the extra ')' here
                    terminal.print_line(start_x, y, &format!("│{:width$}│", "", width = (box_width as usize).saturating_sub(2)))?;
                }
                terminal.print_line(start_x, start_y + box_height - 1, &format!("└{}┘", "─".repeat((box_width as usize).saturating_sub(2))))?;
                std::io::stdout().execute(ResetColor)?;

                if let Some(bg) = box_bg_color {
                    std::io::stdout().execute(SetBackgroundColor(bg))?;
                }
                if let Some(fg) = box_fg_color {
                    std::io::stdout().execute(SetForegroundColor(fg))?;
                }

                let label_text = self.config.command_box.text.as_deref().unwrap_or(" Command box ");
                let label_x = start_x + (box_width / 2).saturating_sub((label_text.len() / 2) as u16);
                terminal.print_line(label_x, start_y, label_text)?;

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
                    let message_fg_color = Self::parse_color(&self.config.colors.message_text);
                    if let Some(fg) = message_fg_color {
                        std::io::stdout().execute(SetForegroundColor(fg))?;
                    }
                    terminal.print_line(start_x + 1, message_y, display_message)?;
                    std::io::stdout().execute(ResetColor)?;
                }
                std::io::stdout().execute(ResetColor)?;

                std::io::stdout().execute(SetCursorStyle::BlinkingBar)?;
                terminal.cursor_position(start_x + 1 + self.command_input.len() as u16, command_line_y)?;

            }
            Mode::Normal => {
                std::io::stdout().execute(SetCursorStyle::BlinkingBlock)?;
                terminal.cursor_position(self.cursor_x as u16, (self.cursor_y - self.scroll_offset_y) as u16)?;
                if !self.message.is_empty() {
                    let message_fg_color = Self::parse_color(&self.config.colors.message_text);
                    if let Some(fg) = message_fg_color {
                        std::io::stdout().execute(SetForegroundColor(fg))?;
                    }
                    terminal.print_line(0, height - 3, &self.message)?;
                    std::io::stdout().execute(ResetColor)?;
                }
            }
            Mode::Insert => {
                std::io::stdout().execute(SetCursorStyle::BlinkingBar)?;
                terminal.cursor_position(self.cursor_x as u16, (self.cursor_y - self.scroll_offset_y) as u16)?;
                if !self.message.is_empty() {
                    let message_fg_color = Self::parse_color(&self.config.colors.message_text);
                    if let Some(fg) = message_fg_color {
                        std::io::stdout().execute(SetForegroundColor(fg))?;
                    }
                    terminal.print_line(0, height - 3, &self.message)?;
                    std::io::stdout().execute(ResetColor)?;
                }
            }
        }

        terminal.flush()?;
        Ok(())
    }

    pub fn handle_key_event(&mut self, event: KeyEvent, terminal: &mut crate::terminal::Terminal) -> io::Result<bool> {
        let (_, height) = terminal.size()?;
        let text_area_height = height.saturating_sub(self.config.mode_bar.height.unwrap_or(2));

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

