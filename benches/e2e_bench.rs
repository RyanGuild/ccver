use ccver::graph::MemoizedCommitGraph;
use ccver::parser::{parse_log, parse_version_format};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

mod fixtures;
use fixtures::*;

/// Benchmark the complete end-to-end workflow: parse -> graph -> version
fn bench_e2e_complete_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_complete_workflow");

    let sizes = [10, 50, 100, 500, 1000, 5000];

    for size in sizes.iter() {
        let log_str = generate_mock_git_log(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &log_str, |b, log| {
            b.iter(|| {
                // Phase 1: Parse version format
                let version_format = parse_version_format(black_box(SIMPLE_FORMAT)).unwrap();

                // Phase 2: Parse git log
                let logs = parse_log(black_box(log)).unwrap();

                // Phase 3: Build commit graph
                let graph = MemoizedCommitGraph::new(logs, &version_format);

                // Phase 4: Get current version (simulating command execution)
                if let Some(head) = graph.head() {
                    let head_guard = head.lock().unwrap();
                    let version = head_guard.version.clone();
                    black_box(version)
                } else {
                    None
                }
            });
        });
    }

    group.finish();
}

/// Benchmark each phase separately to identify bottlenecks
fn bench_e2e_phase_breakdown(c: &mut Criterion) {
    let sizes = [10, 50, 100, 500, 1000, 5000];

    // Phase 1: Parse version format
    {
        let mut group = c.benchmark_group("e2e_phase_1_parse_format");
        group.bench_function("parse_version_format", |b| {
            b.iter(|| parse_version_format(black_box(SIMPLE_FORMAT)));
        });
        group.finish();
    }

    // Phase 2: Parse git log
    {
        let mut group = c.benchmark_group("e2e_phase_2_parse_log");
        for size in sizes.iter() {
            let log_str = generate_mock_git_log(*size);
            group.throughput(Throughput::Elements(*size as u64));
            group.bench_with_input(BenchmarkId::from_parameter(size), &log_str, |b, log| {
                b.iter(|| parse_log(black_box(log)));
            });
        }
        group.finish();
    }

    // Phase 3: Build commit graph
    {
        let mut group = c.benchmark_group("e2e_phase_3_build_graph");
        let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

        for size in sizes.iter() {
            let log_str = generate_mock_git_log(*size);
            let logs = parse_log(&log_str).unwrap();

            group.throughput(Throughput::Elements(*size as u64));
            group.bench_with_input(BenchmarkId::from_parameter(size), &logs, |b, logs| {
                b.iter(|| {
                    MemoizedCommitGraph::new(black_box(logs.clone()), black_box(&version_format))
                });
            });
        }
        group.finish();
    }

    // Phase 4: Get version from graph
    {
        let mut group = c.benchmark_group("e2e_phase_4_get_version");
        let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

        for size in sizes.iter() {
            let log_str = generate_mock_git_log(*size);
            let logs = parse_log(&log_str).unwrap();
            let graph = MemoizedCommitGraph::new(logs, &version_format);

            group.bench_with_input(BenchmarkId::from_parameter(size), &graph, |b, g| {
                b.iter(|| {
                    if let Some(head) = g.head() {
                        let head_guard = head.lock().unwrap();
                        black_box(head_guard.version.clone())
                    } else {
                        None
                    }
                });
            });
        }
        group.finish();
    }
}

/// Benchmark with different version format complexities
fn bench_e2e_format_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_format_complexity");

    let formats = [
        ("simple", SIMPLE_FORMAT),
        ("medium", MEDIUM_FORMAT),
        ("complex", COMPLEX_FORMAT),
        ("very_complex", VERY_COMPLEX_FORMAT),
    ];

    let log_str = generate_mock_git_log(100);

    for (name, format) in formats.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), format, |b, fmt| {
            b.iter(|| {
                let version_format = parse_version_format(black_box(fmt)).unwrap();
                let logs = parse_log(black_box(&log_str)).unwrap();
                let graph = MemoizedCommitGraph::new(logs, &version_format);

                if let Some(head) = graph.head() {
                    let head_guard = head.lock().unwrap();
                    black_box(head_guard.version.clone())
                } else {
                    None
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_e2e_complete_workflow,
    bench_e2e_phase_breakdown,
    bench_e2e_format_complexity,
);
criterion_main!(benches);
