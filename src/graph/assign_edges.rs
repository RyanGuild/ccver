use crate::graph::{CommitGraphT, commit_map::CommitMap};

pub fn assign_edges<'a>(graph: &mut CommitGraphT<'a>, commit_to_index: &CommitMap<'a>) {
    let immutable_graph = &*graph;
    let edges = commit_to_index
        .values()
        .map(|i| (i, immutable_graph.node_weight(*i).unwrap()))
        .flat_map(|(i, ref1)| {
            let parents = ref1.lock().unwrap().log_entry.parent_hashes.clone();
            let parents_iter = parents.iter();
            let parent_index_iter = parents_iter.map(|p| commit_to_index[*p]);
            let current_index_iter = std::iter::repeat_n(i, parent_index_iter.len());
            current_index_iter
                .zip(parent_index_iter)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    for (child, parent) in edges {
        graph.add_edge(*child, parent, ());
    }
}
