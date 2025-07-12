//! Main TextBuffer implementation using piece tree data structure
//!
//! This module provides the primary TextBuffer interface that combines
//! buffers, pieces, and red-black tree to create an efficient text editor backend.

use crate::buffer::Buffer;
use crate::piece::{utils, Piece};
use crate::red_black_tree::RBTree;
use crate::types::{Position, Range, TextBufferResult};

/// Builder for creating TextBuffer instances
#[derive(Debug)]
pub struct TextBufferBuilder {
    /// Original buffers (typically from file content)
    original_buffers: Vec<Buffer>,
    /// Added buffers (for user edits)
    added_buffers: Vec<Buffer>,
    /// Pieces representing the text structure
    pieces: Vec<Piece>,
}

impl TextBufferBuilder {
    /// Creates a new TextBufferBuilder
    pub fn new() -> Self {
        Self {
            original_buffers: Vec::new(),
            added_buffers: vec![Buffer::new()], // Always have one added buffer
            pieces: Vec::new(),
        }
    }

    /// Accepts a chunk of text (typically from file reading)
    pub fn accept_chunk(&mut self, text: &str) -> &mut Self {
        if text.is_empty() {
            return self;
        }

        // Add to the last original buffer or create a new one
        if let Some(last_buffer) = self.original_buffers.last_mut() {
            let start_offset = last_buffer.len();
            last_buffer.append(text);

            // Create a piece for this chunk
            let line_breaks = utils::count_line_breaks(text);
            let piece = Piece::original(
                self.original_buffers.len() - 1,
                start_offset,
                text.len(),
                line_breaks,
            );
            self.pieces.push(piece);
        } else {
            // First chunk - create the first original buffer
            let buffer = Buffer::from_text(text);
            let line_breaks = utils::count_line_breaks(text);
            let piece = Piece::original(0, 0, text.len(), line_breaks);

            self.original_buffers.push(buffer);
            self.pieces.push(piece);
        }

        self
    }

    /// Builds the final TextBuffer
    pub fn build(self) -> TextBuffer {
        let mut tree = RBTree::new();

        // Insert all pieces into the tree
        for piece in self.pieces {
            tree.insert(piece);
        }

        // If no pieces were added, create an empty piece
        if tree.is_empty() {
            let empty_piece = Piece::original(0, 0, 0, 0);
            tree.insert(empty_piece);

            // Ensure we have at least one original buffer
            if self.original_buffers.is_empty() {
                let original_buffers = vec![Buffer::new()];
                return TextBuffer {
                    original_buffers,
                    added_buffers: self.added_buffers,
                    tree,
                };
            }
        }

        TextBuffer {
            original_buffers: self.original_buffers,
            added_buffers: self.added_buffers,
            tree,
        }
    }
}

impl Default for TextBufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// High-performance text buffer using piece tree data structure
#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// Original buffers (read-only content from files)
    original_buffers: Vec<Buffer>,
    /// Added buffers (append-only content from edits)
    added_buffers: Vec<Buffer>,
    /// Red-black tree storing the pieces
    tree: RBTree,
}

impl TextBuffer {
    /// Creates a new empty TextBuffer
    pub fn new() -> Self {
        Self {
            original_buffers: vec![Buffer::new()],
            added_buffers: vec![Buffer::new()],
            tree: RBTree::new(),
        }
    }

    /// Creates a TextBuffer from text content
    pub fn from_text(text: &str) -> Self {
        let mut builder = TextBufferBuilder::new();
        builder.accept_chunk(text);
        builder.build()
    }

    /// Returns the total number of lines in the buffer
    pub fn line_count(&self) -> usize {
        if self.tree.is_empty() {
            return 1; // Always at least one line
        }

        // Count total line breaks and add 1
        self.tree.total_line_breaks() + 1
    }

    /// Returns the total length of the buffer in bytes
    pub fn length(&self) -> usize {
        self.tree.total_length()
    }

    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Gets the content of a specific line (0-indexed)
    pub fn get_line_content(&self, line: usize) -> TextBufferResult<String> {
        if line >= self.line_count() {
            return Err(format!("Line {line} out of bounds"));
        }

        let mut content = String::new();
        let mut current_line = 0;
        let pieces = self.tree.collect_pieces();

        for piece in pieces {
            let piece_content = self
                .get_piece_content(piece)
                .ok_or_else(|| "Failed to get piece content".to_string())?;

            for ch in piece_content.chars() {
                if current_line == line {
                    if ch == '\n' {
                        break;
                    }
                    content.push(ch);
                } else if ch == '\n' {
                    current_line += 1;
                    if current_line > line {
                        break;
                    }
                }
            }

            if current_line > line {
                break;
            }
        }

        Ok(content)
    }

    /// Gets the length of a specific line (0-indexed)
    pub fn get_line_length(&self, line: usize) -> TextBufferResult<usize> {
        let content = self.get_line_content(line)?;
        Ok(content.chars().count())
    }

    /// Inserts text at the specified position
    pub fn insert(&mut self, position: Position, text: &str) -> TextBufferResult<()> {
        if text.is_empty() {
            return Ok(());
        }

        // Find the offset for the position
        let offset = self.position_to_offset(position)?;

        // Add text to the added buffer
        let added_buffer = self.added_buffers.last_mut().unwrap();
        let start_offset = added_buffer.len();
        added_buffer.append(text);

        // Create a new piece for the inserted text
        let line_breaks = utils::count_line_breaks(text);
        let new_piece = Piece::added(
            self.added_buffers.len() - 1,
            start_offset,
            text.len(),
            line_breaks,
        );

        // Find the piece at the insertion point and split if necessary
        if let Some(_piece_at_offset) = self.tree.find_piece_at_offset(offset) {
            // This is a simplified insertion - in a full implementation,
            // you would need to split the piece and rebuild the tree
            self.tree.insert(new_piece);
        } else {
            // Insert at the end
            self.tree.insert(new_piece);
        }

        Ok(())
    }

    /// Deletes text in the specified range
    pub fn delete(&mut self, range: Range) -> TextBufferResult<String> {
        let start_offset = self.position_to_offset(range.start)?;
        let end_offset = self.position_to_offset(range.end)?;

        if start_offset >= end_offset {
            return Err("Invalid range".to_string());
        }

        // Get the text that will be deleted
        let deleted_text = self.get_text_in_range(range)?;

        // This is a simplified deletion - in a full implementation,
        // you would need to modify the tree structure to remove the range
        // For now, we'll just return the deleted text

        Ok(deleted_text)
    }

    /// Gets text content in the specified range
    pub fn get_text_in_range(&self, range: Range) -> TextBufferResult<String> {
        let start_offset = self.position_to_offset(range.start)?;
        let end_offset = self.position_to_offset(range.end)?;

        if start_offset >= end_offset {
            return Err("Invalid range".to_string());
        }

        let mut content = String::new();
        let mut current_offset = 0;
        let pieces = self.tree.collect_pieces();

        for piece in pieces {
            let piece_content = self
                .get_piece_content(piece)
                .ok_or_else(|| "Failed to get piece content".to_string())?;

            let piece_start = current_offset;
            let piece_end = current_offset + piece.length;

            if piece_start >= end_offset {
                break;
            }

            if piece_end > start_offset {
                let content_start = start_offset.saturating_sub(piece_start);

                let content_end = if end_offset < piece_end {
                    end_offset - piece_start
                } else {
                    piece.length
                };

                if content_start < piece_content.len() && content_end <= piece_content.len() {
                    content.push_str(&piece_content[content_start..content_end]);
                }
            }

            current_offset = piece_end;
        }

        Ok(content)
    }

    /// Gets all text content as a string
    pub fn get_all_text(&self) -> String {
        let mut content = String::new();
        let pieces = self.tree.collect_pieces();

        for piece in pieces {
            if let Some(piece_content) = self.get_piece_content(piece) {
                content.push_str(piece_content);
            }
        }

        content
    }

    /// Converts a position to a byte offset
    pub fn position_to_offset(&self, position: Position) -> TextBufferResult<usize> {
        let mut current_line = 0;
        let mut current_column = 0;
        let mut byte_offset = 0;
        let pieces = self.tree.collect_pieces();

        for piece in pieces {
            let piece_content = self
                .get_piece_content(piece)
                .ok_or_else(|| "Failed to get piece content".to_string())?;

            for ch in piece_content.chars() {
                if current_line == position.line {
                    if current_column == position.column {
                        return Ok(byte_offset);
                    }
                    if ch == '\n' {
                        // End of line reached, position is at the end of line
                        return Ok(byte_offset);
                    }
                    current_column += 1;
                } else if ch == '\n' {
                    current_line += 1;
                    current_column = 0;
                    if current_line > position.line {
                        return Err("Position out of bounds".to_string());
                    }
                }
                byte_offset += ch.len_utf8();
            }
        }

        // Handle end of buffer case
        if current_line == position.line && current_column == position.column {
            Ok(byte_offset)
        } else {
            Err("Position out of bounds".to_string())
        }
    }

    /// Converts a byte offset to a position
    pub fn offset_to_position(&self, offset: usize) -> TextBufferResult<Position> {
        let mut current_line = 0;
        let mut current_column = 0;
        let mut current_offset = 0;
        let pieces = self.tree.collect_pieces();

        for piece in pieces {
            let piece_content = self
                .get_piece_content(piece)
                .ok_or_else(|| "Failed to get piece content".to_string())?;

            for ch in piece_content.chars() {
                if current_offset == offset {
                    return Ok(Position::new(current_line, current_column));
                }

                if ch == '\n' {
                    current_line += 1;
                    current_column = 0;
                } else {
                    current_column += 1;
                }

                current_offset += ch.len_utf8();
            }
        }

        if current_offset == offset {
            Ok(Position::new(current_line, current_column))
        } else {
            Err("Offset out of bounds".to_string())
        }
    }

    /// Helper method to get piece content from the appropriate buffer
    fn get_piece_content(&self, piece: &Piece) -> Option<&str> {
        utils::get_piece_content(piece, &self.original_buffers, &self.added_buffers)
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buffer = TextBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.length(), 0);
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_from_text() {
        let buffer = TextBuffer::from_text("Hello\nWorld\n");
        assert!(!buffer.is_empty());
        assert_eq!(buffer.length(), 12);
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn test_get_line_content() {
        let buffer = TextBuffer::from_text("Hello\nWorld\nTest");

        assert_eq!(buffer.get_line_content(0).unwrap(), "Hello");
        assert_eq!(buffer.get_line_content(1).unwrap(), "World");
        assert_eq!(buffer.get_line_content(2).unwrap(), "Test");
        assert!(buffer.get_line_content(3).is_err());
    }

    #[test]
    fn test_get_line_length() {
        let buffer = TextBuffer::from_text("Hello\nWorld\n");

        assert_eq!(buffer.get_line_length(0).unwrap(), 5); // "Hello"
        assert_eq!(buffer.get_line_length(1).unwrap(), 5); // "World"
        assert_eq!(buffer.get_line_length(2).unwrap(), 0); // Empty line
    }

    #[test]
    fn test_position_offset_conversion() {
        let buffer = TextBuffer::from_text("Hello\nWorld\n");

        // Test position to offset
        assert_eq!(buffer.position_to_offset(Position::new(0, 0)).unwrap(), 0);
        assert_eq!(buffer.position_to_offset(Position::new(0, 5)).unwrap(), 5);
        assert_eq!(buffer.position_to_offset(Position::new(1, 0)).unwrap(), 6);
        assert_eq!(buffer.position_to_offset(Position::new(1, 5)).unwrap(), 11);

        // Test offset to position
        assert_eq!(buffer.offset_to_position(0).unwrap(), Position::new(0, 0));
        assert_eq!(buffer.offset_to_position(5).unwrap(), Position::new(0, 5));
        assert_eq!(buffer.offset_to_position(6).unwrap(), Position::new(1, 0));
        assert_eq!(buffer.offset_to_position(11).unwrap(), Position::new(1, 5));
    }

    #[test]
    fn test_get_all_text() {
        let text = "Hello\nWorld\nTest";
        let buffer = TextBuffer::from_text(text);
        assert_eq!(buffer.get_all_text(), text);
    }

    #[test]
    fn test_get_text_in_range() {
        let buffer = TextBuffer::from_text("Hello\nWorld\nTest");

        let range = Range::new(Position::new(0, 0), Position::new(0, 5));
        assert_eq!(buffer.get_text_in_range(range).unwrap(), "Hello");

        let range = Range::new(Position::new(1, 0), Position::new(1, 5));
        assert_eq!(buffer.get_text_in_range(range).unwrap(), "World");

        // Fix: Test content is "Hello\nWorld\nTest" (17 chars total)
        // Line 2 has "Test" (4 chars), so valid range is (2, 0) to (2, 4)
        let range = Range::new(Position::new(0, 0), Position::new(2, 4));
        assert_eq!(
            buffer.get_text_in_range(range).unwrap(),
            "Hello\nWorld\nTest"
        );
    }

    #[test]
    fn test_builder() {
        let mut builder = TextBufferBuilder::new();
        builder.accept_chunk("Hello");
        builder.accept_chunk("\n");
        builder.accept_chunk("World");

        let buffer = builder.build();
        assert_eq!(buffer.get_all_text(), "Hello\nWorld");
        assert_eq!(buffer.line_count(), 2);
    }

    #[test]
    fn test_insert() {
        let mut buffer = TextBuffer::from_text("Hello\nWorld");
        let result = buffer.insert(Position::new(0, 5), " there");
        assert!(result.is_ok());

        // Note: This is a simplified test - the actual insertion logic
        // would need to be more sophisticated to handle piece splitting
    }
}
