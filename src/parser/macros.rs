use super::{Parser, Rule};
use pest_consume::Parser as _;

pub macro cc_parse($rule:ident, $str_ref:expr) {
    match Parser::parse_with_userdata(Rule::$rule, $str_ref, None) {
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
    match Parser::parse_with_userdata(Rule::$rule, $str_ref, Some($data)) {
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

#[allow(unused_macros)]
pub macro dbg_cc_parse($rule:ident, $str_ref:expr) {
    match Parser::parse_with_userdata(Rule::$rule, $str_ref, None) {
        Err(e) => Err(e),
        Ok(parsed) => {
            dbg!(&parsed);
            match parsed.single() {
                Err(e) => Err(e),
                Ok(single) => match Parser::$rule(single) {
                    Err(e) => Err(e),
                    Ok(hydrated) => {
                        dbg!(&hydrated);
                        Ok(hydrated)
                    }
                },
            }
        }
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
