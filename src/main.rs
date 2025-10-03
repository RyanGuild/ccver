#![feature(decl_macro, lock_value_accessors, iterator_try_collect)]

/// The main entry point for the `ccver` application.
///
/// This function parses command-line arguments, initializes logging, and
/// executes the specified command. It supports version formatting and tagging
/// operations.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` if the operation is successful, otherwise
///   returns an error.
///
/// # Errors
///
/// This function will return an error if:
///
/// * The current directory cannot be determined.
/// * The version format cannot be parsed.
/// * The logs cannot be retrieved or processed.
/// * The `git` command is not installed.
///
/// # Example
///
/// ```sh
/// ccver --path /path/to/repo --format "vYY.CC.CC-pre.<short-sha>" tag
/// ```
pub mod args;
pub mod changelog;
pub mod git;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod pattern_macros;
pub mod version;
pub mod version_format;

use std::env::current_dir;
use std::io::Read as _;
use std::path::Path;
use std::path::PathBuf;

use crate::graph::version::TaggedVersionExt as _;
use crate::logs::PeekLogEntry;
use crate::version::Version;
use crate::version_format::VersionFormat;
use args::*;
use ccver::git::is_dirty;
use changelog::ChangeLogData;
use clap::Parser;
use eyre::*;
use git::git_installed;
use logs::GIT_FORMAT_ARGS;
use logs::Logs;
use petgraph::visit::DfsPostOrder;
use petgraph::visit::Walker as _;
use tracing::{Level, debug, error, info, instrument, span, warn};
use tracing_error::ErrorLayer;
use tracing_subscriber::Layer as _;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::graph::MemoizedCommitGraph;
use crate::graph::version::ExistingVersionExt;
use crate::logs::InfersVersionFormat as _;

#[instrument]
fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(
            fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
        )
        .init();

    let _main_span = span!(Level::INFO, "ccver_main").entered();
    info!("Starting ccver application");

    // Check git installation early
    if let Err(e) = git_installed() {
        error!(error = %e, "Git installation check failed");
        return Err(e);
    }

    let parsed_args = CCVerArgs::parse();
    debug!("Parsed command line arguments: {:?}", parsed_args);

    let command = match parsed_args.command {
        Some(command) => {
            info!("Using command from args: {:?}", command);
            Some(command)
        }
        None => match std::env::var("INPUT_COMMAND") {
            std::result::Result::Ok(command) => {
                info!("Using command from environment: {}", command);
                match command.to_ascii_lowercase().trim() {
                    "tag" => Some(CCVerSubCommand::Tag(TagArgs {
                        all: std::env::var("INPUT_COMMAND_TAG_ALL").unwrap_or_default() != "0"
                            && std::env::var("INPUT_COMMAND_TAG_ALL").unwrap_or_default()
                                != "false",
                    })),
                    "changelog" => Some(CCVerSubCommand::ChangeLog),
                    "git-format" => Some(CCVerSubCommand::GitFormat),
                    "peek" => Some(CCVerSubCommand::Peek(PeekArgs {
                        message: std::env::var("INPUT_COMMAND_PEEK_MESSAGE").unwrap_or_default(),
                    })),
                    _ => None,
                }
            }
            Err(_) => {
                debug!("No command specified, will generate version");
                None
            }
        },
    };

    let no_pre = match parsed_args.no_pre {
        true => {
            info!("Using no pre from args: true");
            true
        }
        false => match std::env::var("INPUT_NO_PRE") {
            std::result::Result::Ok(no_pre) => {
                info!("Using no pre from environment: {}", no_pre);
                no_pre != "0" && no_pre != "false"
            }
            Err(_) => {
                info!("Using no pre: false");
                false
            }
        },
    };

    let ci = match parsed_args.ci {
        true => {
            info!("Using ci from args: true");
            true
        }
        false => match std::env::var("INPUT_CI") {
            std::result::Result::Ok(ci) => {
                let ci = ci != "0" && ci != "false";
                info!("Using ci from environment: {}", ci);
                ci
            }
            Err(_) => {
                info!("Using ci: false");
                false
            }
        },
    };

    let format = match parsed_args.format {
        Some(format) => {
            info!("Using format from args: {:?}", format);
            Some(format)
        }
        None => match std::env::var("INPUT_FORMAT") {
            std::result::Result::Ok(format) => {
                info!("Using format from environment: {}", format);
                Some(format)
            }
            Err(_) => {
                info!("Using format: none");
                None
            }
        },
    };

    let path = match parsed_args.path {
        Some(path) => {
            info!("Using path from args: {:?}", path);
            PathBuf::from(path)
        }
        None => match std::env::var("INPUT_PATH") {
            std::result::Result::Ok(path) => {
                info!("Using path from environment: {}", path);
                PathBuf::from(path)
            }
            Err(_) => {
                let current = current_dir().expect("could not get current dir");
                debug!("Using current directory: {:?}", current);
                current
            }
        },
    };

    let mut stdin_string = String::new();

    let logs = {
        let _logs_span = span!(Level::INFO, "load_logs", raw = parsed_args.raw).entered();
        if parsed_args.raw {
            info!("Reading logs from stdin");
            std::io::stdin()
                .read_to_string(&mut stdin_string)
                .map_err(|e| {
                    error!(error = %e, "Failed to read from stdin");
                    e
                })?;
            Logs::from_log_str(stdin_string.leak())?
        } else {
            info!(path = ?path, "Reading logs from path");
            Logs::from_path(&path)?
        }
    };

    info!("Logs count: {}", logs.len());

    let version_format = {
        let _format_span = span!(Level::INFO, "parse_version_format").entered();
        if let Some(format_str) = format {
            info!(format = %format_str, "Parsing custom version format");
            parser::parse_version_format(&format_str).map_err(|e| {
                error!(error = %e, format = %format_str, "Failed to parse version format");
                e
            })?
        } else {
            debug!("Using default version format");
            logs.infer_version_format()
        }
    };

    let graph = {
        let _graph_span = span!(Level::INFO, "build_commit_graph").entered();
        info!("Building commit graph");
        let graph = MemoizedCommitGraph::new(logs.clone(), &version_format);
        info!(
            "Commit graph node count: {} edge count: {}",
            graph.node_count(),
            graph.edge_count()
        );
        debug!("Commit graph created successfully");
        graph
    };

    let stdout = {
        let _command_span = span!(Level::INFO, "execute_command").entered();
        match command {
            None => format!(
                "{}",
                get_current_version(&graph, &path, ci, no_pre, &version_format)?
            ),
            Some(command) => match command {
                CCVerSubCommand::Peek(args) => {
                    let _peek_span =
                        span!(Level::INFO, "peek_command", message = %args.message).entered();
                    let parent_commit = graph.head().unwrap().lock().unwrap().log_entry.commit_hash;
                    let branch = graph.head().unwrap().lock().unwrap().log_entry.branch;
                    let next_entry = args
                        .message
                        .leak()
                        .into_peek_log_entry(parent_commit, branch);
                    let next_version = graph
                        .head()
                        .unwrap()
                        .as_existing_version()
                        .map(|v| v.next_version(&next_entry, &version_format))
                        .unwrap_or_else(|| version_format.as_default_version(&next_entry));

                    debug!(version = %next_version, "Peek result");
                    if no_pre {
                        format!("{}", next_version.no_pre())
                    } else {
                        format!("{}", next_version)
                    }
                }
                CCVerSubCommand::ChangeLog => {
                    let _changelog_span = span!(Level::INFO, "changelog_command").entered();
                    info!("Generating changelog");
                    let changelog = ChangeLogData::new(graph).map_err(|e| {
                        error!(error = %e, "Failed to generate changelog");
                        e
                    })?;
                    debug!("Changelog generated successfully");
                    format!("{}", changelog)
                }
                CCVerSubCommand::GitFormat => {
                    let _git_format_span = span!(Level::DEBUG, "git_format_command").entered();
                    info!("Outputting git format args");
                    format!(
                        "{} {} {} {} {}",
                        GIT_FORMAT_ARGS[0],
                        GIT_FORMAT_ARGS[1],
                        GIT_FORMAT_ARGS[2],
                        GIT_FORMAT_ARGS[3],
                        GIT_FORMAT_ARGS[4]
                    )
                }
                CCVerSubCommand::Tag(args) => {
                    if is_dirty(&path)? {
                        return Err(eyre!("Repo is dirty while tag is true"));
                    }
                    let _tag_span = span!(Level::INFO, "tag_command", all = args.all).entered();
                    info!("Tagging with all: {}", args.all);
                    let version = get_current_version(&graph, &path, ci, no_pre, &version_format)?;
                    if !args.all {
                        git::tag_commit_with_version(
                            &graph.head().unwrap().lock().unwrap().log_entry.commit_hash,
                            &version,
                            &path,
                        )?;
                        format!("{}", version)
                    } else {
                        let new_versions =
                            DfsPostOrder::new(graph.base_graph(), graph.head_idx().unwrap())
                                .iter(graph.base_graph())
                                .map(|idx| {
                                    let weight = graph.node_weight(idx).unwrap().lock().unwrap();
                                    let version = weight.as_existing_version().expect(
                                        "A version was not assigned to a node in the graph",
                                    );

                                    let tagged_version = weight.log_entry.as_tagged_version();
                                    if tagged_version.is_none() {
                                        let _ = git::tag_commit_with_version(
                                            &weight.log_entry.commit_hash,
                                            &version,
                                            &path,
                                        );
                                    }

                                    Ok(format!("{}", version))
                                })
                                .try_collect::<Vec<_>>()?;

                        format!("{}", new_versions.join("\n"))
                    }
                }
            },
        }
    };

    println!("{}", stdout);
    info!("ccver application completed successfully");

    Ok(())
}

#[instrument(skip(graph))]
fn get_current_version(
    graph: &MemoizedCommitGraph,
    path: &Path,
    ci: bool,
    no_pre: bool,
    version_format: &VersionFormat,
) -> Result<Version> {
    debug!("Using default command to get current version");
    return match is_dirty(&path) {
        Result::Ok(dirty) => {
            if ci && dirty {
                Err(eyre!("Repo is dirty while ci is true"))
            } else {
                let head = graph.head();
                debug!("Head: {:#?}", head);
                let head = head.unwrap().lock().unwrap();
                let version = head
                    .version
                    .clone()
                    .ok_or_eyre(eyre!("Current Branch Head Was Not Assigned a Version"));
                version.map(|v| v.build(&head.log_entry, &version_format))
            }
        }
        Err(e) => Err(e),
    }
    .map(|v| {
        info!("Version: {:?}", v);
        if no_pre {
            v.release(
                &graph.head().unwrap().lock().unwrap().log_entry,
                &version_format,
            )
        } else {
            v
        }
    })
    .map_err(|e| {
        error!(error = %e, "Failed to get current version");
        e
    });
}
