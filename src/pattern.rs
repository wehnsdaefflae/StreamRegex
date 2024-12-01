use std::collections::HashMap;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub(crate) id: String,
    pub(crate) states: Vec<State>,
    pub(crate) initial_state: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct State {
    pub(crate) transitions: HashMap<u8, usize>,
    pub(crate) is_final: bool,
}

pub struct PatternBuilder {
    states: Vec<State>,
    transitions: Vec<(usize, u8, usize)>,
}

impl PatternBuilder {
    pub fn new() -> Self {
        PatternBuilder {
            states: vec![State {
                transitions: HashMap::new(),
                is_final: false,
            }],
            transitions: Vec::new(),
        }
    }

    pub fn add_state(&mut self, is_final: bool) -> usize {
        let state_idx = self.states.len();
        self.states.push(State {
            transitions: HashMap::new(),
            is_final,
        });
        state_idx
    }

    pub fn add_transition(&mut self, from: usize, byte: u8, to: usize) -> &mut Self {
        if from >= self.states.len() || to >= self.states.len() {
            panic!("Invalid state index");
        }
        self.transitions.push((from, byte, to));
        self
    }

    pub fn build(mut self, id: String) -> Result<Pattern, Error> {
        // Validate pattern before building
        if self.states.is_empty() {
            return Err(Error::InvalidPattern("Pattern must have at least one state".into()));
        }

        // Build transitions
        for (from, byte, to) in self.transitions {
            self.states[from].transitions.insert(byte, to);
        }

        Ok(Pattern {
            id,
            states: self.states,
            initial_state: 0,
        })
    }
}

impl Default for PatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Convert a string pattern into a state machine
// This is a simplified implementation - a real one would parse regex syntax
pub fn compile_pattern(pattern: &str) -> Result<Pattern, Error> {
    let mut builder = PatternBuilder::new();
    let mut current_state = 0;

    for (i, byte) in pattern.bytes().enumerate() {
        let next_state = builder.add_state(i == pattern.len() - 1);
        builder.add_transition(current_state, byte, next_state);
        current_state = next_state;
    }

    builder.build(pattern.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_builder() {
        let mut builder = PatternBuilder::new();
        let s1 = builder.add_state(false);
        let s2 = builder.add_state(true);

        builder
            .add_transition(0, b'a', s1)
            .add_transition(s1, b'b', s2);

        let pattern = builder.build("test".into()).unwrap();

        assert_eq!(pattern.states.len(), 3);
        assert!(pattern.states[s2].is_final);
    }

    #[test]
    fn test_compile_pattern() {
        let pattern = compile_pattern("abc").unwrap();
        assert_eq!(pattern.states.len(), 4); // initial + 3 states
        assert!(pattern.states.last().unwrap().is_final);
    }
}