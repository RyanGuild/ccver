use std::collections::HashMap;
use std::rc::Rc;

use pest_consume::{Node as PestNode, *};

use crate::logs::{ConventionalSubject, Decoration, LogEntry, LogEntryData, Subject, Tag};
use crate::version::PreTag;
use crate::version_format::CalVerFormat;
use crate::version_format::{
    CalVerFormatSegment::{self, *},
    PreTagFormat::{self, *},
    VersionNumberFormat,
};

use super::grammar::{Parser, Rule};
use super::macros::parsing_error;
use super::{Log, Version, VersionFormat};

pub type Node<'input, 'format> = PestNode<'input, Rule, Option<LexerInput<'format>>>;

pub type LexerResult<T> = eyre::Result<T, pest_consume::Error<Rule>>;

#[derive(Debug, Clone)]
pub struct LexerInput<'a> {
    pub version_format: Rc<VersionFormat<'a>>,
}

impl Default for LexerInput<'_> {
    fn default() -> Self {
        LexerInput {
            version_format: Rc::new(VersionFormat::default()),
        }
    }
}

fn pre_tag_version_number_format<'a>(input: &'a Node) -> Option<VersionNumberFormat> {
    let format = get_version_format(&input);
    match &format.prerelease {
        // Returns the default version
        Some(PreTagFormat::Sha | PreTagFormat::ShortSha) => None,
        Some(
            PreTagFormat::Rc(v)
            | PreTagFormat::Beta(v)
            | PreTagFormat::Alpha(v)
            | PreTagFormat::Build(v)
            | PreTagFormat::Named(_, v),
        ) => Some(v.clone()),
        None => Some(VersionNumberFormat::default()),
    }
}

fn get_version_format<'a>(input: &'a Node) -> Rc<VersionFormat<'a>> {
    input
        .user_data()
        .as_ref()
        .map(|d| d.version_format.clone())
        .unwrap_or(Rc::new(VersionFormat::default()))
}

#[pest_consume::parser]
impl Parser {
    pub fn CCVER_VERSION<'input>(input: Node<'input, '_>) -> LexerResult<Version<'input>> {
        let format = get_version_format(&input);
        match_nodes!(input.children();
            [V_PREFIX(_), VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch)] => Ok(Version {
                major: format.major.parse(major),
                minor: format.minor.parse(minor),
                patch: format.patch.parse(patch),
                prerelease: None
            }),
            [V_PREFIX(_), VERSION_NUMBER(major), VERSION_NUMBER(minor), VERSION_NUMBER(patch), PRE_TAG(pretag)] => Ok(Version {
                major: format.major.parse(major),
                minor: format.minor.parse(minor),
                patch: format.patch.parse(patch),
                prerelease: Some(pretag)
            })
        )
    }

    pub fn PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        match_nodes!(input.children();
            [SHA_PRE_TAG(s)] => Ok(s),
            [RC_PRE_TAG(r)] => Ok(r),
            [BETA_PRE_TAG(b)] => Ok(b),
            [ALPHA_PRE_TAG(a)] => Ok(a),
            [BUILD_PRE_TAG(b)] => Ok(b),
            [NAMED_PRE_TAG(n)] => Ok(n)
        )
    }

    pub fn SHA_PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        match_nodes!(input.children();
            [SHA(s)] => Ok(PreTag::Sha(s)),
            [SHORT_SHA(s)] => Ok(PreTag::ShortSha(s))
        )
    }

    pub fn SHORT_SHA<'input>(input: Node<'input, '_>) -> LexerResult<&'input str> {
        Ok(input.as_str())
    }

    pub fn VERSION_NUMBER<'input>(input: Node<'input, '_>) -> LexerResult<&'input str> {
        Ok(input.as_str())
    }

    pub fn RC_PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        let pre_format = pre_tag_version_number_format(&input).ok_or(parsing_error!(
            input,
            "PreTag must be defined in the version format"
        ))?;

        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Rc(pre_format.parse(v)))
        )
    }

    pub fn BETA_PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        let pre_format = pre_tag_version_number_format(&input).ok_or(parsing_error!(
            input,
            "PreTag must be defined in the version format"
        ))?;
        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Beta(pre_format.parse(v)))
        )
    }

    pub fn ALPHA_PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        let pre_format = pre_tag_version_number_format(&input).ok_or(parsing_error!(
            input,
            "PreTag must be defined in the version format"
        ))?;

        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Alpha(pre_format.parse(v)))
        )
    }

    pub fn BUILD_PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        let pre_format = pre_tag_version_number_format(&input).ok_or(parsing_error!(
            input,
            "PreTag must be defined in the version format"
        ))?;

        match_nodes!(input.children();
            [VERSION_NUMBER(v)] => Ok(PreTag::Build(pre_format.parse(v)))
        )
    }

    pub fn NAMED_PRE_TAG<'input>(input: Node<'input, '_>) -> LexerResult<PreTag<'input>> {
        let pre_format = pre_tag_version_number_format(&input).ok_or(parsing_error!(
            input,
            "PreTag must be defined in the version format"
        ))?;

        match_nodes!(input.children();
            [NAME(n), VERSION_NUMBER(v)] => Ok(
                PreTag::Named(
                    n,
                    pre_format.parse(v)
                )
            )
        )
    }

    pub fn NAME<'input>(input: Node<'input, '_>) -> LexerResult<&'input str> {
        Ok(input.as_str())
    }

    pub fn CCVER_LOG_ENTRY<'input>(input: Node<'input, '_>) -> LexerResult<LogEntry<'input>> {
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
                   Rc::new( LogEntryData {
                        name,
                        branch,
                        commit_hash,
                        commit_datetime,
                        commit_timezone: commit_datetime.timezone(),
                        parent_hashes: parents,
                        footers,
                        decorations,
                        subject
                    })
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
                    Rc::new(LogEntryData {
                        name,
                        branch,
                        commit_hash,
                        commit_datetime,
                        commit_timezone: commit_datetime.timezone(),
                        parent_hashes: parents,
                        footers,
                        decorations: Rc::new([]),
                        subject
                    })
                )
            }
        )
    }

    pub fn ISO8601_DATE(input: Node) -> LexerResult<chrono::DateTime<chrono::Utc>> {
        Ok(chrono::DateTime::parse_from_rfc3339(input.as_str())
            .unwrap()
            .to_utc())
    }

    pub fn EOI(input: Node) -> LexerResult<()> {
        Ok(())
    }

    pub fn TAG<'a>(input: Node<'a, '_>) -> LexerResult<(&'a str, Option<&'a str>, bool)> {
        match_nodes!(input.into_children();
            [TYPE(t), BREAKING_BANG(b)] => Ok((t, None, b)),
            [TYPE(t), SCOPE_SECTION(s), BREAKING_BANG(b)] => Ok((t, Some(s), b)),
        )
    }

    pub fn SCOPE_SECTION<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        match_nodes!(input.into_children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn TYPE<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn SCOPE<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn BREAKING_BANG(input: Node) -> LexerResult<bool> {
        Ok(input.as_str() == "!")
    }

    pub fn DESCRIPTION<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn BODY<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        Ok(input.as_str().trim())
    }

    pub fn BODY_SECTION<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        match_nodes!(input.children();
            [BODY(b)] => Ok(b)
        )
    }

    pub fn FOOTER<'a>(input: Node<'a, '_>) -> LexerResult<(&'a str, &'a str)> {
        match_nodes!(input.children();
            [SCOPE(k),DESCRIPTION(v)] => Ok((k, v))
        )
    }

    pub fn FOOTER_SECTION<'a>(input: Node<'a, '_>) -> LexerResult<HashMap<&'a str, &'a str>> {
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

    pub fn COMMIT_HASHLINE<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        match_nodes!(input.children();
            [SHA(s)] => Ok(s)
        )
    }

    pub fn SHA<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn PARENT_HASHLINE<'a>(input: Node<'a, '_>) -> LexerResult<Rc<[&'a str]>> {
        match_nodes!(input.children();
            [SHA(s)..] => Ok(s.collect())
        )
    }

    pub fn CONVENTIONAL_SUBJECT<'a>(input: Node<'a, '_>) -> LexerResult<ConventionalSubject<'a>> {
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

    pub fn SUBJECT<'a>(input: Node<'a, '_>) -> LexerResult<Subject<'a>> {
        match_nodes!(input.children();
            [CONVENTIONAL_SUBJECT(s)] => Ok(Subject::Conventional(s)),
            [DESCRIPTION(t)] => Ok(Subject::Text(t)),
        )
    }

    pub fn HEAD_DEC<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn TAG_DEC<'a>(input: Node<'a, '_>) -> LexerResult<Tag<'a>> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(Tag::Text(s)),
            [CCVER_VERSION(v)] => Ok(Tag::Version(v))
        )
    }

    pub fn BRANCH_DEC<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        match_nodes!(input.children();
            [SCOPE(s)] => Ok(s)
        )
    }

    pub fn FNAME<'a>(input: Node<'a, '_>) -> LexerResult<&'a str> {
        Ok(input.as_str())
    }

    pub fn REMOTE_DEC<'a>(input: Node<'a, '_>) -> LexerResult<(&'a str, &'a str)> {
        match_nodes!(input.children();
            [FNAME(o), SCOPE(s)] => Ok((o,s))
        )
    }

    pub fn DECORATION<'a>(input: Node<'a, '_>) -> LexerResult<Decoration<'a>> {
        match_nodes!(input.children();
            [HEAD_DEC(h)] =>Ok(Decoration::HeadIndicator(h)),
            [TAG_DEC(t)] => Ok(Decoration::Tag(t)),
            [REMOTE_DEC((o,s))] => Ok(Decoration::RemoteBranch((o,s))),
            [BRANCH_DEC(b)] => Ok(Decoration::Branch(b))
        )
    }

    pub fn DECORATIONS_LINE<'a>(input: Node<'a, '_>) -> LexerResult<Rc<[Decoration<'a>]>> {
        if input.children().count() == 0 {
            return Ok(Rc::new([]));
        } else {
            input.children().map(Parser::DECORATION).collect()
        }
    }

    pub fn CCVER_LOG<'a>(input: Node<'a, '_>) -> LexerResult<Log<'a>> {
        match_nodes!(input.children();
            [CCVER_LOG_ENTRY(e).., EOI(_)] => Ok(e.collect())
        )
    }

    pub fn CCVER_VERSION_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<VersionFormat<'a>> {
        match_nodes!(input.children();
            [V_PREFIX(v_prefix), VERSION_NUMBER_FORMAT(major), VERSION_NUMBER_FORMAT(minor), VERSION_NUMBER_FORMAT(patch)] => {

                let first_calver = if let VersionNumberFormat::CalVer(m) = &major {
                    m.first()
                } else if let VersionNumberFormat::CalVer(m) = &minor {
                    m.first()
                } else if let VersionNumberFormat::CalVer(m) = &patch {
                    m.first()
                } else {
                    None
                };

                match first_calver {
                    Some(Year4 | Year2) => {},
                    None => {},
                    _ => {
                        return Err(parsing_error!(input, "The first CalVer format segment must be YY (Year4) or yy (Year2) to maintain semver monotonic incresing versions"))
                    }
                };

                Ok(
                    VersionFormat {
                        v_prefix,
                        major,
                        minor,
                        patch,
                        prerelease: None
                    }
                )
            },
            [V_PREFIX(v_prefix), VERSION_NUMBER_FORMAT(major), VERSION_NUMBER_FORMAT(minor), VERSION_NUMBER_FORMAT(patch), PRE_TAG_FORMAT(pretag)] => {
                let first_calver = if let VersionNumberFormat::CalVer(m) = &major {
                    m.first().cloned()
                } else if let VersionNumberFormat::CalVer(m) = &minor {
                    m.first().cloned()
                } else if let VersionNumberFormat::CalVer(m) = &patch {
                    m.first().cloned()
                } else {
                    match &pretag {
                        Rc(v)
                        | Beta(v)
                        | Alpha(v)
                        | Build(v)
                        | Named(_, v) => {
                            if let VersionNumberFormat::CalVer(m) = v {
                                m.first().cloned()
                            } else {
                                None
                            }
                        },
                        Sha => None,
                        ShortSha => None,
                    }
                };

                match first_calver {
                    Some(Year4 | Year2) => {},
                    None => {},
                    _ => {
                        return Err(parsing_error!(input, "The first CalVer format segment must be YY (Year4) or yy (Year2) to maintain semver monotonic incresing versions"))
                    }
                };

                Ok(
                    VersionFormat {
                        v_prefix,
                        major,
                        minor,
                        patch,
                        prerelease: Some(pretag)
                    }
                )
            },
        )
    }

    pub fn PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [SHA_PRE_TAG_FORMAT(s)] => Ok(s),
            [RC_PRE_TAG_FORMAT(r)] => Ok(r),
            [BETA_PRE_TAG_FORMAT(b)] => Ok(b),
            [ALPHA_PRE_TAG_FORMAT(a)] => Ok(a),
            [BUILD_PRE_TAG_FORMAT(b)] => Ok(b),
            [NAMED_PRE_TAG_FORMAT(n)] => Ok(n)
        )
    }

    pub fn SHA_PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [SHA_FORMAT(_)] => Ok(PreTagFormat::Sha),
            [SHORT_SHA_FORMAT(_)] => Ok(PreTagFormat::ShortSha)
        )
    }

    pub fn SHA_FORMAT<'a>(input: Node) -> LexerResult<()> {
        Ok(())
    }

    pub fn SHORT_SHA_FORMAT<'a>(input: Node) -> LexerResult<()> {
        Ok(())
    }

    pub fn RC_PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Rc(v))
        )
    }

    pub fn BETA_PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Beta(v))
        )
    }

    pub fn ALPHA_PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Alpha(v))
        )
    }

    pub fn BUILD_PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Build(v))
        )
    }

    pub fn NAMED_PRE_TAG_FORMAT<'a>(input: Node<'a, '_>) -> LexerResult<PreTagFormat<'a>> {
        match_nodes!(input.children();
            [NAME(n), VERSION_NUMBER_FORMAT(v)] => Ok(PreTagFormat::Named(n, v))
        )
    }

    pub fn V_PREFIX(input: Node) -> LexerResult<bool> {
        Ok(input.as_str() == "v")
    }

    pub fn VERSION_NUMBER_FORMAT(input: Node) -> LexerResult<VersionNumberFormat> {
        match_nodes!(input.children();
            [SEMANTIC_VERSION_FORMAT(_)] => Ok(VersionNumberFormat::CCVer),
            [CALENDAR_VERSION_FORMAT(format)] => Ok(VersionNumberFormat::CalVer(format))
        )
    }

    pub fn CALENDAR_VERSION_FORMAT(input: Node) -> LexerResult<CalVerFormat> {
        let segments: CalVerFormat = match_nodes!(input.children();
            [CALENDAR_VERSION_FORMAT_SEGMENT(s)..] => s.collect()
        );
        let sorted = segments.iter().rev().collect::<Vec<_>>().is_sorted();
        if sorted && segments.len() > 0 {
            Ok(segments)
        } else {
            dbg!(&segments);
            Err(parsing_error!(
                input,
                "CalVer Format Segments are not sorted"
            ))
        }
    }

    pub fn CALENDAR_VERSION_FORMAT_SEGMENT(input: Node) -> LexerResult<CalVerFormatSegment> {
        match input.as_str() {
            "YYYY" => Ok(CalVerFormatSegment::Year4),
            "YY" => Ok(CalVerFormatSegment::Year2),
            "E" => Ok(CalVerFormatSegment::Epoch),
            "MM" => Ok(CalVerFormatSegment::Month),
            "DD" => Ok(CalVerFormatSegment::Day),
            "DDD" => Ok(CalVerFormatSegment::DayOfYear),
            "HH" => Ok(CalVerFormatSegment::Hour),
            "mm" => Ok(CalVerFormatSegment::Minute),
            "ss" => Ok(CalVerFormatSegment::Second),
            _ => Err(parsing_error!(input, "Invalid CalVer Format Segment")),
        }
    }

    pub fn SEMANTIC_VERSION_FORMAT(input: Node) -> LexerResult<()> {
        Ok(())
    }
}
