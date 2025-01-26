type 

fn reduce_tagged_pairs(input: Pairs<Rule>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut next = vec![input];

    loop {
        if let Some(pairs) = next.pop() {
            for pair in pairs {
                if let Some(tag) = pair.as_node_tag() {
                    map.insert(tag.to_string(), pair.as_str().to_string());
                } else {
                    next.push(pair.into_inner());
                };
            }
        } else {
            return map;
        };
    }
}