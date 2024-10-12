#![warn(clippy::all, clippy::pedantic)]

use std::env;

use editor::Editor;
use terminal::TResult;

mod buffer;
mod editor;
mod terminal;
mod view;

fn main() -> TResult<()> {
    let mut editor = Editor::new();

    if let Some(path) = env::args().nth(1) {
        editor.view.load(path)?;
    }

    editor.run()
}
