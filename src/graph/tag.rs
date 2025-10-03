use std::collections::HashMap;

use crate::{
    graph::GraphOps,
    logs::{Decoration, LogEntry, Tag},
    version::Version,
};
use petgraph::{
    Direction, Graph,
    graph::{EdgeIndex, NodeIndex},
};

pub trait HasVersionTags {
    fn version_tags(&self) -> Vec<Version>;
}

impl HasVersionTags for LogEntry<'_> {
    fn version_tags(&self) -> Vec<Version> {
        self.decorations
            .iter()
            .filter_map(|d| match d {
                Decoration::Tag(Tag::Version(version)) => Some(version.clone()),
                _ => None,
            })
            .collect()
    }
}

impl HasTextTags for LogEntry<'_> {
    fn text_tags(&self) -> Vec<String> {
        self.decorations
            .iter()
            .filter_map(|d| match d {
                Decoration::Tag(Tag::Text(text)) => Some(text.to_string()),
                _ => None,
            })
            .collect()
    }
}

pub trait HasTextTags {
    fn text_tags(&self) -> Vec<String>;
}

pub struct TagMemo<T, Ix> {
    inner: T,
    version_tag_map: HashMap<Version, NodeIndex<Ix>>,
    text_tag_map: HashMap<String, NodeIndex<Ix>>,
}

impl<N, E, Ty, Ix, T> GraphOps<N, E, Ty, Ix> for TagMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasVersionTags + HasTextTags + Clone,
    Ix: Copy,
{
    fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let clone = weight.clone();
        let idx = self.inner.add_node(weight);

        clone.version_tags().iter().for_each(|v| {
            self.version_tag_map.insert(v.clone(), idx);
        });
        clone.text_tags().iter().for_each(|t| {
            self.text_tag_map.insert(t.clone(), idx);
        });
        idx
    }

    fn node_weight(&self, idx: NodeIndex<Ix>) -> Option<&N> {
        self.inner.node_weight(idx)
    }

    fn node_weight_mut(&mut self, idx: NodeIndex<Ix>) -> Option<&mut N> {
        self.inner.node_weight_mut(idx)
    }

    fn edge_weight(&self, idx: EdgeIndex<Ix>) -> Option<&E> {
        self.inner.edge_weight(idx)
    }

    fn edge_weight_mut(&mut self, idx: EdgeIndex<Ix>) -> Option<&mut E> {
        self.inner.edge_weight_mut(idx)
    }

    fn add_edge(
        &mut self,
        from: NodeIndex<Ix>,
        to: NodeIndex<Ix>,
        weight: E,
    ) -> Option<EdgeIndex<Ix>> {
        self.inner.add_edge(from, to, weight)
    }

    fn contains_edge(&self, from: NodeIndex<Ix>, to: NodeIndex<Ix>) -> bool {
        self.inner.contains_edge(from, to)
    }

    fn node_identifiers(&self) -> Vec<NodeIndex<Ix>> {
        self.inner.node_identifiers()
    }

    fn node_references(&self) -> Vec<(NodeIndex<Ix>, &N)> {
        self.inner.node_references()
    }

    fn edge_identifiers(&self) -> Vec<EdgeIndex<Ix>> {
        self.inner.edge_identifiers()
    }

    fn edge_references(&self) -> Vec<(EdgeIndex<Ix>, &E)> {
        self.inner.edge_references()
    }

    fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    fn node_bound(&self) -> usize {
        self.inner.node_bound()
    }

    fn edge_bound(&self) -> usize {
        self.inner.edge_bound()
    }

    fn neighbors(&self, idx: NodeIndex<Ix>) -> Vec<NodeIndex<Ix>> {
        self.inner.neighbors(idx)
    }

    fn neighbors_directed(&self, idx: NodeIndex<Ix>, dir: Direction) -> Vec<NodeIndex<Ix>> {
        self.inner.neighbors_directed(idx, dir)
    }

    fn base_graph(&self) -> &Graph<N, E, Ty, Ix> {
        self.inner.base_graph()
    }

    fn base_graph_mut(&mut self) -> &mut Graph<N, E, Ty, Ix> {
        self.inner.base_graph_mut()
    }
}

pub trait TagExt<N, E, Ty, Ix> {
    fn version_tag(&self, tag: &Version) -> Option<&N>;
    fn text_tag(&self, tag: &str) -> Option<&N>;
    fn text_tag_idx(&self, tag: &str) -> Option<NodeIndex<Ix>>;
    fn version_tag_idx(&self, tag: &Version) -> Option<NodeIndex<Ix>>;
}

impl<N, E, Ty, Ix, T> TagExt<N, E, Ty, Ix> for TagMemo<T, Ix>
where
    T: GraphOps<N, E, Ty, Ix>,
    N: HasVersionTags + HasTextTags,
    Ix: Copy,
{
    fn version_tag(&self, tag: &Version) -> Option<&N> {
        self.inner.node_weight(*self.version_tag_map.get(tag)?)
    }
    fn text_tag(&self, tag: &str) -> Option<&N> {
        self.inner.node_weight(*self.text_tag_map.get(tag)?)
    }
    fn text_tag_idx(&self, tag: &str) -> Option<NodeIndex<Ix>> {
        self.text_tag_map.get(tag).cloned()
    }
    fn version_tag_idx(&self, tag: &Version) -> Option<NodeIndex<Ix>> {
        self.version_tag_map.get(tag).cloned()
    }
}
