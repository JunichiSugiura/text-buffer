//! Piece module for representing text segments in the piece tree
//!
//! This module implements the Piece struct which represents a contiguous
//! segment of text from either the original or added buffer.

/// Type of piece indicating which buffer it references
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    /// References text from the original buffer
    Original,
    /// References text from the added buffer
    Added,
}

/// A piece represents a contiguous segment of text from a buffer
#[derive(Debug, Clone)]
pub struct Piece {
    /// Which buffer this piece references
    pub piece_type: PieceType,
    /// Buffer index (for multiple buffers)
    pub buffer_index: usize,
    /// Starting position in the buffer (byte offset)
    pub start: usize,
    /// Length of the piece in bytes
    pub length: usize,
    /// Number of line breaks in this piece
    pub line_break_count: usize,
}

impl Piece {
    /// Creates a new piece
    pub fn new(
        piece_type: PieceType,
        buffer_index: usize,
        start: usize,
        length: usize,
        line_break_count: usize,
    ) -> Self {
        Self {
            piece_type,
            buffer_index,
            start,
            length,
            line_break_count,
        }
    }

    /// Creates a piece for the original buffer
    pub fn original(
        buffer_index: usize,
        start: usize,
        length: usize,
        line_break_count: usize,
    ) -> Self {
        Self::new(
            PieceType::Original,
            buffer_index,
            start,
            length,
            line_break_count,
        )
    }

    /// Creates a piece for the added buffer
    pub fn added(
        buffer_index: usize,
        start: usize,
        length: usize,
        line_break_count: usize,
    ) -> Self {
        Self::new(
            PieceType::Added,
            buffer_index,
            start,
            length,
            line_break_count,
        )
    }

    /// Returns the end position of this piece (exclusive)
    pub fn end(&self) -> usize {
        self.start + self.length
    }

    /// Returns true if this piece is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Splits this piece at the given offset within the piece
    /// Returns (left_piece, right_piece) where left_piece contains [0, offset)
    /// and right_piece contains [offset, length)
    pub fn split_at(&self, offset: usize, left_line_breaks: usize) -> (Piece, Piece) {
        assert!(offset <= self.length, "Split offset out of bounds");

        let left = Piece::new(
            self.piece_type,
            self.buffer_index,
            self.start,
            offset,
            left_line_breaks,
        );

        let right = Piece::new(
            self.piece_type,
            self.buffer_index,
            self.start + offset,
            self.length - offset,
            self.line_break_count - left_line_breaks,
        );

        (left, right)
    }

    /// Creates a sub-piece from this piece
    pub fn sub_piece(&self, start: usize, length: usize, line_break_count: usize) -> Piece {
        assert!(start + length <= self.length, "Sub-piece out of bounds");

        Piece::new(
            self.piece_type,
            self.buffer_index,
            self.start + start,
            length,
            line_break_count,
        )
    }
}

impl PartialEq for Piece {
    fn eq(&self, other: &Self) -> bool {
        self.piece_type == other.piece_type
            && self.buffer_index == other.buffer_index
            && self.start == other.start
            && self.length == other.length
    }
}

impl Eq for Piece {}

/// Position within a piece, used for efficient lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PiecePosition {
    /// Line number within the piece (0-indexed)
    pub line: usize,
    /// Byte offset from the start of the line
    pub offset: usize,
}

impl PiecePosition {
    pub fn new(line: usize, offset: usize) -> Self {
        Self { line, offset }
    }
}

/// Helper functions for working with pieces and buffers
pub mod utils {
    use super::*;
    use crate::buffer::Buffer;

    /// Counts the number of line breaks in a text slice
    pub fn count_line_breaks(text: &str) -> usize {
        text.chars().filter(|&c| c == '\n').count()
    }

    /// Gets the text content of a piece from the appropriate buffer
    pub fn get_piece_content<'a>(
        piece: &Piece,
        original_buffers: &'a [Buffer],
        added_buffers: &'a [Buffer],
    ) -> Option<&'a str> {
        let buffers = match piece.piece_type {
            PieceType::Original => original_buffers,
            PieceType::Added => added_buffers,
        };

        let buffer = buffers.get(piece.buffer_index)?;
        if piece.start + piece.length <= buffer.len() {
            Some(buffer.slice(piece.start, piece.start + piece.length))
        } else {
            None
        }
    }

    /// Finds the line and column position within a piece at a given byte offset
    pub fn position_in_piece(
        piece: &Piece,
        offset: usize,
        original_buffers: &[Buffer],
        added_buffers: &[Buffer],
    ) -> Option<(usize, usize)> {
        // Allow offset equal to piece length (end of piece)
        if offset > piece.length {
            return None;
        }

        let content = get_piece_content(piece, original_buffers, added_buffers)?;

        // Handle the case where offset is 0 or piece is empty
        if offset == 0 || content.is_empty() {
            return Some((0, 0));
        }

        // Ensure we don't go out of bounds
        let safe_offset = offset.min(content.len());
        let target_content = &content[..safe_offset];

        let line = count_line_breaks(target_content);
        let column = if let Some(last_newline) = target_content.rfind('\n') {
            target_content[last_newline + 1..].chars().count()
        } else {
            target_content.chars().count()
        };

        Some((line, column))
    }

    /// Finds the byte offset within a piece at a given line and column
    pub fn offset_in_piece(
        piece: &Piece,
        line: usize,
        column: usize,
        original_buffers: &[Buffer],
        added_buffers: &[Buffer],
    ) -> Option<usize> {
        let content = get_piece_content(piece, original_buffers, added_buffers)?;

        let mut current_line = 0;
        let mut byte_offset = 0;

        for ch in content.chars() {
            if current_line == line {
                if column == 0 {
                    return Some(byte_offset);
                }

                let mut char_count = 0;
                let mut line_byte_offset = 0;

                for line_ch in content[byte_offset..].chars() {
                    if line_ch == '\n' {
                        break;
                    }
                    if char_count == column {
                        return Some(byte_offset + line_byte_offset);
                    }
                    char_count += 1;
                    line_byte_offset += line_ch.len_utf8();
                }

                if char_count == column {
                    return Some(byte_offset + line_byte_offset);
                }

                return None;
            }

            if ch == '\n' {
                current_line += 1;
            }

            byte_offset += ch.len_utf8();
        }

        if current_line == line && column == 0 {
            Some(byte_offset)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    #[test]
    fn test_piece_creation() {
        let piece = Piece::original(0, 10, 20, 2);
        assert_eq!(piece.piece_type, PieceType::Original);
        assert_eq!(piece.buffer_index, 0);
        assert_eq!(piece.start, 10);
        assert_eq!(piece.length, 20);
        assert_eq!(piece.line_break_count, 2);
        assert_eq!(piece.end(), 30);
    }

    #[test]
    fn test_piece_split() {
        let piece = Piece::original(0, 10, 20, 3);
        let (left, right) = piece.split_at(8, 1);

        assert_eq!(left.start, 10);
        assert_eq!(left.length, 8);
        assert_eq!(left.line_break_count, 1);

        assert_eq!(right.start, 18);
        assert_eq!(right.length, 12);
        assert_eq!(right.line_break_count, 2);
    }

    #[test]
    fn test_piece_sub_piece() {
        let piece = Piece::original(0, 10, 20, 3);
        let sub = piece.sub_piece(5, 10, 2);

        assert_eq!(sub.start, 15);
        assert_eq!(sub.length, 10);
        assert_eq!(sub.line_break_count, 2);
    }

    #[test]
    fn test_count_line_breaks() {
        assert_eq!(utils::count_line_breaks("hello"), 0);
        assert_eq!(utils::count_line_breaks("hello\n"), 1);
        assert_eq!(utils::count_line_breaks("hello\nworld\n"), 2);
        assert_eq!(utils::count_line_breaks("\n\n\n"), 3);
    }

    #[test]
    fn test_get_piece_content() {
        let buffer = Buffer::from_text("Hello\nWorld\nTest");
        let piece = Piece::original(0, 6, 5, 0); // "World"
        let buffers = vec![buffer];

        let content = utils::get_piece_content(&piece, &buffers, &[]);
        assert_eq!(content, Some("World"));
    }

    #[test]
    fn test_position_in_piece() {
        let buffer = Buffer::from_text("Hello\nWorld\nTest");
        // Fix: The string "Hello\nWorld\nTest" is 16 bytes, not 17
        let piece = Piece::original(0, 0, 16, 2); // Entire buffer
        let buffers = vec![buffer];

        // Test position at start
        assert_eq!(
            utils::position_in_piece(&piece, 0, &buffers, &[]),
            Some((0, 0))
        );

        // Test position at end of first line
        assert_eq!(
            utils::position_in_piece(&piece, 5, &buffers, &[]),
            Some((0, 5))
        );

        // Test position at start of second line
        assert_eq!(
            utils::position_in_piece(&piece, 6, &buffers, &[]),
            Some((1, 0))
        );
    }
}
