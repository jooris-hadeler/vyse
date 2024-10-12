use std::{fs, io, path::PathBuf};

#[derive(Debug, Default)]
pub struct Buffer {
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path.into())?;
        let lines = content.lines().map(str::to_string).collect();

        Ok(Self { lines })
    }

    pub fn get_line_length(&self, number: usize) -> usize {
        self.lines
            .get(number)
            .map_or(0, |line| line.len().saturating_sub(1))
    }

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

    pub fn get_last_line_index(&self) -> usize {
        self.lines.len().saturating_sub(1)
    }
}
