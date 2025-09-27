use std::collections::HashMap;
use std::sync::Arc;

use pest_consume::{Node as PestNode, *};

use crate::logs::{ConventionalSubject, Decoration, LogEntry, Subject, Tag};
use crate::version::{PreTag, VersionNumber};
use crate::version_format::CalVerFormat;
use crate::version_format::{
    CalVerFormatSegment::{self, *},
    PreTagFormat::{self, *},
    VersionNumberFormat,
};

use super::grammar::{Parser, Rule};
use super::macros::parsing_error;
use super::{Logs, Version, VersionFormat};

#[derive(Debug, Clone)]
pub enum ParserInputs {
    LogParsing(Option<VersionFormat>),
    FormatParsing,
}
macro log_parsing_context($input:expr) {{
    match $input.user_data() {
        ParserInputs::LogParsing(v) => v.clone(),
        ParserInputs::FormatParsing => panic!("expected log parsing context"),
    }
}}

pub type Node<'input> = PestNode<'input, Rule, ParserInputs>;

pub type InterpreterResult<T> = eyre::Result<T, pest_consume::Error<Rule>>;

macro pre_format($input:expr) {{
    let pre_format = log_parsing_context!($input)
        .unwrap_or_default()
        .prerelease
        .unwrap_or_default();
    pre_format.version_format()
}}

#[pest_consume::parser]
impl Parser {
    pub fn CCVER_VERSION(input: Node) -> InterpreterResult<Version> {
        let parser_input = log_parsing_context!(input).unwrap_or_default();
        match_nodes!(input.children();
            [V_PREFIX(v_prefix), VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch)] => Ok(Version {
                v_prefix,
                major: parser_input.major.parse(major),
                minor: parser_input.minor.parse(minor),
                patch: parser_input.patch.parse(patch),
                prerelease: None
            }),
            [V_PREFIX(v_prefix), VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), PRE_TAG(pretag)] => Ok(Version {
                v_prefix,
                major: parser_input.major.parse(major),
                minor: parser_input.minor.parse(minor),
                patch: parser_input.patch.parse(patch),
                prerelease: Some(pretag)
            })
        )
    }

    pub fn PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        match_nodes!(input.children();
            [SHA_PRE_TAG(s)] => Ok(s),
            [RC_PRE_TAG(r)] => Ok(r),
            [BETA_PRE_TAG(b)] => Ok(b),
            [ALPHA_PRE_TAG(a)] => Ok(a),
            [BUILD_PRE_TAG(b)] => Ok(b),
            [NAMED_PRE_TAG(n)] => Ok(n)
        )
    }

    pub fn SHA_PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        match_nodes!(input.children();
            [SHA(s)] => Ok(PreTag::Sha(VersionNumber::Sha(s.to_string()))),
            [SHORT_SHA(s)] => Ok(PreTag::ShortSha(VersionNumber::ShortSha(s.to_string())))
        )
    }

    pub fn SHORT_SHA<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn VERSION_NUMBER<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn RC_PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        let pre_format = pre_format!(input);

        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Rc(pre_format.parse(v)))
        )
    }

    pub fn BETA_PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        let pre_format = pre_format!(input);
        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Beta(pre_format.parse(v)))
        )
    }

    pub fn ALPHA_PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        let pre_format = pre_format!(input);

        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Alpha(pre_format.parse(v)))
        )
    }

    pub fn BUILD_PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        let pre_format = pre_format!(input);

        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Build(pre_format.parse(v)))
        )
    }

    pub fn NAMED_PRE_TAG<'a>(input: Node<'a>) -> InterpreterResult<PreTag> {
        let pre_format = pre_format!(input);

        match_nodes!(input.children();
            [NAME(n), VERSION_NUMBER(v)] => Ok(
                PreTag::Named(
                    n.to_string(),
                    pre_format.parse(v)
                )
            )
        )
    }

    pub fn NAME<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn CCVER_LOG_ENTRY<'a>(input: Node<'a>) -> InterpreterResult<LogEntry<'a>> {
        match_nodes!(input.children();
            [
                SCOPE(name),
                SCOPE(branch),
                COMMIT_HASHLINE(commit_hash),
                ISO8601_DATE(commit_datetime),
                DECORATIONS_LINE(decorations),
                PARENT_HASHLINE(parents),
                SUBJECT(subject),
                FOOTER_SECTION(footers),

            ] => {
                Ok(
                   LogEntry {
                        name,
                        branch,
                        commit_hash,
                        commit_datetime,
                        commit_timezone: commit_datetime.timezone(),
                        parent_hashes: parents,
                        footers,
                        decorations,
                        subject
                    }
                )
            },
            [
                SCOPE(name),
                SCOPE(branch),
                COMMIT_HASHLINE(commit_hash),
                ISO8601_DATE(commit_datetime),
                PARENT_HASHLINE(parents),
                SUBJECT(subject),
                FOOTER_SECTION(footers),

            ] => {
                Ok(
                    LogEntry {
                        name,
                        branch,
                        commit_hash,
                        commit_datetime,
                        commit_timezone: commit_datetime.timezone(),
                        parent_hashes: parents,
                        footers,
                        decorations: Arc::new([]),
                        subject
                    }
                )
            }
        )
    }

    pub fn ISO8601_DATE(input: Node) -> InterpreterResult<chrono::DateTime<chrono::Utc>> {
        Ok(chrono::DateTime::parse_from_rfc3339(input.as_str())
            .unwrap()
            .to_utc())
    }

    pub fn EOI(input: Node) -> InterpreterResult<()> {
        Ok(())
    }

    pub fn TAG<'a>(input: Node<'a>) -> InterpreterResult<(&'a str, Option<&'a str>, bool)> {
        match_nodes!(input.into_children();
            [TYPE(t), BREAKING_BANG(b)] => Ok((t, None, b)),
            [TYPE(t), SCOPE_SECTION(s), BREAKING_BANG(b)] => Ok((t, Some(s), b)),
        )
    }

    pub fn SCOPE_SECTION<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        match_nodes!(input.into_children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn TYPE<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn SCOPE<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn BREAKING_BANG(input: Node) -> InterpreterResult<bool> {
        Ok(input.as_str() == "!")
    }

    pub fn DESCRIPTION<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn BODY<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str().trim())
    }

    pub fn BODY_SECTION<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        match_nodes!(input.children();
            [BODY(b)] => Ok(b)
        )
    }

    pub fn FOOTER<'a>(input: Node<'a>) -> InterpreterResult<(&'a str, &'a str)> {
        match_nodes!(input.children();
            [SCOPE(k),DESCRIPTION(v)] => Ok((k, v))
        )
    }

    pub fn FOOTER_SECTION<'a>(input: Node<'a>) -> InterpreterResult<HashMap<&'a str, &'a str>> {
        if input.children().count() == 0 {
            return Ok(HashMap::new());
        } else {
            Ok(input
                .children()
                .map(Parser::FOOTER)
                .map(|f| f.unwrap())
                .collect())
        }
    }

    pub fn COMMIT_HASHLINE<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        match_nodes!(input.children();
            [SHA(s)] => Ok(s)
        )
    }

    pub fn SHA<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn PARENT_HASHLINE<'a>(input: Node<'a>) -> InterpreterResult<Arc<[&'a str]>> {
        match_nodes!(input.children();
            [SHA(s)..] => Ok(s.collect())
        )
    }

    pub fn CONVENTIONAL_SUBJECT<'a>(input: Node<'a>) -> InterpreterResult<ConventionalSubject<'a>> {
        match_nodes!(input.children();
            [TAG((commit_type, scope, breaking)), DESCRIPTION(description)] => Ok(
                ConventionalSubject {
                    breaking,
                    commit_type,
                    description,
                    scope,
                }
            )
        )
    }

    pub fn SUBJECT<'a>(input: Node<'a>) -> InterpreterResult<Subject<'a>> {
        match_nodes!(input.children();
            [CONVENTIONAL_SUBJECT(s)] => Ok(Subject::Conventional(s)),
            [DESCRIPTION(t)] => Ok(Subject::Text(t)),
        )
    }

    pub fn HEAD_DEC<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn TAG_DEC<'a>(input: Node<'a>) -> InterpreterResult<Tag<'a>> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(Tag::Text(s)),
            [CCVER_VERSION(v)] => Ok(Tag::Version(v))
        )
    }

    pub fn BRANCH_DEC<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn FNAME<'a>(input: Node<'a>) -> InterpreterResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn REMOTE_DEC<'a>(input: Node<'a>) -> InterpreterResult<(&'a str, &'a str)> {
        match_nodes!(input.children();
            [FNAME(o), SCOPE(s)] => Ok((o,s))
        )
    }

    pub fn DECORATION<'a>(input: Node<'a>) -> InterpreterResult<Decoration<'a>> {
        match_nodes!(input.children();
            [HEAD_DEC(h)] =>Ok(Decoration::HeadIndicator(h)),
            [TAG_DEC(t)] => Ok(Decoration::Tag(t)),
            [REMOTE_DEC((o,s))] => Ok(Decoration::RemoteBranch((o,s))),
            [BRANCH_DEC(b)] => Ok(Decoration::Branch(b))
        )
    }

    pub fn DECORATIONS_LINE<'a>(input: Node<'a>) -> InterpreterResult<Arc<[Decoration<'a>]>> {
        if input.children().count() == 0 {
            return Ok(Arc::new([]));
        } else {
            input.children().map(Parser::DECORATION).collect()
        }
    }

    pub fn CCVER_LOG<'a>(input: Node<'a>) -> InterpreterResult<Logs<'a>> {
        match_nodes!(input.children();
            [CCVER_LOG_ENTRY(e).., EOI(_)] => Ok(e.collect())
        )
    }

    pub fn CCVER_VERSION_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<VersionFormat> {
        let version_format = match_nodes!(input.children();
            [V_PREFIX(v_prefix), VERSION_NUMBER_FORMAT(major), VERSION_NUMBER_FORMAT(minor), VERSION_NUMBER_FORMAT(patch)] => {
                VersionFormat {
                    v_prefix,
                    major,
                    minor,
                    patch,
                    prerelease: None
                }
            },
            [V_PREFIX(v_prefix), VERSION_NUMBER_FORMAT(major), VERSION_NUMBER_FORMAT(minor), VERSION_NUMBER_FORMAT(patch), PRE_TAG_FORMAT(pretag)] => {
                VersionFormat {
                    v_prefix,
                    major,
                    minor,
                    patch,
                    prerelease: Some(pretag)
                }
            },
        );

        let first_calver = if let VersionNumberFormat::CalVer(m) = &version_format.major {
            m.first().cloned()
        } else if let VersionNumberFormat::CalVer(m) = &version_format.minor {
            m.first().cloned()
        } else if let VersionNumberFormat::CalVer(m) = &version_format.patch {
            m.first().cloned()
        } else {
            match &version_format.prerelease {
                Some(Rc(v) | Beta(v) | Alpha(v) | Build(v) | Named(_, v)) => {
                    if let VersionNumberFormat::CalVer(m) = v {
                        m.first().cloned()
                    } else {
                        None
                    }
                }
                Some(Sha | ShortSha) | None => None,
            }
        };

        match first_calver {
            Some(Year4 | Year2) | None => {}
            _ => {
                return Err(parsing_error!(
                    input,
                    "The first CalVer format segment must be YY (Year4) or yy (Year2) to maintain semver monotonic incresing versions"
                ));
            }
        };

        Ok(version_format)
    }

    pub fn PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [SHA_PRE_TAG_FORMAT(s)] => Ok(s),
            [RC_PRE_TAG_FORMAT(r)] => Ok(r),
            [BETA_PRE_TAG_FORMAT(b)] => Ok(b),
            [ALPHA_PRE_TAG_FORMAT(a)] => Ok(a),
            [BUILD_PRE_TAG_FORMAT(b)] => Ok(b),
            [NAMED_PRE_TAG_FORMAT(n)] => Ok(n)
        )
    }

    pub fn SHA_PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [SHA_FORMAT(_)] => Ok(PreTagFormat::Sha),
            [SHORT_SHA_FORMAT(_)] => Ok(PreTagFormat::ShortSha)
        )
    }

    pub fn SHA_FORMAT<'a>(input: Node) -> InterpreterResult<()> {
        Ok(())
    }

    pub fn SHORT_SHA_FORMAT<'a>(input: Node) -> InterpreterResult<()> {
        Ok(())
    }

    pub fn RC_PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Rc(v))
        )
    }

    pub fn BETA_PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Beta(v))
        )
    }

    pub fn ALPHA_PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Alpha(v))
        )
    }

    pub fn BUILD_PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Build(v))
        )
    }

    pub fn NAMED_PRE_TAG_FORMAT<'a>(input: Node<'a>) -> InterpreterResult<PreTagFormat> {
        match_nodes!(input.children();
            [NAME(n), VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Named(n.to_string(), v))
        )
    }

    pub fn V_PREFIX(input: Node) -> InterpreterResult<bool> {
        Ok(input.as_str() == "v")
    }

    pub fn VERSION_NUMBER_FORMAT(input: Node) -> InterpreterResult<VersionNumberFormat> {
        match_nodes!(input.children();
            [SEMANTIC_VERSION_FORMAT(_)] => Ok(VersionNumberFormat::CCVer),
            [CALENDAR_VERSION_FORMAT(format)] => Ok(VersionNumberFormat::CalVer(format)),
            [SHA_VERSION_FORMAT(s)] => Ok(s),
        )
    }

    pub fn SHA_VERSION_FORMAT(input: Node) -> InterpreterResult<VersionNumberFormat> {
        match_nodes!(input.children();
            [SHA_FORMAT(_)] => Ok(VersionNumberFormat::Sha),
            [SHORT_SHA_FORMAT(_)] => Ok(VersionNumberFormat::ShortSha)
        )
    }

    pub fn CALENDAR_VERSION_FORMAT(input: Node) -> InterpreterResult<CalVerFormat> {
        let segments: CalVerFormat = match_nodes!(input.children();
            [CALENDAR_VERSION_FORMAT_SEGMENT(s)..] => s.collect()
        );
        let sorted = segments.iter().rev().collect::<Vec<_>>().is_sorted();
        if sorted && !segments.is_empty() {
            Ok(segments)
        } else {
            dbg!(&segments);
            Err(parsing_error!(
                input,
                "CalVer Format Segments are not sorted"
            ))
        }
    }

    pub fn CALENDAR_VERSION_FORMAT_SEGMENT(input: Node) -> InterpreterResult<CalVerFormatSegment> {
        match input.as_str() {
            "YY" => Ok(CalVerFormatSegment::Year4),
            "yy" => Ok(CalVerFormatSegment::Year2),
            "E" => Ok(CalVerFormatSegment::Epoch),
            "MM" => Ok(CalVerFormatSegment::Month),
            "DD" => Ok(CalVerFormatSegment::Day),
            "DDD" => Ok(CalVerFormatSegment::DayOfYear),
            "hh" => Ok(CalVerFormatSegment::Hour),
            "mm" => Ok(CalVerFormatSegment::Minute),
            "ss" => Ok(CalVerFormatSegment::Second),
            _ => Err(parsing_error!(input, "Invalid CalVer Format Segment")),
        }
    }

    pub fn SEMANTIC_VERSION_FORMAT(input: Node) -> InterpreterResult<()> {
        Ok(())
    }
}
