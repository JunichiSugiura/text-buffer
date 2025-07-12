//! Buffer module for storing text content and line break information
//!
//! This module implements the Buffer struct which holds text content and
//! pre-computed line break positions for efficient line-based operations.

/// A buffer that stores text content and line break positions
#[derive(Debug, Clone)]
pub struct Buffer {
    /// The actual text content
    pub content: String,
    /// Positions of line breaks in the content (byte offsets)
    pub line_starts: Vec<usize>,
}

impl Buffer {
    /// Creates a new empty buffer
    pub fn new() -> Self {
        Self {
            content: String::new(),
            line_starts: vec![0], // Always start with position 0
        }
    }

    /// Creates a buffer from the given text content
    pub fn from_text(text: &str) -> Self {
        let mut buffer = Self {
            content: text.to_string(),
            line_starts: Vec::new(),
        };
        buffer.compute_line_starts();
        buffer
    }

    /// Appends text to the buffer and updates line starts
    pub fn append(&mut self, text: &str) {
        let start_offset = self.content.len();
        self.content.push_str(text);

        // Find new line breaks in the appended text
        let mut offset = start_offset;
        for ch in text.chars() {
            if ch == '\n' {
                offset += ch.len_utf8();
                self.line_starts.push(offset);
            } else {
                offset += ch.len_utf8();
            }
        }
    }

    /// Computes and stores all line break positions
    fn compute_line_starts(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0); // First line always starts at 0

        let mut offset = 0;
        for ch in self.content.chars() {
            if ch == '\n' {
                offset += ch.len_utf8();
                self.line_starts.push(offset);
            } else {
                offset += ch.len_utf8();
            }
        }
    }

    /// Returns the number of lines in the buffer
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns the byte length of the buffer content
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Gets a slice of the buffer content
    pub fn slice(&self, start: usize, end: usize) -> &str {
        &self.content[start..end]
    }

    /// Gets the start position of a specific line (0-indexed)
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Gets the end position of a specific line (0-indexed)
    pub fn line_end(&self, line: usize) -> Option<usize> {
        use std::cmp::Ordering;
        match (line + 1).cmp(&self.line_starts.len()) {
            Ordering::Less => {
                // Not the last line, end is just before the next line start
                Some(self.line_starts[line + 1] - 1)
            }
            Ordering::Equal => {
                // Last line, end is the buffer end
                Some(self.content.len())
            }
            Ordering::Greater => None,
        }
    }

    /// Gets the content of a specific line (0-indexed)
    pub fn line_content(&self, line: usize) -> Option<&str> {
        let start = self.line_start(line)?;
        let end = self.line_end(line)?;
        Some(&self.content[start..end])
    }

    /// Finds the line number for a given byte offset
    pub fn line_at_offset(&self, offset: usize) -> Option<usize> {
        if offset > self.content.len() {
            return None;
        }

        // Binary search for the line containing this offset
        match self.line_starts.binary_search(&offset) {
            Ok(index) => Some(index),
            Err(index) => {
                if index > 0 {
                    Some(index - 1)
                } else {
                    Some(0)
                }
            }
        }
    }

    /// Converts a line/column position to a byte offset
    pub fn offset_at_position(&self, line: usize, column: usize) -> Option<usize> {
        let line_start = self.line_start(line)?;
        let line_content = self.line_content(line)?;

        // Count UTF-8 characters to get the correct byte offset
        let mut char_count = 0;
        let mut byte_offset = 0;

        for ch in line_content.chars() {
            if char_count == column {
                break;
            }
            char_count += 1;
            byte_offset += ch.len_utf8();
        }

        if char_count == column {
            Some(line_start + byte_offset)
        } else {
            None
        }
    }

    /// Converts a byte offset to a line/column position
    pub fn position_at_offset(&self, offset: usize) -> Option<(usize, usize)> {
        let line = self.line_at_offset(offset)?;
        let line_start = self.line_start(line)?;
        let line_content = self.line_content(line)?;

        // Count UTF-8 characters from line start to offset
        let bytes_from_line_start = offset - line_start;
        let mut column = 0;
        let mut byte_count = 0;

        for ch in line_content.chars() {
            if byte_count >= bytes_from_line_start {
                break;
            }
            byte_count += ch.len_utf8();
            column += 1;
        }

        Some((line, column))
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buffer = Buffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.line_count(), 1); // Always at least one line
    }

    #[test]
    fn test_buffer_from_text() {
        let buffer = Buffer::from_text("Hello\nWorld\n");
        assert_eq!(buffer.len(), 12);
        assert_eq!(buffer.line_count(), 3); // "Hello\n", "World\n", ""
        assert_eq!(buffer.line_starts, vec![0, 6, 12]);
    }

    #[test]
    fn test_line_content() {
        let buffer = Buffer::from_text("Hello\nWorld\nTest");
        assert_eq!(buffer.line_content(0), Some("Hello"));
        assert_eq!(buffer.line_content(1), Some("World"));
        assert_eq!(buffer.line_content(2), Some("Test"));
        assert_eq!(buffer.line_content(3), None);
    }

    #[test]
    fn test_append() {
        let mut buffer = Buffer::from_text("Hello");
        buffer.append("\nWorld");
        assert_eq!(buffer.content, "Hello\nWorld");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line_content(0), Some("Hello"));
        assert_eq!(buffer.line_content(1), Some("World"));
    }

    #[test]
    fn test_position_conversion() {
        let buffer = Buffer::from_text("Hello\nWorld\n");

        // Test offset to position
        assert_eq!(buffer.position_at_offset(0), Some((0, 0))); // Start of "Hello"
        assert_eq!(buffer.position_at_offset(5), Some((0, 5))); // End of "Hello"
        assert_eq!(buffer.position_at_offset(6), Some((1, 0))); // Start of "World"
        assert_eq!(buffer.position_at_offset(11), Some((1, 5))); // End of "World"

        // Test position to offset
        assert_eq!(buffer.offset_at_position(0, 0), Some(0));
        assert_eq!(buffer.offset_at_position(0, 5), Some(5));
        assert_eq!(buffer.offset_at_position(1, 0), Some(6));
        assert_eq!(buffer.offset_at_position(1, 5), Some(11));
    }

    #[test]
    fn test_line_at_offset() {
        let buffer = Buffer::from_text("Hello\nWorld\nTest");
        assert_eq!(buffer.line_at_offset(0), Some(0));
        assert_eq!(buffer.line_at_offset(3), Some(0));
        assert_eq!(buffer.line_at_offset(6), Some(1));
        assert_eq!(buffer.line_at_offset(9), Some(1));
        assert_eq!(buffer.line_at_offset(12), Some(2));
    }
}
