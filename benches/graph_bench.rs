use ccver::graph::MemoizedCommitGraph;
use ccver::parser::{parse_log, parse_version_format};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

mod fixtures;
use fixtures::*;

/// Benchmark complete graph construction from logs of varying sizes
fn bench_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_construction");

    let sizes = [10, 50, 100, 500, 1000];
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

/// Benchmark graph operations: parent/child lookups
fn bench_graph_parent_child_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_parent_child_ops");

    let sizes = [10, 50, 100, 500, 1000];
    let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

    for size in sizes.iter() {
        let log_str = generate_mock_git_log(*size);
        let logs = parse_log(&log_str).unwrap();
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        if let Some(head_idx) = graph.head_idx() {
            group.bench_with_input(
                BenchmarkId::new("parent_idxs", size),
                &(&graph, head_idx),
                |b, (g, idx)| {
                    b.iter(|| g.parent_idxs(black_box(*idx)));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("child_idxs", size),
                &(&graph, head_idx),
                |b, (g, idx)| {
                    b.iter(|| g.child_idxs(black_box(*idx)));
                },
            );
        }
    }

    group.finish();
}

/// Benchmark commit lookup by hash
fn bench_graph_commit_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_commit_lookup");

    let sizes = [10, 50, 100, 500, 1000, 5000];
    let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

    for size in sizes.iter() {
        let log_str = generate_mock_git_log(*size);
        let logs = parse_log(&log_str).unwrap();
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        // Look up the middle commit
        let target_hash = format!("{:040x}", size / 2);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(&graph, target_hash.as_str()),
            |b, (g, hash)| {
                b.iter(|| g.commit_by_hash(black_box(hash)));
            },
        );
    }

    group.finish();
}

/// Benchmark graph traversal operations
fn bench_graph_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_traversal");

    use petgraph::visit::{Bfs, Dfs, Walker};

    let sizes = [10, 50, 100, 500, 1000, 5000];
    let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

    for size in sizes.iter() {
        let log_str = generate_mock_git_log(*size);
        let logs = parse_log(&log_str).unwrap();
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        if let Some(head_idx) = graph.head_idx() {
            group.throughput(Throughput::Elements(*size as u64));

            group.bench_with_input(
                BenchmarkId::new("bfs", size),
                &(&graph, head_idx),
                |b, (g, idx)| {
                    b.iter(|| {
                        let nodes: Vec<_> = Bfs::new(g.base_graph(), *idx)
                            .iter(g.base_graph())
                            .collect();
                        black_box(nodes)
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("dfs", size),
                &(&graph, head_idx),
                |b, (g, idx)| {
                    b.iter(|| {
                        let nodes: Vec<_> = Dfs::new(g.base_graph(), *idx)
                            .iter(g.base_graph())
                            .collect();
                        black_box(nodes)
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark graph node and edge counting operations
fn bench_graph_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_metrics");

    let sizes = [10, 50, 100, 500, 1000];
    let version_format = parse_version_format(SIMPLE_FORMAT).unwrap();

    for size in sizes.iter() {
        let log_str = generate_mock_git_log(*size);
        let logs = parse_log(&log_str).unwrap();
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        group.bench_with_input(BenchmarkId::new("node_count", size), &graph, |b, g| {
            b.iter(|| black_box(g.node_count()));
        });

        group.bench_with_input(BenchmarkId::new("edge_count", size), &graph, |b, g| {
            b.iter(|| black_box(g.edge_count()));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_graph_construction,
    bench_graph_parent_child_operations,
    bench_graph_commit_lookup,
    bench_graph_traversal,
    bench_graph_metrics,
);
criterion_main!(benches);
