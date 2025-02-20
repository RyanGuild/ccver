use core::str;
use lexer::{LexerInput, LexerResult};
use std::rc::Rc;

use crate::{logs::Log, version::Version, version_format::VersionFormat};

#[cfg(test)]
mod tests;

mod macros;
use macros::{cc_parse, cc_parse_with_data};

mod grammar;
use grammar::Parser;
use grammar::Rule;

mod lexer;

pub fn parse_log(log: &str) -> LexerResult<Log> {
    cc_parse!(CCVER_LOG, log)
}

pub fn parse_version_format(format: &str) -> LexerResult<VersionFormat> {
    cc_parse!(CCVER_VERSION_FORMAT, format)
}

pub fn parse_version<'input, 'format>(
    version: &'input str,
    format: VersionFormat<'format>,
) -> LexerResult<Version<'input>> {
    cc_parse_with_data!(
        CCVER_VERSION,
        version,
        LexerInput {
            version_format: Rc::new(format)
        }
    )
}
