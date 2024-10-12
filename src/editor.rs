use std::panic::{set_hook, take_hook};

use crate::{
    terminal::{self, Position, TResult},
    view::View,
};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub struct Editor {
    should_quit: bool,
    pub view: View,
}

impl Editor {
    pub fn new() -> Self {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = terminal::terminate();
            current_hook(panic_info);
        }));

        let view = View::default();

        Self {
            should_quit: false,
            view,
        }
    }

    pub fn run(&mut self) -> TResult<()> {
        terminal::initialize()?;
        self.repl()?;
        terminal::terminate()
    }

    fn repl(&mut self) -> TResult<()> {
        loop {
            self.render()?;

            if self.should_quit {
                break;
            }

            let event = read()?;
            self.handle_event(&event);
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) {
        match event {
            // Handle quit event.
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                self.should_quit = true;
            }

            event => self.view.handle_event(event),
        }
    }

    fn render(&mut self) -> TResult<()> {
        terminal::hide_cursor()?;

        if self.should_quit {
            terminal::clear_screen()?;
            terminal::move_cursor_to(Position { x: 0, y: 0 })?;
            terminal::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
        }

        terminal::show_cursor()?;
        terminal::execute()
    }
}
