use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::sync::Arc;
use crate::{StreamMatcher, Pattern, PatternBuilder};

/// Python wrapper for StreamMatcher
#[pyclass]
pub struct PyStreamMatcher {
    matcher: Arc<StreamMatcher>,
}

#[pymethods]
impl PyStreamMatcher {
    #[new]
    fn new() -> Self {
        PyStreamMatcher {
            matcher: Arc::new(StreamMatcher::new())
        }
    }

    fn add_pattern(&mut self, pattern: &str, pattern_id: Option<String>) -> PyResult<String> {
        let id = pattern_id.unwrap_or_else(|| format!("pattern_{}", self.matcher.patterns.len()));
        let mut builder = PatternBuilder::new();

        // Convert the Python pattern string to our Pattern type
        // This is simplified - you'd want more robust pattern parsing
        let pattern = builder.build(id.clone());

        Arc::get_mut(&mut self.matcher)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to get mutable reference"))?
            .add_pattern(pattern);

        Ok(id)
    }

    fn process_chunk(&mut self, data: &[u8]) -> PyResult<()> {
        Arc::get_mut(&mut self.matcher)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to get mutable reference"))?
            .process_chunk(data);
        Ok(())
    }

    fn memory_usage(&self) -> usize {
        self.matcher.memory_usage()
    }
}

// Module initialization
#[pymodule]
fn streamregex_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyStreamMatcher>()?;
    Ok(())
}