use std::{
    fmt::Display,
    io::{self, stdout, Write},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size as crossterm_size, Clear, ClearType},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub type TResult<T> = Result<T, io::Error>;

pub fn terminate() -> TResult<()> {
    execute()?;
    disable_raw_mode()?;
    Ok(())
}

pub fn initialize() -> TResult<()> {
    enable_raw_mode()?;
    clear_screen()?;
    move_cursor_to(Position { x: 0, y: 0 })?;
    execute()
}

pub fn clear_screen() -> TResult<()> {
    queue!(stdout(), Clear(ClearType::All))
}

pub fn clear_line() -> TResult<()> {
    queue!(stdout(), Clear(ClearType::CurrentLine))
}

pub fn move_cursor_to(position: Position) -> TResult<()> {
    queue!(stdout(), MoveTo(position.x, position.y))
}

pub fn hide_cursor() -> TResult<()> {
    queue!(stdout(), Hide)
}

pub fn show_cursor() -> TResult<()> {
    queue!(stdout(), Show)
}

pub fn print(text: impl Display) -> TResult<()> {
    queue!(stdout(), Print(text))
}

pub fn size() -> TResult<Size> {
    let (width, height) = crossterm_size()?;
    Ok(Size { width, height })
}

pub fn set_foreground_color(color: Color) -> TResult<()> {
    queue!(stdout(), SetForegroundColor(color))
}

pub fn set_background_color(color: Color) -> TResult<()> {
    queue!(stdout(), SetBackgroundColor(color))
}

pub fn execute() -> TResult<()> {
    stdout().flush()
}
