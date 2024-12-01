//! StreamRegex: High-performance pattern matching for streaming data
//!
//! This library provides zero-storage pattern matching capabilities
//! optimized for streaming environments.

#![warn(missing_docs)]

mod error;
mod matcher;
mod pattern;

#[cfg(feature = "python")]
pub mod ffi;

pub use error::Error;
pub use matcher::StreamMatcher;
pub use pattern::{Pattern, PatternBuilder, compile_pattern};

/// Result type for StreamRegex operations
pub type Result<T> = std::result::Result<T, Error>;

// Re-export common types for convenience
pub mod prelude {
    pub use crate::Pattern;
    pub use crate::PatternBuilder;
    pub use crate::StreamMatcher;
    pub use crate::Result;
    pub use crate::Error;
    pub use crate::compile_pattern;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_matching() {
        let pattern = compile_pattern("test").unwrap();
        let mut matcher = StreamMatcher::new();
        let mut matches = Vec::new();

        matcher.add_callback(|pattern_id| {
            matches.push(pattern_id.to_string());
        });

        matcher.add_pattern(pattern);
        matcher.process_chunk(b"this is a test string");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "test");
    }

    #[test]
    fn test_memory_usage() {
        let pattern = compile_pattern("pattern").unwrap();
        let matcher = StreamMatcher::new();

        // Memory usage should be relatively constant
        let initial_usage = matcher.memory_usage();
        let mut big_data = vec![0u8; 1_000_000];
        matcher.process_chunk(&big_data);

        // Memory shouldn't grow significantly with input size
        assert!(matcher.memory_usage() - initial_usage < 1024 * 10); // Less than 10KB growth
    }
}