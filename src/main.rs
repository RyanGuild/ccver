pub mod args;
pub mod changelog;
pub mod current_version;
pub mod git;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod pattern_macros;
pub mod peek;
pub mod tag;
pub mod version;
pub mod version_format;

use std::io::Read as _;

use args::*;
use changelog::ChangeLogData;
use clap::Parser;
use eyre::*;
use git::git_installed;
use logs::GIT_FORMAT_ARGS;
use logs::Logs;
use tracing::{Level, debug, error, info, span};
use tracing_error::ErrorLayer;
use tracing_subscriber::Layer as _;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::graph::MemoizedCommitGraph;

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
            info!(path = ?parsed_args.path, "Reading logs from path");
            Logs::from_path(&parsed_args.path)?
        }
    };

    info!("Logs count: {}", logs.len());

    let version_format = {
        let _format_span = span!(Level::INFO, "parse_version_format").entered();

        info!(format = %parsed_args.format, "Parsing custom version format");
        parser::parse_version_format(&parsed_args.format).map_err(|e| {
            error!(error = %e, format = %parsed_args.format, "Failed to parse version format");
            e
        })?
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
        debug!("Executing command: {:?}", parsed_args.command);
        let result = match parsed_args.command {
            None => {
                if let Some(ref changelog_path) = parsed_args.changelog_path {
                    let changelog = ChangeLogData::new(&graph).map_err(|e| {
                        error!(error = %e, "Failed to generate changelog");
                        e
                    })?;
                    std::fs::write(changelog_path, format!("{}", changelog))?;
                };
                let version = if parsed_args.message.is_none() {
                    let _default_command_span = span!(Level::INFO, "default_command").entered();
                    format!(
                        "{}",
                        crate::current_version::current_version_handler(
                            &graph,
                            &parsed_args,
                            &version_format
                        )?
                    )
                } else {
                    crate::peek::peek_command_handler(&graph, &parsed_args, &version_format)?
                };

                if parsed_args.tag || parsed_args.tag_all {
                    crate::tag::tag_command_handler(&graph, &parsed_args, &version_format)?;
                }

                version
            }
            Some(CCVerSubCommand::GitFormat) => {
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
        };

        result
    };

    println!("{}", stdout);
    info!("ccver application completed successfully");

    Ok(())
}
