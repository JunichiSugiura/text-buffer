//! # Text Buffer
//!
//! A high-performance text buffer implementation using the piece tree data structure,
//! inspired by VS Code's text buffer reimplementation.
//!
//! The piece tree provides efficient text editing operations with O(log n) complexity
//! for insertions, deletions, and lookups while maintaining low memory overhead.

mod buffer;
mod piece;
mod red_black_tree;
mod text_buffer;
mod types;

pub use buffer::Buffer;
pub use piece::{Piece, PiecePosition, PieceType};
pub use red_black_tree::{Color, RBNode, RBTree};
pub use text_buffer::{TextBuffer, TextBufferBuilder};
pub use types::{Position, Range, TextBufferResult};

/// Utility functions for working with pieces and text content
pub mod utils {
    pub use crate::piece::utils::*;
}
