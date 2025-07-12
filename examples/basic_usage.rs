use text_buffer::{Position, Range, TextBuffer, TextBufferBuilder};

fn main() {
    println!("=== TextBuffer Basic Usage Example ===\n");

    // Example 1: Creating a text buffer from a string
    println!("1. Creating TextBuffer from string:");
    let buffer = TextBuffer::from_text("Hello, World!\nThis is line 2.\nAnd this is line 3.");

    println!("   Total lines: {}", buffer.line_count());
    println!("   Total length: {} bytes", buffer.length());
    println!("   Is empty: {}", buffer.is_empty());

    // Example 2: Getting line content
    println!("\n2. Getting line content:");
    for i in 0..buffer.line_count() {
        match buffer.get_line_content(i) {
            Ok(content) => println!("   Line {}: '{}'", i, content),
            Err(e) => println!("   Error getting line {}: {}", i, e),
        }
    }

    // Example 3: Using the builder pattern
    println!("\n3. Using TextBufferBuilder:");
    let mut builder = TextBufferBuilder::new();
    builder.accept_chunk("First chunk");
    builder.accept_chunk("\n");
    builder.accept_chunk("Second chunk");
    builder.accept_chunk("\n");
    builder.accept_chunk("Third chunk");

    let built_buffer = builder.build();
    println!("   Built buffer content: '{}'", built_buffer.get_all_text());
    println!("   Built buffer lines: {}", built_buffer.line_count());

    // Example 4: Position and offset conversions
    println!("\n4. Position and offset conversions:");
    let pos = Position::new(1, 5); // Line 1, Column 5
    println!("   Position: Line {}, Column {}", pos.line, pos.column);

    match buffer.position_to_offset(pos) {
        Ok(offset) => {
            println!("   Converted to offset: {}", offset);
            match buffer.offset_to_position(offset) {
                Ok(back_pos) => println!(
                    "   Converted back to position: Line {}, Column {}",
                    back_pos.line, back_pos.column
                ),
                Err(e) => println!("   Error converting back: {}", e),
            }
        }
        Err(e) => println!("   Error converting position: {}", e),
    }

    // Example 5: Getting text in range
    println!("\n5. Getting text in range:");
    let range = Range::new(Position::new(0, 0), Position::new(0, 5));
    match buffer.get_text_in_range(range) {
        Ok(text) => println!("   Text in range: '{}'", text),
        Err(e) => println!("   Error getting range: {}", e),
    }

    // Example 6: Line length
    println!("\n6. Line lengths:");
    for i in 0..buffer.line_count() {
        match buffer.get_line_length(i) {
            Ok(length) => println!("   Line {} length: {} characters", i, length),
            Err(e) => println!("   Error getting line {} length: {}", i, e),
        }
    }

    // Example 7: Demonstrating UTF-8 support
    println!("\n7. UTF-8 support:");
    let utf8_buffer = TextBuffer::from_text("Hello 世界\nこんにちは\n🦀 Rust");
    println!("   UTF-8 content: '{}'", utf8_buffer.get_all_text());
    println!("   UTF-8 lines: {}", utf8_buffer.line_count());

    for i in 0..utf8_buffer.line_count() {
        if let Ok(content) = utf8_buffer.get_line_content(i) {
            if let Ok(length) = utf8_buffer.get_line_length(i) {
                println!("   Line {}: '{}' ({} characters)", i, content, length);
            }
        }
    }

    println!("\n=== Example completed ===");
}
