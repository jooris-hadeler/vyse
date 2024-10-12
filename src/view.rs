use std::path::PathBuf;

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind},
    style::Color,
};

use crate::{
    buffer::Buffer,
    terminal::{self, Position, Size, TResult},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Default)]
pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
    pub current_size: Size,
    pub cursor_location: Location,
    pub scroll_offset: Location,
}

impl View {
    /// Loads a new file into the view.
    pub fn load<P: Into<PathBuf>>(&mut self, path: P) -> TResult<()> {
        self.buffer = Buffer::from_path(path)?;
        self.needs_redraw = true;

        Ok(())
    }

    /// Rendes the whole view to the screen.
    pub fn render(&mut self) -> TResult<()> {
        if !self.needs_redraw && !self.is_of_sufficient_size() {
            return Ok(());
        }

        self.render_buffer()?;
        self.render_status_bar()?;

        terminal::move_cursor_to(self.get_relative_cursor_position())?;

        self.needs_redraw = false;

        Ok(())
    }

    /// Returns whether or the window is big enough to render the editor.
    fn is_of_sufficient_size(&self) -> bool {
        self.current_size.height > 1 && self.current_size.width > 0
    }

    /// Returns the height of the buffer.
    fn buffer_height(&self) -> u16 {
        self.current_size.height.saturating_sub(1)
    }

    /// Renders the buffer to the screen.
    fn render_buffer(&mut self) -> TResult<()> {
        for pos_y in 0..self.buffer_height() {
            let buffer_row_index = pos_y as usize + self.scroll_offset.row;

            if let Some(line) = self.buffer.get_truncated_line(
                buffer_row_index,
                self.scroll_offset.col,
                self.current_size.width as usize,
            ) {
                render_line(pos_y, line)?;
            } else {
                render_line(pos_y, "~")?;
            }
        }

        Ok(())
    }

    /// Renders the statusbar to the screen.
    fn render_status_bar(&mut self) -> TResult<()> {
        let Size { height, width } = self.current_size;

        terminal::move_cursor_to(Position {
            x: 0,
            y: height - 1,
        })?;

        // Set status bar colors and clear line.
        terminal::set_foreground_color(Color::Black)?;
        terminal::set_background_color(Color::White)?;
        terminal::clear_line()?;

        // Display current file path.
        terminal::print(self.buffer.path.to_str().unwrap())?;

        // Display cursor position.
        let current_location = format!(
            "LINE {} COL {}",
            self.cursor_location.row + 1,
            self.cursor_location.col + 1
        );
        terminal::move_cursor_to(Position {
            x: width.saturating_sub(current_location.len() as u16),
            y: height - 1,
        })?;
        terminal::print(current_location)?;

        // Reset status bar colors.
        terminal::set_foreground_color(Color::White)?;
        terminal::set_background_color(Color::Black)?;

        Ok(())
    }

    /// Calculates the cursor position relative to the current scroll position.
    fn get_relative_cursor_position(&self) -> Position {
        #[allow(clippy::cast_possible_truncation)]
        Position {
            x: self
                .cursor_location
                .col
                .saturating_sub(self.scroll_offset.col) as u16,
            y: self
                .cursor_location
                .row
                .saturating_sub(self.scroll_offset.row) as u16,
        }
    }

    /// Handles events, e.g. input or resizing.
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            Event::Resize(width, height) => self.handle_resize_event(*width, *height),

            _ => (),
        }
    }

    /// Handles a resize event.
    fn handle_resize_event(&mut self, width: u16, height: u16) {
        self.current_size = Size { width, height };
        self.needs_redraw = true;
    }

    /// Handles an input event.
    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        match key_event.code {
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End => self.move_cursor(key_event.code),
            _ => (),
        }
    }

    /// Moves the cursor based on a pressed key.
    fn move_cursor(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Left => {
                // If we are at the beginning of a line, go to the end of the previous line.
                if self.cursor_location.col == 0 {
                    if self.cursor_location.row != 0 {
                        self.cursor_location.row = self.cursor_location.row.saturating_sub(1);
                        self.cursor_location.col =
                            self.buffer.get_line_length(self.cursor_location.row);
                    }
                } else {
                    self.cursor_location.col -= 1;
                }
            }
            KeyCode::Right => {
                let current_line_length = self.buffer.get_line_length(self.cursor_location.row);

                // If we are at the end of the line go to the beginning of the next line.
                if self.cursor_location.col == current_line_length {
                    self.cursor_location.row = self.cursor_location.row.saturating_add(1);
                    self.cursor_location.col = 0;
                } else {
                    self.cursor_location.col = self.cursor_location.col.saturating_add(1);
                }
            }
            KeyCode::Up => self.cursor_location.row = self.cursor_location.row.saturating_sub(1),
            KeyCode::Down => self.cursor_location.row = self.cursor_location.row.saturating_add(1),
            KeyCode::PageUp => self.cursor_location = Location { row: 0, col: 0 },
            KeyCode::PageDown => {
                self.cursor_location = Location {
                    row: usize::MAX,
                    col: 0,
                }
            }

            KeyCode::Home => self.cursor_location.col = 0,
            KeyCode::End => self.cursor_location.col = usize::MAX,
            _ => (),
        }

        // Clamp the column to the end of the current line.
        let current_line_length = self.buffer.get_line_length(self.cursor_location.row);
        self.cursor_location.col = self.cursor_location.col.min(current_line_length);

        // Clamp the row to the last line.
        let last_line_index = self.buffer.get_last_line_index();
        self.cursor_location.row = self.cursor_location.row.min(last_line_index);

        self.update_scroll();

        self.needs_redraw = true;
    }

    /// Updates the scroll status to ensure we can always see the cursor.
    fn update_scroll(&mut self) {
        // If we scroll up and are outside the view, readjust to include the cursor.
        if self.scroll_offset.row > self.cursor_location.row {
            self.scroll_offset.row = self.cursor_location.row;
        }

        let view_end_row = self
            .scroll_offset
            .row
            .saturating_add(self.buffer_height().saturating_sub(1) as usize);

        // If we scroll down and are now outside the view, readjust to include the cursor.
        if self.cursor_location.row > view_end_row {
            let offset = self.cursor_location.row.saturating_sub(view_end_row);
            self.scroll_offset.row = self.scroll_offset.row.saturating_add(offset);
        }

        // If we scroll left and are now outside the view, readjust to include the cursor.
        if self.scroll_offset.col > self.cursor_location.col {
            self.scroll_offset.col = self.cursor_location.col;
        }

        let view_end_col = self
            .scroll_offset
            .col
            .saturating_add(self.current_size.width as usize);

        // If we scroll right and are now outside of the view, readjust to include the cursor.
        if self.cursor_location.col > view_end_col {
            let offset = self.cursor_location.col.saturating_sub(view_end_col);
            self.scroll_offset.col = self.scroll_offset.col.saturating_add(offset);
        }
    }
}

/// Rendes a line of text at the given y position.
fn render_line(pos_y: u16, line_text: &str) -> TResult<()> {
    terminal::move_cursor_to(Position { x: 0, y: pos_y })?;
    terminal::clear_line()?;
    terminal::print(line_text)
}
