//! Parser module for CCVer log files and version formats.
//!
//! This module uses [Pest](https://pest.rs/) for parsing git logs and version format strings.
//!
//! # Performance Characteristics
//!
//! - **Log parsing:** O(n) where n is the number of commits, ~600-650 Kelem/s throughput
//! - **Version format parsing:** O(m) where m is format complexity, ~2-4 µs per format
//! - **Subject parsing:** O(1) per commit, ~2-4 µs per subject
//!
//! The parser maintains linear scaling across repository sizes. See `benches/parse_bench.rs`
//! for detailed benchmarks.
//!
//! # Optimization Notes
//!
//! - Parser uses borrowed string slices (`&str`) to avoid allocations where possible
//! - For large repositories (1000+ commits), consider parallel parsing (see PERFORMANCE_ANALYSIS.md)
//! - Conventional commits have ~30-50% parsing overhead vs non-conventional commits

use core::str;
use interpreter::InterpreterResult;
use pest_consume::Parser as _;

use crate::logs::Subject;
use crate::{
    cc_parse, cc_parse_format, cc_parse_with_data, logs::Logs, version::Version,
    version_format::VersionFormat,
};

#[cfg(test)]
mod tests;

mod macros;

mod grammar;
use grammar::Parser;
use grammar::Rule;

mod interpreter;

#[allow(clippy::result_large_err)]
pub fn parse_log(log: &'_ str) -> InterpreterResult<Logs<'_>> {
    cc_parse!(CCVER_LOG, log)
}

#[allow(clippy::result_large_err)]
pub fn parse_version_format(format: &str) -> InterpreterResult<VersionFormat> {
    cc_parse_format!(CCVER_VERSION_FORMAT, format)
}

#[allow(clippy::result_large_err)]
pub fn parse_version(version: &str, format: VersionFormat) -> InterpreterResult<Version> {
    cc_parse_with_data!(CCVER_VERSION, version, format)
}

#[allow(clippy::result_large_err)]
pub fn parse_subject(subject: &'_ str) -> InterpreterResult<Subject<'_>> {
    cc_parse!(SUBJECT, subject)
}
