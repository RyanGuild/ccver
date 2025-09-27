use super::{Parser, Rule};
use crate::parser::interpreter::ParserInputs;

use pest_consume::Parser as _;

pub macro cc_parse($rule:ident, $str_ref:expr) {
    match Parser::parse_with_userdata(Rule::$rule, $str_ref, ParserInputs::LogParsing(None)) {
        Err(e) => Err(e),
        Ok(parsed) => match parsed.single() {
            Err(e) => Err(e),
            Ok(single) => match Parser::$rule(single) {
                Err(e) => Err(e),
                Ok(hydrated) => Ok(hydrated),
            },
        },
    }
}

pub macro cc_parse_with_data($rule:ident, $str_ref:expr, $data:expr) {
    match Parser::parse_with_userdata(Rule::$rule, $str_ref, ParserInputs::LogParsing(Some($data)))
    {
        Err(e) => Err(e),
        Ok(parsed) => match parsed.single() {
            Err(e) => Err(e),
            Ok(single) => match Parser::$rule(single) {
                Err(e) => Err(e),
                Ok(hydrated) => Ok(hydrated),
            },
        },
    }
}

pub macro cc_parse_format($rule:ident, $str_ref:expr) {
    match Parser::parse_with_userdata(Rule::$rule, $str_ref, ParserInputs::FormatParsing) {
        Err(e) => Err(e),
        Ok(parsed) => match parsed.single() {
            Err(e) => Err(e),
            Ok(single) => match Parser::$rule(single) {
                Err(e) => Err(e),
                Ok(hydrated) => Ok(hydrated),
            },
        },
    }
}

pub macro parsing_error($input:expr, $message:expr) {
    pest_consume::Error::new_from_span(
        pest::error::ErrorVariant::CustomError {
            message: $message.to_string(),
        },
        $input.as_span(),
    )
}
