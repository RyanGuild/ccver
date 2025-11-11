#!/usr/bin/env rust
//! Local GitHub Actions workflow runner using act
//!
//! This tool orchestrates running GitHub Actions workflows locally using act,
//! handling workflow dependencies and generating required secrets automatically.

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// Local GitHub Actions workflow runner using act
#[derive(Parser, Debug)]
#[command(name = "act-runner")]
#[command(about = "Local GitHub Actions workflow orchestration", long_about = None)]
#[command(
    after_help = "Run 'list' command to see all available workflows discovered from .github/workflows/"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Set up .env file with GitHub token
    Setup,
    /// List available workflows
    List,
    /// Run a specific workflow or 'all' for pre-release suite
    Run {
        /// Workflow name to run (or 'all' for complete pre-release suite)
        workflow: String,
    },
}

#[derive(Debug, Clone)]
struct Workflow {
    name: String,
    file: String,
    description: String,
    deps: Vec<String>,
}

/// Discover workflows from .github/workflows directory
fn discover_workflows() -> Vec<Workflow> {
    let workflows_dir = Path::new(".github/workflows");

    if !workflows_dir.exists() {
        eprintln!("Warning: .github/workflows directory not found");
        return Vec::new();
    }

    let mut workflows = Vec::new();

    // Read all .yml and .yaml files from workflows directory
    if let Ok(entries) = fs::read_dir(workflows_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only process .yml and .yaml files
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    if let Some(workflow) = parse_workflow_file(&path) {
                        workflows.push(workflow);
                    }
                }
            }
        }
    }

    // Sort by name for consistent ordering
    workflows.sort_by(|a, b| a.name.cmp(&b.name));

    workflows
}

/// Parse a workflow file to extract metadata
fn parse_workflow_file(path: &Path) -> Option<Workflow> {
    let content = fs::read_to_string(path).ok()?;
    let filename = path.file_name()?.to_str()?;

    // Extract workflow name from the 'name:' field
    let name = content
        .lines()
        .find(|line| line.trim_start().starts_with("name:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
        .unwrap_or(filename.trim_end_matches(".yml").trim_end_matches(".yaml"));

    // Create a simple name for CLI usage (lowercase, hyphenated)
    let cli_name = filename
        .trim_end_matches(".yml")
        .trim_end_matches(".yaml")
        .to_string();

    // Try to extract dependencies from workflow_call jobs
    let mut deps = Vec::new();
    let mut in_jobs = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("jobs:") {
            in_jobs = true;
            continue;
        }

        if in_jobs {
            // Look for 'needs:' or 'uses:' to find dependencies
            if trimmed.starts_with("needs:") {
                if let Some(needs_str) = trimmed.split(':').nth(1) {
                    let needs_str = needs_str.trim();
                    if needs_str.starts_with('[') {
                        // Array format: needs: [build, test]
                        for dep in needs_str.trim_matches(|c| c == '[' || c == ']').split(',') {
                            let dep = dep.trim();
                            if !dep.is_empty() {
                                deps.push(dep.to_string());
                            }
                        }
                    } else {
                        // Single dependency
                        deps.push(needs_str.to_string());
                    }
                }
            }

            // Check for workflow_call uses (like in cicd.yml)
            if trimmed.starts_with("uses:") && trimmed.contains("./") {
                if let Some(uses_path) = trimmed.split(':').nth(1) {
                    let workflow_file = uses_path
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_start_matches("./");

                    // Extract the workflow name from the path
                    if let Some(dep_name) = Path::new(workflow_file)
                        .file_stem()
                        .and_then(|s| s.to_str())
                    {
                        deps.push(dep_name.to_string());
                    }
                }
            }
        }
    }

    // Deduplicate dependencies
    deps.sort();
    deps.dedup();

    Some(Workflow {
        name: cli_name,
        file: path.to_string_lossy().to_string(),
        description: name.to_string(),
        deps,
    })
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Setup => setup_environment(),
        Commands::List => list_workflows(),
        Commands::Run { workflow } => {
            ensure_env_file();
            run_workflows(&workflow);
        }
    }
}

fn list_workflows() {
    let workflows = discover_workflows();

    if workflows.is_empty() {
        println!("No workflows found in .github/workflows/");
        return;
    }

    println!("Available workflows:\n");
    for workflow in workflows {
        let deps = if workflow.deps.is_empty() {
            "none".to_string()
        } else {
            workflow.deps.join(", ")
        };
        println!("  {:12} - {}", workflow.name, workflow.description);
        println!("               File: {}", workflow.file);
        println!("               Dependencies: {}", deps);
        println!();
    }
}

fn ensure_env_file() {
    if !std::path::Path::new(".env").exists() {
        eprintln!("Warning: .env file not found. Run 'cargo run --bin act-runner -- setup' first.");
        eprintln!("Attempting to create .env file automatically...");
        setup_environment();
    }
}

fn setup_environment() {
    println!("Setting up local act environment...\n");

    // Check if gh CLI is available
    let gh_check = Command::new("gh").arg("--version").output();

    let token = if gh_check.is_ok() {
        println!("Getting GitHub token from gh CLI...");
        let output = Command::new("gh")
            .args(&["auth", "token"])
            .output()
            .expect("Failed to get token from gh CLI");

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            eprintln!(
                "Failed to get token from gh CLI. Please install and authenticate with gh CLI:"
            );
            eprintln!("  brew install gh");
            eprintln!("  gh auth login");
            std::process::exit(1);
        }
    } else {
        eprintln!("gh CLI not found. Please install it:");
        eprintln!("  brew install gh");
        eprintln!("  gh auth login");
        std::process::exit(1);
    };

    // Create .env file
    let env_content = format!(
        "# GitHub Actions Environment Variables for act\n\
         # Auto-generated by act-runner\n\
         \n\
         GITHUB_TOKEN={}\n\
         \n\
         # Apple signing secrets (optional - steps will be skipped in local runs)\n\
         # APPLE_CERTIFICATE_DATA=\n\
         # APPLE_CERTIFICATE_PASSWORD=\n\
         # APPLE_TEAM_ID=\n",
        token
    );

    fs::write(".env", env_content).expect("Failed to write .env file");
    println!("✓ Created .env file with GitHub token");

    // Determine container architecture based on host
    let host_arch = env::consts::ARCH;
    let container_arch = match host_arch {
        "x86_64" => "linux/amd64",
        "aarch64" => "linux/arm64",
        "arm" => "linux/arm/v7",
        _ => "linux/amd64", // fallback
    };

    // Create .actrc file
    let actrc_content = format!(
        "# act configuration for local GitHub Actions testing\n\
         # Auto-generated by act-runner setup\n\
         # Use smallest valid image for faster performance\n\
         --container-architecture={}\n\
         --platform=ubuntu-latest=catthehacker/ubuntu:act-latest\n\
         --platform=ubuntu-24.04-arm=catthehacker/ubuntu:act-latest\n\
         # Note: Windows and macOS runners are not mapped - jobs using these runners\n\
         # will be automatically skipped by act when the platform is unavailable\n\
         # - Windows MSVC builds: require link.exe (not available in Linux containers)\n\
         # - macOS builds: require native macOS runners (cross doesn't support macOS from Linux)\n\
         # Both will build successfully on GitHub Actions with native runners\n\
         \n\
         # Artifact storage\n\
         --artifact-server-path=.act-artifacts\n\
         \n\
         # Default secrets file\n\
         --secret-file=.env\n\
         \n\
         # Use host network for faster downloads\n\
         --use-gitignore=true\n\
         \n\
         # Verbose output (commented out to reduce debug logging)\n\
         # --verbose\n",
        container_arch
    );

    fs::write(".actrc", actrc_content).expect("Failed to write .actrc file");
    println!(
        "✓ Created .actrc file with platform mappings for {}",
        container_arch
    );

    // Check if act is installed
    let act_check = Command::new("act").arg("--version").output();

    if act_check.is_ok() {
        println!("✓ act is installed");
    } else {
        eprintln!("\n⚠ act is not installed. Please install it:");
        eprintln!("  brew install act");
    }

    println!("\n✓ Setup complete! You can now run workflows with:");
    println!("  cargo run --bin act-runner -- run <workflow-name>");
}

fn run_workflows(target: &str) {
    // Check if act is installed
    let act_check = Command::new("act").arg("--version").output();

    if act_check.is_err() {
        eprintln!("Error: act is not installed. Please install it:");
        eprintln!("  brew install act");
        std::process::exit(1);
    }

    let workflows = discover_workflows();

    if workflows.is_empty() {
        eprintln!("Error: No workflows found in .github/workflows/");
        std::process::exit(1);
    }

    if target == "all" {
        println!("Running pre-release workflow suite...\n");
        run_all_workflows(&workflows);
    } else {
        let workflow = workflows.iter().find(|w| w.name == target);
        match workflow {
            Some(w) => run_workflow(w),
            None => {
                eprintln!("Error: Unknown workflow '{}'", target);
                eprintln!("Run 'cargo run --bin act-runner -- list' to see available workflows");
                std::process::exit(1);
            }
        }
    }
}

fn run_all_workflows(workflows: &[Workflow]) {
    // Sort workflows by dependency order using topological sort
    let sorted_workflows = topological_sort(workflows);

    println!(
        "Running {} workflows in dependency order...\n",
        sorted_workflows.len()
    );

    for workflow in sorted_workflows {
        println!("\n{}", "=".repeat(60));
        println!(
            "Running workflow: {} ({})",
            workflow.name, workflow.description
        );
        println!("{}\n", "=".repeat(60));

        run_workflow(&workflow);
    }

    println!("\n{}", "=".repeat(60));
    println!("All workflows completed!");
    println!("{}", "=".repeat(60));
}

/// Extract the first job name from a workflow file
fn extract_first_job_name(content: &str) -> Option<String> {
    let mut in_jobs = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("jobs:") {
            in_jobs = true;
            continue;
        }

        if in_jobs {
            // First non-indented line with a colon after "jobs:" is usually the job name
            if !trimmed.starts_with(' ') && !trimmed.is_empty() && trimmed.contains(':') {
                return Some(trimmed.trim_end_matches(':').to_string());
            }

            // Some workflows indent job names with 2 spaces
            if trimmed.starts_with("  ") && !trimmed.starts_with("    ") {
                let job_line = trimmed.trim_start();
                if job_line.contains(':') && !job_line.starts_with('#') {
                    return Some(job_line.trim_end_matches(':').to_string());
                }
            }
        }
    }

    None
}

/// Simple topological sort for workflow dependencies
fn topological_sort(workflows: &[Workflow]) -> Vec<Workflow> {
    let mut sorted = Vec::new();
    let mut visited = HashMap::new();
    let workflow_map: HashMap<_, _> = workflows
        .iter()
        .map(|w| (w.name.clone(), w.clone()))
        .collect();

    fn visit(
        workflow: &Workflow,
        workflow_map: &HashMap<String, Workflow>,
        visited: &mut HashMap<String, bool>,
        sorted: &mut Vec<Workflow>,
    ) {
        if let Some(&true) = visited.get(&workflow.name) {
            return; // Already visited
        }

        visited.insert(workflow.name.clone(), true);

        // Visit dependencies first
        for dep in &workflow.deps {
            if let Some(dep_workflow) = workflow_map.get(dep) {
                visit(dep_workflow, workflow_map, visited, sorted);
            }
        }

        sorted.push(workflow.clone());
    }

    for workflow in workflows {
        visit(workflow, &workflow_map, &mut visited, &mut sorted);
    }

    sorted
}

fn run_workflow(workflow: &Workflow) {
    println!(
        "Running workflow: {} - {}",
        workflow.name, workflow.description
    );
    println!("File: {}", workflow.file);

    if !workflow.deps.is_empty() {
        println!("Dependencies: {}", workflow.deps.join(", "));
        println!("Note: Make sure dependencies have been run first");
    }

    println!("\nExecuting act...\n");

    // Determine the host platform for container architecture
    let host_arch = env::consts::ARCH;

    // Map Rust architecture to Docker platform architecture
    let container_arch = match host_arch {
        "x86_64" => "linux/amd64",
        "aarch64" => "linux/arm64",
        "arm" => "linux/arm/v7",
        _ => "linux/amd64", // fallback
    };

    let mut act_cmd = Command::new("act");

    // Prefer workflow_dispatch when available, fall back to job name for workflow_call only
    let workflow_content = fs::read_to_string(&workflow.file).unwrap_or_default();
    let has_workflow_dispatch = workflow_content.contains("workflow_dispatch");

    if has_workflow_dispatch {
        // Workflow supports workflow_dispatch - use it
        act_cmd.arg("workflow_dispatch");
    } else {
        // No workflow_dispatch, try to run job directly
        if let Some(job_name) = extract_first_job_name(&workflow_content) {
            println!(
                "Note: Workflow doesn't have workflow_dispatch, using job '{}'",
                job_name
            );
            act_cmd.arg("-j").arg(job_name);
        } else {
            eprintln!("Warning: Could not determine job name, trying workflow_dispatch anyway");
            act_cmd.arg("workflow_dispatch");
        }
    }

    act_cmd
        .arg("-W")
        .arg(&workflow.file)
        .arg("--container-architecture")
        .arg(container_arch);

    // Set environment variable to indicate we're running in act
    // Workflows can use this to conditionally skip steps
    act_cmd.arg("--env").arg("ACT_RUNNER=true");

    // For build workflow, note about platform limitations
    // Only Linux targets will run in act (native builds or cross-compilation with cross tool)
    // - Linux targets: native builds or cross-compilation with cross tool (musl targets)
    // - Windows targets: skipped (MSVC requires link.exe, not available in Linux containers)
    // - macOS targets: skipped (require native macOS runners, cross doesn't support macOS from Linux)
    if workflow.name == "build" {
        println!("Note: Only Linux builds will run in act");
        println!("      Windows and macOS builds will be skipped (require native runners)");
        println!("      All Linux targets (x86_64, aarch64, glibc, musl) will run in act");
        println!("      Windows and macOS builds will run successfully on GitHub Actions");
        println!("      Windows builds use static linking for better portability");
    }

    // For docker workflow, only build arm64 images in act
    if workflow.name == "docker" {
        println!("Note: Building only linux/arm64 Docker image in act");
        println!("      Multi-platform build (amd64 + arm64) will run on GitHub Actions");
        println!("      SBOM artifacts will be skipped (placeholder used)");
    }

    // For sbom workflow, use arm64 binary in act
    if workflow.name == "sbom" {
        println!("Note: Generating SBOM for arm64 binary in act");
        println!("      Full SBOM generation for both platforms on GitHub Actions");
    }

    // For profile workflow, note about signal handling limitations
    if workflow.name == "profile" {
        println!(
            "Note: Profiling workflow will fail in act (containers don't support signal-based profiling)"
        );
        println!("      Signal-based pprof profiling is required to generate flamegraphs");
        println!(
            "      Run this workflow on GitHub Actions runners for full flamegraph generation"
        );
    }

    // For test workflow, note about platform limitations
    if workflow.name == "test" {
        println!("Note: Only ubuntu-latest tests will run in act");
        println!("      Windows and macOS tests require native runners");
        println!("      All platforms will test successfully on GitHub Actions");
    }

    // For cicd workflow, provide guidance about workflow_call limitations
    if workflow.name == "cicd" {
        println!("Note: cicd workflow calls multiple workflows via workflow_call");
        println!("      Jobs using windows-latest or macos-latest will be automatically");
        println!("      skipped by act (no platform mapping) and by workflow skip logic");
        println!("      All jobs will run successfully on GitHub Actions");
        println!("      Consider running individual workflows for better act compatibility");
    }

    let status = act_cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!("\n✓ Workflow '{}' completed successfully", workflow.name);
            } else {
                eprintln!("\n✗ Workflow '{}' failed", workflow.name);
                eprintln!(
                    "Tip: Some workflows may require dependencies to be run first or may not work with act"
                );
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\n✗ Failed to execute act: {}", e);
            std::process::exit(1);
        }
    }
}
