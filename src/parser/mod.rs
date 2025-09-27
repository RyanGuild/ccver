use core::str;
use interpreter::InterpreterResult;

use crate::logs::Subject;
use crate::{logs::Logs, version::Version, version_format::VersionFormat};

#[cfg(test)]
mod tests;

mod macros;
use macros::{cc_parse, cc_parse_format, cc_parse_with_data};

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
