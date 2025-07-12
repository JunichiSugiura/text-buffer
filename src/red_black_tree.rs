//! Red-Black Tree implementation for the piece tree
//!
//! This module implements a Red-Black Tree that stores pieces with additional
//! metadata for efficient line-based and offset-based lookups.

use crate::piece::Piece;

/// Color of a Red-Black Tree node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

/// A node in the Red-Black Tree
#[derive(Debug, Clone)]
pub struct RBNode {
    /// The piece stored in this node
    pub piece: Piece,
    /// Color of the node
    pub color: Color,
    /// Total length of text in the left subtree
    pub left_subtree_length: usize,
    /// Total number of line breaks in the left subtree
    pub left_subtree_line_breaks: usize,
    /// Left child
    pub left: Option<Box<RBNode>>,
    /// Right child
    pub right: Option<Box<RBNode>>,
}

impl RBNode {
    /// Creates a new Red-Black Tree node
    pub fn new(piece: Piece) -> Self {
        Self {
            piece,
            color: Color::Red, // New nodes are always red
            left_subtree_length: 0,
            left_subtree_line_breaks: 0,
            left: None,
            right: None,
        }
    }

    /// Creates a new black node (used for root)
    pub fn new_black(piece: Piece) -> Self {
        let mut node = Self::new(piece);
        node.color = Color::Black;
        node
    }

    /// Returns true if this node is red
    pub fn is_red(&self) -> bool {
        self.color == Color::Red
    }

    /// Returns true if this node is black
    pub fn is_black(&self) -> bool {
        self.color == Color::Black
    }

    /// Updates the cached metadata for this node
    pub fn update_metadata(&mut self) {
        self.left_subtree_length = self
            .left
            .as_ref()
            .map(|node| node.left_subtree_length + node.piece.length + node.right_subtree_length())
            .unwrap_or(0);

        self.left_subtree_line_breaks = self
            .left
            .as_ref()
            .map(|node| {
                node.left_subtree_line_breaks
                    + node.piece.line_break_count
                    + node.right_subtree_line_breaks()
            })
            .unwrap_or(0);
    }

    /// Returns the total length of the right subtree
    pub fn right_subtree_length(&self) -> usize {
        self.right
            .as_ref()
            .map(|node| node.left_subtree_length + node.piece.length + node.right_subtree_length())
            .unwrap_or(0)
    }

    /// Returns the total line breaks in the right subtree
    pub fn right_subtree_line_breaks(&self) -> usize {
        self.right
            .as_ref()
            .map(|node| {
                node.left_subtree_line_breaks
                    + node.piece.line_break_count
                    + node.right_subtree_line_breaks()
            })
            .unwrap_or(0)
    }

    /// Returns the total length of this node and its subtrees
    pub fn total_length(&self) -> usize {
        self.left_subtree_length + self.piece.length + self.right_subtree_length()
    }

    /// Returns the total line breaks of this node and its subtrees
    pub fn total_line_breaks(&self) -> usize {
        self.left_subtree_line_breaks
            + self.piece.line_break_count
            + self.right_subtree_line_breaks()
    }
}

/// Red-Black Tree for storing pieces
#[derive(Debug, Clone)]
pub struct RBTree {
    /// Root node of the tree
    pub root: Option<Box<RBNode>>,
    /// Total number of nodes in the tree
    pub size: usize,
}

impl RBTree {
    /// Creates a new empty Red-Black Tree
    pub fn new() -> Self {
        Self {
            root: None,
            size: 0,
        }
    }

    /// Creates a tree with a single piece
    pub fn from_piece(piece: Piece) -> Self {
        let mut tree = Self::new();
        tree.insert(piece);
        tree
    }

    /// Returns true if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the number of nodes in the tree
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns the total length of all text in the tree
    pub fn total_length(&self) -> usize {
        self.root
            .as_ref()
            .map(|node| node.total_length())
            .unwrap_or(0)
    }

    /// Returns the total number of line breaks in the tree
    pub fn total_line_breaks(&self) -> usize {
        self.root
            .as_ref()
            .map(|node| node.total_line_breaks())
            .unwrap_or(0)
    }

    /// Inserts a piece into the tree
    pub fn insert(&mut self, piece: Piece) {
        self.root = Self::insert_recursive(self.root.take(), piece);
        if let Some(ref mut root) = self.root {
            root.color = Color::Black; // Root is always black
        }
        self.size += 1;
    }

    /// Recursive helper for insertion
    fn insert_recursive(node: Option<Box<RBNode>>, piece: Piece) -> Option<Box<RBNode>> {
        let mut node = match node {
            None => return Some(Box::new(RBNode::new(piece))),
            Some(node) => node,
        };

        // Insert based on offset (we'll insert at the end for simplicity)
        // In a real implementation, you'd compare based on the desired insertion point
        node.right = Self::insert_recursive(node.right.take(), piece);

        // Red-Black Tree balancing
        if Self::is_red(&node.right) && !Self::is_red(&node.left) {
            node = Self::rotate_left(node);
        }
        if Self::is_red(&node.left) && Self::is_red_left_child(&node.left) {
            node = Self::rotate_right(node);
        }
        if Self::is_red(&node.left) && Self::is_red(&node.right) {
            Self::flip_colors(&mut node);
        }

        node.update_metadata();
        Some(node)
    }

    /// Finds a piece at the given offset
    pub fn find_piece_at_offset(&self, offset: usize) -> Option<&Piece> {
        Self::find_piece_at_offset_recursive(self.root.as_deref(), offset)
    }

    /// Recursive helper for finding piece at offset
    fn find_piece_at_offset_recursive(node: Option<&RBNode>, offset: usize) -> Option<&Piece> {
        let node = node?;

        if offset < node.left_subtree_length {
            // Target is in left subtree
            Self::find_piece_at_offset_recursive(node.left.as_deref(), offset)
        } else if offset < node.left_subtree_length + node.piece.length {
            // Target is in this node
            Some(&node.piece)
        } else {
            // Target is in right subtree
            let right_offset = offset - node.left_subtree_length - node.piece.length;
            Self::find_piece_at_offset_recursive(node.right.as_deref(), right_offset)
        }
    }

    /// Finds pieces at the given line
    pub fn find_pieces_at_line(&self, line: usize) -> Vec<&Piece> {
        let mut pieces = Vec::new();
        Self::find_pieces_at_line_recursive(self.root.as_deref(), line, &mut pieces);
        pieces
    }

    /// Recursive helper for finding pieces at line
    fn find_pieces_at_line_recursive<'a>(
        node: Option<&'a RBNode>,
        line: usize,
        pieces: &mut Vec<&'a Piece>,
    ) {
        let node = match node {
            Some(node) => node,
            None => return,
        };

        if line < node.left_subtree_line_breaks {
            // Target line is in left subtree
            Self::find_pieces_at_line_recursive(node.left.as_deref(), line, pieces);
        } else if line < node.left_subtree_line_breaks + node.piece.line_break_count {
            // Target line intersects with this node
            pieces.push(&node.piece);
        } else {
            // Target line is in right subtree
            let right_line = line - node.left_subtree_line_breaks - node.piece.line_break_count;
            Self::find_pieces_at_line_recursive(node.right.as_deref(), right_line, pieces);
        }
    }

    /// Collects all pieces in order
    pub fn collect_pieces(&self) -> Vec<&Piece> {
        let mut pieces = Vec::new();
        Self::collect_pieces_recursive(self.root.as_deref(), &mut pieces);
        pieces
    }

    /// Recursive helper for collecting pieces
    fn collect_pieces_recursive<'a>(node: Option<&'a RBNode>, pieces: &mut Vec<&'a Piece>) {
        if let Some(node) = node {
            Self::collect_pieces_recursive(node.left.as_deref(), pieces);
            pieces.push(&node.piece);
            Self::collect_pieces_recursive(node.right.as_deref(), pieces);
        }
    }

    // Red-Black Tree helper methods
    fn is_red(node: &Option<Box<RBNode>>) -> bool {
        node.as_ref().map(|n| n.is_red()).unwrap_or(false)
    }

    fn is_red_left_child(node: &Option<Box<RBNode>>) -> bool {
        node.as_ref()
            .and_then(|n| n.left.as_ref())
            .map(|n| n.is_red())
            .unwrap_or(false)
    }

    fn rotate_left(mut node: Box<RBNode>) -> Box<RBNode> {
        let mut new_root = node.right.take().unwrap();
        node.right = new_root.left.take();
        new_root.color = node.color;
        node.color = Color::Red;

        node.update_metadata();
        new_root.left = Some(node);
        new_root.update_metadata();

        new_root
    }

    fn rotate_right(mut node: Box<RBNode>) -> Box<RBNode> {
        let mut new_root = node.left.take().unwrap();
        node.left = new_root.right.take();
        new_root.color = node.color;
        node.color = Color::Red;

        node.update_metadata();
        new_root.right = Some(node);
        new_root.update_metadata();

        new_root
    }

    fn flip_colors(node: &mut Box<RBNode>) {
        node.color = Color::Red;
        if let Some(ref mut left) = node.left {
            left.color = Color::Black;
        }
        if let Some(ref mut right) = node.right {
            right.color = Color::Black;
        }
    }
}

impl Default for RBTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Piece;

    #[test]
    fn test_empty_tree() {
        let tree = RBTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.total_length(), 0);
        assert_eq!(tree.total_line_breaks(), 0);
    }

    #[test]
    fn test_single_piece_tree() {
        let piece = Piece::original(0, 0, 10, 2);
        let tree = RBTree::from_piece(piece.clone());

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.total_length(), 10);
        assert_eq!(tree.total_line_breaks(), 2);
    }

    #[test]
    fn test_insert_and_find() {
        let mut tree = RBTree::new();
        let piece1 = Piece::original(0, 0, 10, 1);
        let piece2 = Piece::original(0, 10, 15, 2);

        tree.insert(piece1.clone());
        tree.insert(piece2.clone());

        assert_eq!(tree.len(), 2);
        assert_eq!(tree.total_length(), 25);
        assert_eq!(tree.total_line_breaks(), 3);

        // Find piece at offset 5 (should be piece1)
        let found = tree.find_piece_at_offset(5);
        assert!(found.is_some());
        assert_eq!(found.unwrap().start, 0);

        // Find piece at offset 15 (should be piece2)
        let found = tree.find_piece_at_offset(15);
        assert!(found.is_some());
        assert_eq!(found.unwrap().start, 10);
    }

    #[test]
    fn test_collect_pieces() {
        let mut tree = RBTree::new();
        let piece1 = Piece::original(0, 0, 10, 1);
        let piece2 = Piece::original(0, 10, 15, 2);
        let piece3 = Piece::added(0, 0, 5, 0);

        tree.insert(piece1.clone());
        tree.insert(piece2.clone());
        tree.insert(piece3.clone());

        let pieces = tree.collect_pieces();
        assert_eq!(pieces.len(), 3);
    }

    #[test]
    fn test_node_metadata() {
        let piece = Piece::original(0, 0, 10, 2);
        let mut node = RBNode::new(piece);

        assert_eq!(node.left_subtree_length, 0);
        assert_eq!(node.left_subtree_line_breaks, 0);
        assert_eq!(node.total_length(), 10);
        assert_eq!(node.total_line_breaks(), 2);

        node.update_metadata();
        assert_eq!(node.left_subtree_length, 0);
        assert_eq!(node.left_subtree_line_breaks, 0);
    }
}
