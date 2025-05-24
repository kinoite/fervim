use crossterm::event::{Event, KeyEventKind};
use std::env;
use std::io;
use crossterm::style::ResetColor;
use crossterm::ExecutableCommand;
use std::path::PathBuf;

mod editor;
mod terminal;
mod config;

fn main() -> io::Result<()> {
    let filename = env::args().nth(1);

    let config_path = if let Some(config_dir) = dirs::config_dir() {
        let mut path = config_dir;
        path.push("fervim");
        path.push("config.toml");
        path
    } else {
        PathBuf::from("config.toml")
    };

    let config = match config::Config::load(config_path.to_str().unwrap_or("config.toml")) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: Could not load config from {:?} ({}). Using default configuration.", config_path, e);
            config::Config::default()
        }
    };

    let mut terminal = terminal::Terminal::new()?;
    let mut editor = editor::Editor::new(filename, config)?;

    loop {
        let (_, height) = terminal.size()?;
        let text_area_height = height - 2;

        editor.draw(&mut terminal)?;

        match terminal.read_event()? {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    if !editor.handle_key_event(key_event, &mut terminal)? {
                        break;
                    }
                }
            }
            Event::Resize(_, _) => {
                editor.adjust_scroll(text_area_height);
            }
            _ => {}
        }
    }

    std::io::stdout().execute(ResetColor)?;
    Ok(())
}

