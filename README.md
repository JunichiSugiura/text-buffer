# TextBuffer - Piece Tree Implementation in Rust

A high-performance text buffer implementation using the piece tree data structure, inspired by [VS Code's text buffer reimplementation](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation).

## Features

- **Efficient Text Operations**: O(log n) complexity for insertions, deletions, and lookups
- **Memory Efficient**: Uses piece tree structure to minimize memory overhead
- **UTF-8 Support**: Full Unicode support with proper character handling
- **Line-based Operations**: Fast line content access and manipulation
- **Builder Pattern**: Convenient API for constructing text buffers from chunks

## Architecture

The implementation consists of several key components:

### 1. Buffer (`src/buffer.rs`)
- Stores text content and pre-computed line break positions
- Supports both original content (from files) and added content (from edits)
- Provides efficient line-based access methods

### 2. Piece (`src/piece.rs`)
- Represents a contiguous segment of text from a buffer
- Contains metadata like buffer index, start position, length, and line break count
- Supports splitting and sub-piece operations

### 3. Red-Black Tree (`src/red_black_tree.rs`)
- Self-balancing binary search tree for storing pieces
- Maintains additional metadata for efficient offset and line-based lookups
- Provides O(log n) insertion, deletion, and search operations

### 4. TextBuffer (`src/text_buffer.rs`)
- Main API combining all components
- Provides high-level operations like insert, delete, and content access
- Supports position-to-offset and offset-to-position conversions

## Usage

### Basic Usage

```rust
use text_buffer::TextBuffer;

// Create a text buffer from string
let mut buffer = TextBuffer::from_text("Hello\nWorld\n");

// Get line content
let line0 = buffer.get_line_content(0).unwrap(); // "Hello"
let line1 = buffer.get_line_content(1).unwrap(); // "World"

// Get buffer information
println!("Lines: {}", buffer.line_count()); // 3
println!("Length: {}", buffer.length()); // 12
```

### Using the Builder

```rust
use text_buffer::TextBufferBuilder;

let mut builder = TextBufferBuilder::new();
builder.accept_chunk("Hello");
builder.accept_chunk("\n");
builder.accept_chunk("World");

let buffer = builder.build();
assert_eq!(buffer.get_all_text(), "Hello\nWorld");
```

### Position and Range Operations

```rust
use text_buffer::{TextBuffer, Position, Range};

let buffer = TextBuffer::from_text("Hello\nWorld\n");

// Convert between positions and offsets
let pos = Position::new(1, 2); // Line 1, Column 2
let offset = buffer.position_to_offset(pos).unwrap();
let back_to_pos = buffer.offset_to_position(offset).unwrap();

// Get text in range
let range = Range::new(Position::new(0, 0), Position::new(1, 5));
let text = buffer.get_text_in_range(range).unwrap(); // "Hello\nWorld"
```

## Performance Characteristics

| Operation | Time Complexity | Space Complexity |
|-----------|----------------|------------------|
| Insert text | O(log n) | O(1) |
| Delete text | O(log n) | O(1) |
| Get line content | O(log n + k) | O(k) |
| Position ↔ Offset | O(log n + k) | O(1) |
| Get text in range | O(log n + k) | O(k) |

Where:
- `n` = number of pieces in the tree
- `k` = length of the requested content

## Implementation Details

### Piece Tree Structure

The piece tree maintains two types of buffers:
- **Original buffers**: Read-only content from files
- **Added buffers**: Append-only content from user edits

Each piece references a segment of text from one of these buffers, avoiding the need to copy or move large amounts of text during editing operations.

### Red-Black Tree Properties

The red-black tree stores pieces and maintains additional metadata:
- `left_subtree_length`: Total byte length of left subtree
- `left_subtree_line_breaks`: Total line breaks in left subtree

This metadata enables efficient offset-based and line-based lookups without traversing the entire tree.

### Memory Efficiency

- Original file content is stored once and never modified
- Edits are stored separately in append-only buffers
- Pieces contain only small metadata (buffer index, offset, length)
- No large string concatenations or array reallocations

## Testing

Run the test suite:

```bash
cargo test
```

The implementation includes comprehensive tests covering:
- Basic buffer operations
- Piece manipulation and splitting
- Red-black tree insertion and lookup
- Text buffer API operations
- Position and offset conversions
- UTF-8 handling

## Limitations and Future Work

This implementation provides a solid foundation but has some areas for improvement:

1. **Simplified Insertion/Deletion**: The current implementation doesn't fully handle piece splitting during insertions or complex deletions
2. **Tree Balancing**: While the red-black tree provides good performance, the insertion logic could be optimized for text editing patterns
3. **Memory Compaction**: Long editing sessions might benefit from periodic buffer compaction
4. **Undo/Redo**: The piece tree structure naturally supports undo/redo, but this isn't implemented yet

## References

- [VS Code Text Buffer Reimplementation](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation)
- [The Piece Table - the Unsung Hero of Your Text Editor](https://dev.to/_darrenburns/the-piece-table---the-unsung-hero-of-your-text-editor-al8)
- [Data Structures for Text Sequences](https://www.cs.unm.edu/~crowley/papers/sds.pdf)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 
