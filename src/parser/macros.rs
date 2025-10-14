#[macro_export]
macro_rules! cc_parse {
    ($rule:ident, $str_ref:expr) => {
        match $crate::parser::Parser::parse_with_userdata(
            $crate::parser::Rule::$rule,
            $str_ref,
            $crate::parser::interpreter::ParserInputs::LogParsing(None),
        ) {
            Err(e) => Err(e),
            Ok(parsed) => match parsed.single() {
                Err(e) => Err(e),
                Ok(single) => match $crate::parser::Parser::$rule(single) {
                    Err(e) => Err(e),
                    Ok(hydrated) => Ok(hydrated),
                },
            },
        }
    };
}

#[macro_export]
macro_rules! cc_parse_with_data {
    ($rule:ident, $str_ref:expr, $data:expr) => {
        match $crate::parser::Parser::parse_with_userdata(
            $crate::parser::Rule::$rule,
            $str_ref,
            $crate::parser::interpreter::ParserInputs::LogParsing(Some($data)),
        ) {
            Err(e) => Err(e),
            Ok(parsed) => match parsed.single() {
                Err(e) => Err(e),
                Ok(single) => match $crate::parser::Parser::$rule(single) {
                    Err(e) => Err(e),
                    Ok(hydrated) => Ok(hydrated),
                },
            },
        }
    };
}

#[macro_export]
macro_rules! cc_parse_format {
    ($rule:ident, $str_ref:expr) => {
        match $crate::parser::Parser::parse_with_userdata(
            $crate::parser::Rule::$rule,
            $str_ref,
            $crate::parser::interpreter::ParserInputs::FormatParsing,
        ) {
            Err(e) => Err(e),
            Ok(parsed) => match parsed.single() {
                Err(e) => Err(e),
                Ok(single) => match $crate::parser::Parser::$rule(single) {
                    Err(e) => Err(e),
                    Ok(hydrated) => Ok(hydrated),
                },
            },
        }
    };
}

#[macro_export]
macro_rules! parsing_error {
    ($input:expr, $message:expr) => {
        pest_consume::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: $message.to_string(),
            },
            $input.as_span(),
        )
    };
}
