use streamregex::StreamMatcher;

let mut matcher = StreamMatcher::new();
matcher.add_callback(|pattern_id| {
    println!("Match found: {}", pattern_id);
});

// Process streaming data
matcher.process_chunk(&data);