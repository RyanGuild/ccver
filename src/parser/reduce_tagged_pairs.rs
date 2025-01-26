use std::collections::HashMap;

use pest::{iterators::Pairs, RuleType};

type PestDynamicTaggedData = HashMap<String, PestDynamicTaggedDataPropery>;

#[derive(Debug)]
enum PestDynamicTaggedDataPropery {
    Value(String),
    Array(Vec<PestDynamicTaggedDataPropery>),
    Map(Box<PestDynamicTaggedData>)
}

fn reduce_pairs<T: RuleType>(key: String, pairs: Pairs<T>) -> PestDynamicTaggedDataPropery {
    let mut map: PestDynamicTaggedData = HashMap::new();
    for pair in pairs {
        if let Some(tag_str) = pair.as_node_tag() {
            let tag = tag_str.to_string();
            let val = pair.as_str().to_string();
            let inner = pair.into_inner();

            if inner.len() == 0 {
                if map.contains_key(&tag) {
                    let existing = map.get_mut(&tag).unwrap();
                    match existing {
                        PestDynamicTaggedDataPropery::Array(arr) => {
                            arr.push(PestDynamicTaggedDataPropery::Value(val));
                        }
                        PestDynamicTaggedDataPropery::Value(existing_val) => {
                            let arr = vec![
                                PestDynamicTaggedDataPropery::Value(existing_val.clone()),
                                PestDynamicTaggedDataPropery::Value(val),
                            ];
                            map.insert(tag, PestDynamicTaggedDataPropery::Array(arr));
                        }
                        _ => {
                            panic!("unexpected type");
                        }
                    }
                } else {
                    map.insert(tag, PestDynamicTaggedDataPropery::Value(val));
                }
            } else {
                map.insert(tag.clone(), reduce_pairs(tag, inner));
            }
        }

    }

    return PestDynamicTaggedDataPropery::Map(Box::new(map));
}


#[cfg(test)]
mod reduce_pairs_tests {
    use super::*;
    use pest::Parser as _;

    #[test]
    fn test_reduce_pairs() {
        let input = crate::parser::git::Parser::parse(
            crate::parser::git::Rule::LOG_ENTRY,
            include_str!("./git/entry2.txt"),
        )
        .unwrap();
        let tags = reduce_pairs("root".to_string(), input);
        println!("{:#?}", tags);
    }
        
}