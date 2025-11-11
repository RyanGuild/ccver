use ccver::changelog::ChangeLogData;
use ccver::graph::MemoizedCommitGraph;
use ccver::parser::{parse_log, parse_version_format};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

mod fixtures;
use fixtures::*;

/// Benchmark version retrieval operations
fn bench_version_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_retrieval");

    let sizes = [10, 50, 100, 500, 1000, 5000];
    let version_format =
        parse_version_format(SIMPLE_FORMAT).expect("failed to parse version format");

    for size in sizes.iter() {
        let log_str = Box::leak(generate_mock_git_log(*size).into_boxed_str());
        let logs = parse_log(log_str).expect("failed to parse log");
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        group.bench_with_input(
            BenchmarkId::new("get_head_version", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    if let Some(head) = g.head() {
                        let head_guard = head.lock().expect("failed to lock head");
                        black_box(head_guard.version.clone())
                    } else {
                        None
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark tag command operations
fn bench_tag_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_operations");

    use petgraph::visit::{DfsPostOrder, Walker};

    let sizes = [10, 50, 100, 500, 1000, 5000];
    let version_format =
        parse_version_format(SIMPLE_FORMAT).expect("failed to parse version format");

    for size in sizes.iter() {
        let log_str = Box::leak(generate_mock_git_log(*size).into_boxed_str());
        let logs = parse_log(log_str).expect("failed to parse log");
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("collect_all_versions", size),
            &graph,
            |b, g| {
                b.iter(|| {
                    if let Some(head_idx) = g.head_idx() {
                        let versions: Vec<_> = DfsPostOrder::new(g.base_graph(), head_idx)
                            .iter(g.base_graph())
                            .map(|idx| {
                                let weight = g
                                    .node_weight(idx)
                                    .unwrap()
                                    .lock()
                                    .expect("failed to lock node weight");
                                weight.version.clone()
                            })
                            .collect();
                        black_box(versions)
                    } else {
                        vec![]
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark changelog generation for various graph sizes
fn bench_changelog_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("changelog_generation");

    let sizes = [10, 50, 100, 500, 1000, 5000];
    let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

    for size in sizes.iter() {
        let log_str = Box::leak(generate_mock_git_log(*size).into_boxed_str());
        let logs = parse_log(log_str).unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &logs, |b, logs| {
            b.iter(|| {
                let graph =
                    MemoizedCommitGraph::new(black_box(logs.clone()), black_box(&version_format));
                let changelog = ChangeLogData::new(&graph).unwrap();
                black_box(changelog.to_string())
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_version_retrieval,
    bench_tag_operations,
    bench_changelog_generation,
);
criterion_main!(benches);
