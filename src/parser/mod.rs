pub mod cc;
pub mod git;
mod reduce_tagged_pairs;

mod flat_tag_map;

#[cfg(test)]
mod api_tests {
    use super::*;
    use flat_tag_map::flat_tag_map;
    use pest::Parser as _;

    #[test]
    fn tag_tests() {
        let input = git::Parser::parse(
            git::Rule::COMMIT_LOGLINE,
            "commit 1234567890abcdef1234567890abcdef12345678",
        )
        .unwrap();
        let tags = flat_tag_map(input);
        tags.get("commit_sha").expect("commit_sha not found");
        println!("{:#?}", tags);
    }
}
