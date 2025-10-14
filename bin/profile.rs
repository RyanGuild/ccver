/// Profiling utility for ccver
///
/// This binary provides ad-hoc profiling capabilities for various ccver operations.
/// It generates flamegraphs and protobuf profiles for performance analysis.
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;

/// Generate mock git log for profiling
fn generate_mock_git_log(num_commits: usize) -> String {
    let mut log = String::new();
    for i in 0..num_commits {
        let commit_hash = format!("{:040x}", i);
        let parent_hash = if i > 0 {
            format!("{:040x}", i - 1)
        } else {
            String::new()
        };

        let subject = match i % 5 {
            0 => format!("feat: add feature {}", i),
            1 => format!("fix: fix bug {}", i),
            2 => format!("chore: update dependency {}", i),
            3 => format!("docs: update documentation {}", i),
            _ => format!("refactor: refactor code {}", i),
        };

        let decoration = if i == num_commits - 1 {
            " (HEAD -> main)"
        } else if i % 10 == 0 {
            " (tag: v0.1.0)"
        } else {
            ""
        };

        log.push_str(&format!(
            "name=\nTest-User-{}\nbranch=\nmain\ncommit=\n{}\ncommit-time=\n2024-01-01T00:00:00Z\ndec=\n{}\nparent=\n{}\nsub=\n{}\ntrailers=\n\n",
            i, commit_hash, decoration, parent_hash, subject
        ));
    }
    log
}

/// Output format for profiling results
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Generate flamegraph SVG files (default)
    Flamegraph,
}

/// Operation to profile
#[derive(Debug, Clone, Copy, ValueEnum)]
enum Operation {
    /// Profile log parsing
    Parse,
    /// Profile graph construction
    Graph,
    /// Profile end-to-end workflow
    E2E,
    /// Profile all operations
    All,
}

/// Profiling utility for ccver
///
/// Generate flamegraphs and profiles for various ccver operations
#[derive(Parser, Debug)]
#[command(name = "profile")]
#[command(about = "Profile ccver operations and generate flamegraphs", long_about = None)]
struct Args {
    /// Operation to profile
    #[arg(value_enum)]
    operation: Operation,

    /// Number of commits to use for profiling
    size: usize,

    /// Output format for profiling results
    #[arg(short, long, value_enum, default_value = "flamegraph")]
    format: OutputFormat,

    /// Output directory for profiling results
    #[arg(short, long, default_value = "target/profiling")]
    output: PathBuf,
}

fn profile_parse(
    size: usize,
    output_dir: &PathBuf,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    use ccver::parser::parse_log;

    println!("Profiling parse operation with {} commits...", size);
    let log_str = generate_mock_git_log(size);

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000) // Increased from 100 to 1000 Hz for more samples
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()?;

    // Run the operation multiple times for better profiling data
    for _ in 0..100 {
        let _ = parse_log(&log_str)?;
    }

    save_profile(guard, output_dir, "parse", format)?;
    println!("✓ Parse profiling complete");
    Ok(())
}

fn profile_graph(
    size: usize,
    output_dir: &PathBuf,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    use ccver::graph::MemoizedCommitGraph;
    use ccver::parser::{parse_log, parse_version_format};

    println!("Profiling graph construction with {} commits...", size);
    let log_str = generate_mock_git_log(size);
    let logs = parse_log(&log_str)?;
    let version_format = parse_version_format("vCC.CC.CC")?;

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000) // Increased from 100 to 1000 Hz for more samples
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()?;

    // Run the operation multiple times for better profiling data
    for _ in 0..100 {
        let _graph = MemoizedCommitGraph::new(logs.clone(), &version_format);
    }

    save_profile(guard, output_dir, "graph", format)?;
    println!("✓ Graph profiling complete");
    Ok(())
}

fn profile_e2e(
    size: usize,
    output_dir: &PathBuf,
    format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    use ccver::graph::MemoizedCommitGraph;
    use ccver::parser::{parse_log, parse_version_format};

    println!("Profiling end-to-end workflow with {} commits...", size);
    let log_str = generate_mock_git_log(size);

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000) // Increased from 100 to 1000 Hz for more samples
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()?;

    // Run the complete workflow multiple times
    for _ in 0..100 {
        let version_format = parse_version_format("vCC.CC.CC")?;
        let logs = parse_log(&log_str)?;
        let graph = MemoizedCommitGraph::new(logs, &version_format);

        // Simulate getting version
        if let Some(head) = graph.head() {
            let _version = head.lock().unwrap().version.clone();
        }
    }

    save_profile(guard, output_dir, "e2e", format)?;
    println!("✓ E2E profiling complete");
    Ok(())
}

fn save_profile(
    guard: pprof::ProfilerGuard,
    output_dir: &PathBuf,
    name: &str,
    _format: &OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = guard
        .report()
        .build()
        .map_err(|e| format!("Failed to build profiling report: {:?}", e))?;

    let flamegraph_path = output_dir.join(format!("{}.svg", name));
    let file = fs::File::create(&flamegraph_path)?;
    report.flamegraph(file)?;
    println!("  → Flamegraph: {}", flamegraph_path.display());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output)?;

    println!("ccver profiling utility");
    println!("=======================");
    println!("Operation: {:?}", args.operation);
    println!("Size: {} commits", args.size);
    println!("Format: {:?}", args.format);
    println!("Output: {}", args.output.display());
    println!();

    match args.operation {
        Operation::Parse => profile_parse(args.size, &args.output, &args.format)?,
        Operation::Graph => profile_graph(args.size, &args.output, &args.format)?,
        Operation::E2E => profile_e2e(args.size, &args.output, &args.format)?,
        Operation::All => {
            profile_parse(args.size, &args.output, &args.format)?;
            profile_graph(args.size, &args.output, &args.format)?;
            profile_e2e(args.size, &args.output, &args.format)?;
        }
    }

    println!("\nProfiling complete! View flamegraphs:");
    println!("  open {}", args.output.display());

    Ok(())
}
