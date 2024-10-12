use std::{fs, io, path::PathBuf};

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            lines: Vec::new(),
            path: "<empty file>".into(),
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub path: PathBuf,
}

impl Buffer {
    /// Loads a buffer from a path.
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, io::Error> {
        let path = path.into();
        let content = fs::read_to_string(&path)?;
        let lines = content.lines().map(str::to_string).collect();

        Ok(Self { lines, path })
    }

    /// Calculates the line length for the line at a given index.
    pub fn get_line_length(&self, index: usize) -> usize {
        self.lines
            .get(index)
            .map_or(0, |line| line.len().saturating_sub(1))
    }

    /// Computes the truncated line, considering the column we are in and the window width.
    pub fn get_truncated_line(&self, row: usize, col: usize, width: usize) -> Option<&str> {
        let line = self.lines.get(row)?;
        let mut included_chars = line.char_indices().skip(col).take(width);

        let start = included_chars.next().map(|(idx, _)| idx);

        if start.is_none() {
            return Some("");
        }

        let start = start.unwrap();
        let end = included_chars.last().map_or(start, |(idx, _)| idx);

        Some(&line[start..=end])
    }

    /// Returns the index of the last line.
    pub fn get_last_line_index(&self) -> usize {
        self.lines.len().saturating_sub(1)
    }
}
