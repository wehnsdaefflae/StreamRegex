use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Core types for pattern matching
#[derive(Debug, Clone)]
pub struct Pattern {
    pub(crate) id: String,
    pub(crate) states: Vec<State>,
    pub(crate) initial_state: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct State {
    transitions: HashMap<u8, usize>,
    is_final: bool,
}

// StreamMatcher is the main interface for pattern matching
pub struct StreamMatcher {
    patterns: Vec<Pattern>,
    current_states: Vec<usize>,
    memory_usage: Arc<AtomicUsize>,
    callbacks: Vec<Box<dyn Fn(&str) + Send + Sync>>,
}

impl StreamMatcher {
    pub fn new() -> Self {
        StreamMatcher {
            patterns: Vec::new(),
            current_states: Vec::new(),
            memory_usage: Arc::new(AtomicUsize::new(0)),
            callbacks: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.current_states.push(pattern.initial_state);
        self.patterns.push(pattern);
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn process_byte(&mut self, byte: u8) {
        for (pattern_idx, current_state) in self.current_states.iter_mut().enumerate() {
            let pattern = &self.patterns[pattern_idx];

            if let Some(next_state) = pattern.states[*current_state].transitions.get(&byte) {
                *current_state = *next_state;

                if pattern.states[*current_state].is_final {
                    for callback in &self.callbacks {
                        callback(&pattern.id);
                    }
                }
            } else {
                *current_state = pattern.initial_state;
            }
        }
    }

    pub fn process_chunk(&mut self, data: &[u8]) {
        for &byte in data {
            self.process_byte(byte);
        }
    }

    pub fn memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }
}