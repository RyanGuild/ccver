use std::collections::HashMap;

use pest::Parser as _;
use pest_derive::Parser as PP;

#[derive(PP)]
#[grammar = "parser/cc/rules.pest"]
pub struct Parser;

pub struct ConventionalCommit {
    commit_type: String,
    breaking: bool,
    scope: Option<String>,
    description: String,
    body: Option<String>,
    footer: HashMap<String, String>,
}

pub fn parse_commit_message(commit_message: &str) -> ConventionalCommit {
    let commit = Parser::parse(Rule::COMMIT, commit_message).unwrap();

    let commit_type = commit
        .find_first_tagged("type")
        .unwrap()
        .as_str()
        .to_string();
    let breaking = commit.find_first_tagged("breaking").is_some();
    let scope = commit
        .find_first_tagged("scope")
        .map(|s| s.as_str().to_string());
    let description = commit
        .find_first_tagged("description")
        .unwrap()
        .as_str()
        .to_string();
    let body = commit
        .find_first_tagged("body")
        .map(|b| b.as_str().to_string());
    let footer: HashMap<String, String> = commit
        .find_tagged("footer")
        .map(|footer_pair| {
            let footer_inner = footer_pair.into_inner();
            (
                footer_inner
                    .find_first_tagged("footer_key")
                    .unwrap()
                    .as_str()
                    .to_string(),
                footer_inner
                    .find_first_tagged("footer_value")
                    .unwrap()
                    .as_str()
                    .to_string(),
            )
        })
        .collect();

    return ConventionalCommit {
        commit_type,
        breaking,
        scope,
        description,
        body,
        footer,
    };
}

#[cfg(test)]
mod parser_tests {
    use indoc::indoc;
    use pest::Parser as _;

    use super::*;
    use crate::parser::flat_tag_map::flat_tag_map;

    #[test]
    fn components() {
        Parser::parse(Rule::TYPE, "feat").unwrap();
        let full = Parser::parse(Rule::TAG, "feat(src)!:").unwrap();
        Parser::parse(Rule::TAG, "feat:").unwrap();
        Parser::parse(Rule::COMMIT, "feat: add new feature").unwrap();
        println!("{:#?}", full);
    }

    #[test]
    fn tag_tests() {
        let input = Parser::parse(Rule::COMMIT, "feat: add new feature").unwrap();
        let tags = flat_tag_map(input);
        println!("{:#?}", tags);
        tags.get("type").expect("type not found");
    }

    #[test]
    fn spaceing_test() {
        let example = Parser::parse(
            Rule::COMMIT,
            indoc! {
            "feat(src): add new feature

            Body Content
            :}

            Footer-Name: Footer Content
            Data: test2
            "},
        )
        .unwrap();

        let example2 = Parser::parse(
            Rule::COMMIT,
            indoc! {
            "feat(src): add new feature


            Body Content
            :}






            Footer-Name: Footer Content
            Data: test2
            "},
        )
        .unwrap();
    }
}
