use streamregex::prelude::*;
use std::io::Cursor;
use std::sync::mpsc;
use std::thread;

#[test]
fn test_large_stream_processing() {
    let pattern = compile_pattern("needle").unwrap();
    let mut matcher = StreamMatcher::new();
    let (tx, rx) = mpsc::channel();

    matcher.add_callback(move |pattern_id| {
        tx.send(pattern_id.to_string()).unwrap();
    });

    matcher.add_pattern(pattern);

    // Create a large stream with known matches
    let mut data = vec![b'x'; 1_000_000];
    data[500_000..500_006].copy_from_slice(b"needle");
    data[750_000..750_006].copy_from_slice(b"needle");

    // Process in chunks
    let mut cursor = Cursor::new(data);
    let mut buffer = vec![0; 1024];

    while let Ok(n) = cursor.read(&mut buffer) {
        if n == 0 { break; }
        matcher.process_chunk(&buffer[..n]);
    }

    // Check we found exactly two matches
    let matches: Vec<_> = rx.try_iter().collect();
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_concurrent_processing() {
    let pattern = compile_pattern("test").unwrap();
    let matcher = StreamMatcher::new();
    let matcher = std::sync::Arc::new(std::sync::Mutex::new(matcher));

    let mut handles = vec![];

    // Create multiple threads processing data
    for i in 0..4 {
        let matcher = matcher.clone();
        let handle = thread::spawn(move || {
            let data = format!("thread{} test data", i);
            matcher.lock().unwrap().process_chunk(data.as_bytes());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_memory_usage_over_time() {
    let pattern = compile_pattern("memory").unwrap();
    let mut matcher = StreamMatcher::new();
    matcher.add_pattern(pattern);

    let initial_memory = matcher.memory_usage();

    // Process increasing amounts of data
    for size in [1024, 10_240, 102_400, 1_024_000] {
        let data = vec![b'x'; size];
        matcher.process_chunk(&data);

        // Memory usage should remain relatively constant
        let current_memory = matcher.memory_usage();
        assert!(
            (current_memory as i64 - initial_memory as i64).abs() < 1024 * 10,
            "Memory usage grew too much: {} vs {}",
            current_memory,
            initial_memory
        );
    }
}

#[test]
fn test_pattern_matching_accuracy() {
    let test_cases = vec![
        ("abc", "xxabcxx", 1),
        ("a+b+c+", "aabcc", 1),
        ("password=\\w+", "password=secret123", 1),
        ("needle", "haystackneedlemore", 1),
        ("multi", "multimulti", 2),
    ];

    for (pattern, input, expected_matches) in test_cases {
        let pattern = compile_pattern(pattern).unwrap();
        let mut matcher = StreamMatcher::new();
        let mut matches = 0;

        matcher.add_callback(move |_| matches += 1);
        matcher.add_pattern(pattern);
        matcher.process_chunk(input.as_bytes());

        assert_eq!(
            matches,
            expected_matches,
            "Pattern '{}' on input '{}' produced {} matches, expected {}",
            pattern,
            input,
            matches,
            expected_matches
        );
    }
}