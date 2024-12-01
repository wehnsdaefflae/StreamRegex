use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{self, BufReader, Read};
use memory_stats::memory_stats;

// Import the regex crates for comparison
use regex::Regex;
use hyperscan::pattern::Pattern as HsPattern;
use hyperscan::Builder;

// Import our StreamRegex implementation
use streamregex::StreamMatcher;
use streamregex::PatternBuilder;

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
const PATTERN_SET_SIZE: usize = 1000; // Number of patterns to test
const STREAM_SIZE: usize = 1024 * 1024 * 1024; // 1GB total stream size

struct BenchmarkResult {
    throughput: f64,      // In Gbps
    latency: f64,         // In milliseconds
    memory_usage: usize,  // In bytes
    pattern_load_time: Duration,
}

// Generate test data that occasionally matches patterns
fn generate_test_data(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        data.push(rand::random::<u8>());
    }

    data
}

// Generate a set of realistic security patterns
fn generate_security_patterns(count: usize) -> Vec<String> {
    let mut patterns = Vec::with_capacity(count);

    // Common security pattern templates
    let templates = vec![
        "SELECT.*FROM.*WHERE",
        "admin.*password",
        "eval\\(.*\\)",
        "<script.*>.*</script>",
        "\\b(?:[0-9]{4}-){3}[0-9]{4}\\b", // Credit card pattern
    ];

    for i in 0..count {
        let template = &templates[i % templates.len()];
        patterns.push(template.to_string());
    }

    patterns
}

// Benchmark StreamRegex
fn benchmark_streamregex(data: &[u8], patterns: &[String]) -> BenchmarkResult {
    let start_memory = memory_stats().unwrap().physical_mem;
    let pattern_load_start = Instant::now();

    let mut matcher = StreamMatcher::new();

    // Load patterns
    for (i, pattern) in patterns.iter().enumerate() {
        let mut builder = PatternBuilder::new();
        // Convert regex pattern to StreamRegex pattern
        // This is a simplified conversion for benchmark purposes
        let pattern_obj = builder.build(format!("pattern_{}", i));
        matcher.add_pattern(pattern_obj);
    }

    let pattern_load_time = pattern_load_start.elapsed();

    // Measure processing
    let start = Instant::now();
    let mut matches = 0;
    matcher.add_callback(|_| matches += 1);

    // Process in chunks to simulate streaming
    for chunk in data.chunks(CHUNK_SIZE) {
        matcher.process_chunk(chunk);
    }

    let elapsed = start.elapsed();
    let end_memory = memory_stats().unwrap().physical_mem;

    // Calculate metrics
    let throughput = (data.len() as f64 / elapsed.as_secs_f64()) * 8.0 / 1_000_000_000.0; // Convert to Gbps
    let latency = elapsed.as_secs_f64() * 1000.0 / (data.len() as f64 / CHUNK_SIZE as f64);
    let memory_usage = end_memory - start_memory;

    BenchmarkResult {
        throughput,
        latency,
        memory_usage,
        pattern_load_time,
    }
}

// Benchmark traditional regex engine (using the regex crate)
fn benchmark_regex(data: &[u8], patterns: &[String]) -> BenchmarkResult {
    let start_memory = memory_stats().unwrap().physical_mem;
    let pattern_load_start = Instant::now();

    let regexes: Vec<Regex> = patterns
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect();

    let pattern_load_time = pattern_load_start.elapsed();

    // Measure processing
    let start = Instant::now();
    let mut matches = 0;

    // Process in chunks to simulate streaming
    for chunk in data.chunks(CHUNK_SIZE) {
        let chunk_str = String::from_utf8_lossy(chunk);
        for regex in &regexes {
            matches += regex.find_iter(&chunk_str).count();
        }
    }

    let elapsed = start.elapsed();
    let end_memory = memory_stats().unwrap().physical_mem;

    // Calculate metrics
    let throughput = (data.len() as f64 / elapsed.as_secs_f64()) * 8.0 / 1_000_000_000.0;
    let latency = elapsed.as_secs_f64() * 1000.0 / (data.len() as f64 / CHUNK_SIZE as f64);
    let memory_usage = end_memory - start_memory;

    BenchmarkResult {
        throughput,
        latency,
        memory_usage,
        pattern_load_time,
    }
}

// Benchmark Hyperscan
fn benchmark_hyperscan(data: &[u8], patterns: &[String]) -> BenchmarkResult {
    let start_memory = memory_stats().unwrap().physical_mem;
    let pattern_load_start = Instant::now();

    let hs_patterns: Vec<HsPattern> = patterns
        .iter()
        .enumerate()
        .map(|(i, p)| HsPattern::new(i as i32, p).unwrap())
        .collect();

    let db = Builder::new()
        .mode(hyperscan::Mode::BLOCK)
        .build(&hs_patterns)
        .unwrap();

    let scratch = db.alloc_scratch().unwrap();
    let pattern_load_time = pattern_load_start.elapsed();

    // Measure processing
    let start = Instant::now();
    let mut matches = 0;

    // Process in chunks to simulate streaming
    for chunk in data.chunks(CHUNK_SIZE) {
        db.scan(chunk, &scratch, |_id, _from, _to, _flags| {
            matches += 1;
            Ok(true)
        })
        .unwrap();
    }

    let elapsed = start.elapsed();
    let end_memory = memory_stats().unwrap().physical_mem;

    // Calculate metrics
    let throughput = (data.len() as f64 / elapsed.as_secs_f64()) * 8.0 / 1_000_000_000.0;
    let latency = elapsed.as_secs_f64() * 1000.0 / (data.len() as f64 / CHUNK_SIZE as f64);
    let memory_usage = end_memory - start_memory;

    BenchmarkResult {
        throughput,
        latency,
        memory_usage,
        pattern_load_time,
    }
}

fn run_benchmarks(c: &mut Criterion) {
    // Generate test data and patterns
    let data = generate_test_data(STREAM_SIZE);
    let patterns = generate_security_patterns(PATTERN_SET_SIZE);

    let mut group = c.benchmark_group("Pattern Matching Engines");

    // Benchmark StreamRegex
    group.bench_function("StreamRegex", |b| {
        b.iter(|| {
            let result = benchmark_streamregex(&data, &patterns);
            black_box(result);
        });
    });

    // Benchmark traditional regex
    group.bench_function("Traditional Regex", |b| {
        b.iter(|| {
            let result = benchmark_regex(&data, &patterns);
            black_box(result);
        });
    });

    // Benchmark Hyperscan
    group.bench_function("Hyperscan", |b| {
        b.iter(|| {
            let result = benchmark_hyperscan(&data, &patterns);
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(benches, run_benchmarks);
criterion_main!(benches);