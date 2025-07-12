//! Common types used throughout the text buffer implementation

/// Position in the text buffer represented as line and column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Range in the text buffer from start to end position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// Result of a text buffer operation
pub type TextBufferResult<T> = Result<T, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_range_creation() {
        let start = Position::new(1, 0);
        let end = Position::new(2, 5);
        let range = Range::new(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }
}
