use ccver::parser::{parse_log, parse_subject, parse_version, parse_version_format};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

mod fixtures;
use fixtures::*;

/// Benchmark parsing version format strings of varying complexity
fn bench_parse_version_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_version_format");

    let formats = [
        ("simple", SIMPLE_FORMAT),
        ("medium", MEDIUM_FORMAT),
        ("complex", COMPLEX_FORMAT),
        ("very_complex", VERY_COMPLEX_FORMAT),
    ];

    for (name, format) in formats.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), format, |b, fmt| {
            b.iter(|| parse_version_format(black_box(fmt)));
        });
    }

    group.finish();
}

/// Benchmark parsing git log strings of varying sizes
fn bench_parse_log(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_log");

    let sizes = [10, 50, 100, 500, 1000, 5000];

    for size in sizes.iter() {
        let log = generate_mock_git_log(*size);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &log, |b, log| {
            b.iter(|| parse_log(black_box(log)));
        });
    }

    group.finish();
}

/// Benchmark parsing conventional commit subjects
fn bench_parse_conventional_subject(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_subject_conventional");

    for (idx, subject) in CONVENTIONAL_COMMITS.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(idx), subject, |b, subj| {
            b.iter(|| parse_subject(black_box(subj)));
        });
    }

    group.finish();
}

/// Benchmark parsing non-conventional commit subjects
fn bench_parse_non_conventional_subject(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_subject_non_conventional");

    for (idx, subject) in NON_CONVENTIONAL_COMMITS.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(idx), subject, |b, subj| {
            b.iter(|| parse_subject(black_box(subj)));
        });
    }

    group.finish();
}

/// Benchmark parsing version strings
fn bench_parse_version(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_version");

    // Parse a simple version format first
    let format = parse_version_format(SIMPLE_FORMAT).unwrap();

    // Simple versions that match SIMPLE_FORMAT
    let simple_versions = ["v1.0.0", "v2.3.4", "v10.20.30", "v0.0.1"];

    for (idx, version_str) in simple_versions.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(idx), version_str, |b, ver| {
            b.iter(|| {
                let _ = parse_version(black_box(ver), black_box(format.clone()));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_version_format,
    bench_parse_log,
    bench_parse_conventional_subject,
    bench_parse_non_conventional_subject,
    bench_parse_version,
);
criterion_main!(benches);
