use std::time::Instant;
use text_buffer::{Position, TextBufferBuilder};

fn main() {
    println!("=== TextBuffer Performance Demo ===\n");

    // Demo 1: Building a large text buffer from chunks
    println!("1. Building large text buffer from chunks:");
    let start = Instant::now();
    let mut builder = TextBufferBuilder::new();

    // Simulate reading a large file in chunks
    for i in 0..1000 {
        builder.accept_chunk(&format!("This is line {} with some content.\n", i));
    }

    let large_buffer = builder.build();
    let build_time = start.elapsed();

    println!(
        "   Built buffer with {} lines in {:?}",
        large_buffer.line_count(),
        build_time
    );
    println!("   Total length: {} bytes", large_buffer.length());

    // Demo 2: Random line access performance
    println!("\n2. Random line access performance:");
    let start = Instant::now();
    let mut total_chars = 0;

    // Access 100 random lines
    for i in (0..100).map(|x| (x * 7) % large_buffer.line_count()) {
        if let Ok(content) = large_buffer.get_line_content(i) {
            total_chars += content.len();
        }
    }

    let access_time = start.elapsed();
    println!("   Accessed 100 random lines in {:?}", access_time);
    println!("   Total characters read: {}", total_chars);

    // Demo 3: Sequential line access
    println!("\n3. Sequential line access performance:");
    let start = Instant::now();
    let mut line_count = 0;

    for i in 0..large_buffer.line_count() {
        if large_buffer.get_line_content(i).is_ok() {
            line_count += 1;
        }
    }

    let sequential_time = start.elapsed();
    println!(
        "   Accessed {} lines sequentially in {:?}",
        line_count, sequential_time
    );

    // Demo 4: Position conversion performance
    println!("\n4. Position conversion performance:");
    let start = Instant::now();
    let mut successful_conversions = 0;

    for line in (0..100).map(|x| x * 10) {
        for col in 0..10 {
            let pos = Position::new(line % large_buffer.line_count(), col);
            if let Ok(offset) = large_buffer.position_to_offset(pos) {
                if large_buffer.offset_to_position(offset).is_ok() {
                    successful_conversions += 1;
                }
            }
        }
    }

    let conversion_time = start.elapsed();
    println!(
        "   Performed {} position conversions in {:?}",
        successful_conversions, conversion_time
    );

    // Demo 5: Memory efficiency comparison
    println!("\n5. Memory efficiency demonstration:");

    // Create a buffer with repeated content to show piece tree efficiency
    let mut efficient_builder = TextBufferBuilder::new();
    let repeated_content = "This line is repeated many times to demonstrate efficiency.\n";

    // Add the same content multiple times - in a piece tree, this is very efficient
    for _ in 0..100 {
        efficient_builder.accept_chunk(repeated_content);
    }

    let efficient_buffer = efficient_builder.build();
    println!(
        "   Created buffer with {} lines of repeated content",
        efficient_buffer.line_count()
    );
    println!("   Total length: {} bytes", efficient_buffer.length());

    // Demo 6: UTF-8 handling performance
    println!("\n6. UTF-8 handling performance:");
    let utf8_content = "Hello 世界! 🦀 Rust is awesome! こんにちは\n";
    let mut utf8_builder = TextBufferBuilder::new();

    let start = Instant::now();
    for _ in 0..100 {
        utf8_builder.accept_chunk(utf8_content);
    }

    let utf8_buffer = utf8_builder.build();
    let utf8_build_time = start.elapsed();

    println!(
        "   Built UTF-8 buffer with {} lines in {:?}",
        utf8_buffer.line_count(),
        utf8_build_time
    );

    // Test UTF-8 line access
    let start = Instant::now();
    let mut utf8_char_count = 0;

    for i in 0..utf8_buffer.line_count() {
        if let Ok(length) = utf8_buffer.get_line_length(i) {
            utf8_char_count += length;
        }
    }

    let utf8_access_time = start.elapsed();
    println!(
        "   Counted {} UTF-8 characters in {:?}",
        utf8_char_count, utf8_access_time
    );

    println!("\n=== Performance Demo Completed ===");
    println!("\nKey Observations:");
    println!("- Building large buffers from chunks is efficient due to append-only structure");
    println!("- Random line access maintains good performance due to tree structure");
    println!("- Position conversions are fast with cached line break information");
    println!("- UTF-8 handling is properly supported with character-level operations");
    println!("- Memory usage is optimized through piece-based storage");
}
