use crossterm::event::{Event, KeyEventKind};
use std::env;
use std::io;

mod editor;
mod terminal;

fn main() -> io::Result<()> {
    let filename = env::args().nth(1);

    let mut terminal = terminal::Terminal::new()?;
    let mut editor = editor::Editor::new(filename)?;

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

    Ok(())
}

