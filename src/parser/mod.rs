use core::str;
use interpreter::InterpreterResult;

use crate::{logs::Log, version::Version, version_format::VersionFormat};

#[cfg(test)]
mod tests;

mod macros;
use macros::{cc_parse, cc_parse_with_data};

mod grammar;
use grammar::Parser;
use grammar::Rule;

mod interpreter;

pub fn parse_log(log: &str) -> InterpreterResult<Log> {
    cc_parse!(CCVER_LOG, log)
}

pub fn parse_version_format(format: &str) -> InterpreterResult<VersionFormat> {
    cc_parse!(CCVER_VERSION_FORMAT, format)
}

pub fn parse_version(version: &str, format: VersionFormat) -> InterpreterResult<Version> {
    cc_parse_with_data!(CCVER_VERSION, version, format)
}
