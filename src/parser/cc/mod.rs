use std::{collections::HashMap};
use pest_consume::{match_nodes, Node, Parser as PP};

#[derive(PP)]
#[grammar = "parser/cc/rules.pest"]
pub struct Parser;

#[pest_consume::parser]
impl Parser {
    fn TYPE(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn SCOPE(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn BREAKING_BANG(input: Node<Rule, ()>) -> Result<bool, pest_consume::Error<Rule>> {
        Ok(true)
    }

    fn DESCRIPTION(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn BODY(input: Node<Rule, ()>) -> Result<String, pest_consume::Error<Rule>> {
        Ok(input.as_str().to_string())
    }

    fn FOOTER(input: Node<Rule, ()>) -> Result<(String, String), pest_consume::Error<Rule>> {
        let mut key = None;
        let mut value = None;
        match_nodes!(input.into_children();
            [SCOPE(k)] => key = Some(k),
            [DESCRIPTION(v)] => value = Some(v),
        );
        Ok((key.expect("footer key not found"), value.expect("footer value not found")))
    }


    fn EOI(_input: Node<Rule, ()>) -> Result<(), pest_consume::Error<Rule>> {
        Ok(())
    }


    fn COMMIT(input: Node<Rule, ()>) -> Result<ConventionalCommit, pest_consume::Error<Rule>> {
        let mut commit_type = None;
        let mut breaking = false;
        let mut scope = None;
        let mut description = None;
        let mut body = None;
        let mut footer = HashMap::new();
        

        match_nodes!(input.into_children();
            [TYPE(t)] => commit_type = Some(t),
            [BREAKING_BANG(_)] => breaking = true,
            [SCOPE(s)] => scope = Some(s),
            [DESCRIPTION(d)] => description = Some(d),
            [BODY(b)] => body = Some(b),
            [FOOTER(f)..] => footer = f.collect(),
            [EOI(_)] => (),
        );


        Ok(ConventionalCommit {
            commit_type: commit_type.expect("commit type not found"),
            breaking,
            scope,
            description: description.expect("description not found"),
            body,
            footer,
        })
    }
}

#[derive(Debug)]
pub struct ConventionalCommit {
    commit_type: String,
    breaking: bool,
    scope: Option<String>,
    description: String,
    body: Option<String>,
    footer: HashMap<String, String>,
}


#[cfg(test)]
mod parser_tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn components() -> Result<(), pest_consume::Error<Rule>> {
        Parser::parse(Rule::TYPE, "feat").unwrap();
        let full = Parser::parse(Rule::COMMIT, "feat(src)!: body").unwrap();
        Parser::parse(Rule::TAG, "feat:").unwrap();
        Parser::parse(Rule::COMMIT, "feat: add new feature").unwrap();
        println!("{:#?}", full);

        let single = full.single()?;

        let commit = Parser::COMMIT(single)?;

        println!("{:#?}", commit);




        Ok(())




        
    }

    // #[test]
    // fn tag_tests() {
    //     let input = Parser::parse(Rule::COMMIT, "feat: add new feature").unwrap();
    //     let tags = flat_tag_map(input);
    //     println!("{:#?}", tags);
    //     tags.get("type").expect("type not found");
    // }

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
