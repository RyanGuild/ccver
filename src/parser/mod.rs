use core::str;
use interpreter::InterpreterResult;

use crate::graph::CommitGraph;
use crate::{logs::Logs, version::Version, version_format::VersionFormat};

#[cfg(test)]
mod tests;

mod macros;
use macros::{cc_parse, cc_parse_format, cc_parse_with_data};

mod grammar;
use grammar::Parser;
use grammar::Rule;

mod interpreter;

pub fn parse_log(log: &'_ str) -> InterpreterResult<Logs<'_>> {
    cc_parse!(CCVER_LOG, log)
}

pub fn parse_version_format<'graph>(
    format: &str,
    graph: &'graph CommitGraph<'graph>,
) -> InterpreterResult<VersionFormat<'graph>> {
    cc_parse_format!(CCVER_VERSION_FORMAT, format, graph)
}

pub fn parse_version(version: &str, format: VersionFormat) -> InterpreterResult<Version> {
    cc_parse_with_data!(CCVER_VERSION, version, format)
}
