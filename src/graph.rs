use eyre::{eyre, OptionExt, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Bfs, DfsPostOrder, Reversed, Walker};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::parser::Decoration;
use crate::parser::{CCVerLog, CCVerLogEntry, Decoration::HeadIndicator};

pub type CCVerPetGraph<'a> = DiGraph<CCVerLogEntry<'a>, ()>;

#[derive(Debug)]
pub struct CCVerCommitGraphData<'a> {
    petgraph: CCVerPetGraph<'a>,
    head_index: NodeIndex,
    tail_index: NodeIndex,
    commit_to_index: HashMap<String, NodeIndex>,
}

pub type CCVerCommitGraph<'a> = Rc<RefCell<CCVerCommitGraphData<'a>>>;
pub enum Directions {
    Backward,
    Forward,
}

#[derive(Debug)]
pub enum Locations<'a> {
    Head,
    Initial,
    Sha(&'a str),
    Decoration(Decoration<'a>),
}

impl Locations<'_> {
    fn to_idx(&self, graph: &CCVerCommitGraphData) -> Result<NodeIndex> {
        match self {
            Locations::Head => Ok(graph.headidx()),
            Locations::Initial => Ok(graph.tailidx()),
            Locations::Sha(sha) => graph.commitidx(&sha),
            Locations::Decoration(d) => match d {
                Decoration::HeadIndicator(_) => Ok(graph.headidx()),
                Decoration::Tag(t) => graph.tagidx(t),
                Decoration::RemoteBranch((o, b)) => graph.remoteidx(o, b),
                Decoration::Branch(b) => graph.branchidx(b),
            },
        }
    }
}

impl CCVerCommitGraphData<'_> {
    pub fn new<'a>(logs: CCVerLog<'a>) -> Result<CCVerCommitGraph<'a>> {
        let mut petgraph = DiGraph::new();
        let commit_to_index: HashMap<String, NodeIndex> = logs
            .into_iter()
            .map(|l| (l.commit_hash.to_string(), petgraph.add_node(l.clone())))
            .collect();

        let indexes = commit_to_index.values();

        let edges: Vec<(NodeIndex, NodeIndex)> = indexes
            .flat_map(|i| {
                let ref1 = petgraph[*i].clone();
                let parents = ref1.parent_hashes.clone();
                let parents_iter = parents.into_iter();
                let parent_index_iter = parents_iter.map(|p| commit_to_index[*p]);
                let current_index_iter = std::iter::repeat_n((*i).clone(), parent_index_iter.len());
                current_index_iter
                    .zip(parent_index_iter)
                    .collect::<Vec<(NodeIndex, NodeIndex)>>()
            })
            .collect();

        for (child, parent) in edges {
            petgraph.add_edge(child.clone(), parent.clone(), ());
        }

        let head_index = petgraph
            .node_indices()
            .filter(|i| {
                let node = petgraph[*i].clone();
                for dec in node.decorations.iter() {
                    match dec {
                        HeadIndicator(_) => return true,
                        _ => continue,
                    };
                }
                return false;
            })
            .next()
            .ok_or_eyre("could not find HEAD in graph")?
            .clone();

        let tail_index = petgraph
            .node_indices()
            .filter(|i| {
                let node = petgraph[*i].clone();
                if node.parent_hashes.len() == 0 {
                    true
                } else {
                    false
                }
            })
            .next()
            .ok_or_eyre("could not find initial commit circular history or no commits detected")?
            .clone();

        Ok(Rc::new(RefCell::new(CCVerCommitGraphData {
            petgraph,
            head_index,
            tail_index,
            commit_to_index,
        })))
    }

    pub fn commit(&self, index: &str) -> Result<CCVerLogEntry<'_>> {
        Ok(self.petgraph[self.commitidx(index)?].clone())
    }

    fn commitidx(&self, index: &str) -> Result<NodeIndex> {
        self.commit_to_index
            .get(index)
            .map(|n| *n)
            .ok_or_eyre("could not find commit in commit map")
    }

    pub fn head(&self) -> CCVerLogEntry {
        self.petgraph[self.head_index].clone()
    }

    fn headidx(&self) -> NodeIndex {
        self.head_index
    }

    pub fn tail(&self) -> CCVerLogEntry {
        self.petgraph[self.tail_index].clone()
    }

    fn tailidx(&self) -> NodeIndex {
        self.tail_index
    }

    pub fn tag(&self, tag: &str) -> Result<CCVerLogEntry> {
        Ok(self.petgraph[self.tagidx(tag)?].clone())
    }

    fn tagidx(&self, tag: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx].decorations.iter() {
                match dec {
                    Decoration::Tag(t) => {
                        if *t == tag {
                            return Ok(idx);
                        } else {
                            continue;
                        };
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }
        Err(eyre!("tag not found in history"))
    }

    pub fn branch(&self, branch: &str) -> Result<CCVerLogEntry> {
        Ok(self.petgraph[self.branchidx(branch)?].clone())
    }

    fn branchidx(&self, branch: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx].decorations.iter() {
                match dec {
                    Decoration::Branch(b) => {
                        if *b == branch {
                            return Ok(idx);
                        } else {
                            continue;
                        };
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }
        Err(eyre!("branch not found in history"))
    }

    pub fn remote(&self, remote: &str, branch: &str) -> Result<CCVerLogEntry> {
        Ok(self.petgraph[self.remoteidx(remote, branch)?].clone())
    }

    fn remoteidx(&self, remote: &str, branch: &str) -> Result<NodeIndex> {
        for idx in self.petgraph.node_indices() {
            for dec in self.petgraph[idx].decorations.iter() {
                match dec {
                    Decoration::RemoteBranch((o, b)) => {
                        if *o == remote && *b == branch {
                            return Ok(idx);
                        } else {
                            continue;
                        };
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }
        Err(eyre!("remote not found in history"))
    }

    pub fn iter_from<'a>(
        &'a self,
        location: Locations,
        direction: Directions,
    ) -> Result<Box<dyn Iterator<Item = (NodeIndex, CCVerLogEntry<'a>)> + 'a>> {
        let start = location.to_idx(&self)?;

        match direction {
            Directions::Backward => Ok(Box::new(
                Bfs::new(&self.petgraph, start)
                    .iter(&self.petgraph)
                    .map(|idx| (idx, self.petgraph[idx].clone())),
            )),

            Directions::Forward => {
                let graph = Reversed(&self.petgraph);
                Ok(Box::new(
                    Bfs::new(graph, start)
                        .iter(graph)
                        .map(|idx| (idx, self.petgraph[idx].clone())),
                ))
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeIndex, CCVerLogEntry)> {
        self.petgraph
            .node_indices()
            .map(|idx| (idx, self.petgraph[idx].clone()))
    }

    pub fn dfs_postorder_history(&self) -> impl Iterator<Item = (NodeIndex, CCVerLogEntry)> {
        let start = self.headidx();
        DfsPostOrder::new(&self.petgraph, start)
            .iter(&self.petgraph)
            .map(|idx| (idx, self.petgraph[idx].clone()))
    }

    pub fn bfs_history(&self) -> impl Iterator<Item = (NodeIndex,CCVerLogEntry)> {
        let start = self.headidx();
        let graph = &self.petgraph;
        Bfs::new(graph, start)
            .iter(graph)
            .map(|idx| (idx, self.petgraph[idx].clone()))
    }
}

#[cfg(test)]
mod graph_tests {

    use crate::logs::Logs;
    use eyre::Result;

    use super::{Directions, Locations};

    #[test]
    fn test_iter_from() -> Result<()> {
        let mut logs = Logs::default();
        let graph = logs.get_graph();
        let graph = graph.borrow();

        // Second commit
        let second_commit = "40f8bef8e7c290ebe0e52b91fa84fee30b4a162d";

        let iter1: Vec<_> = graph
            .iter_from(Locations::Sha(second_commit), Directions::Backward)?
            .collect();

        assert_eq!(iter1.len(), 2);

        let iter2: Vec<_> = graph
            .iter_from(Locations::Sha(second_commit), Directions::Forward)?
            .collect();

        assert_ne!(iter2.len(),2);


        // -1 on account of the second_commit appears in both iters
        assert_eq!(iter1.len() + iter2.len() - 1, graph.dfs_postorder_history().count());
        assert_eq!(iter1.len() + iter2.len() - 1, graph.bfs_history().count());

        Ok(())
    }

    #[test]
    fn test_graph_walk() {
        let mut logs = Logs::default();
        let graph = logs.get_graph();
        let logs: Vec<String> = graph
            .borrow()
            .dfs_postorder_history()
            .map(|(_,n)| n.commit_hash.to_string())
            .collect();
        assert_ne!(logs.len(), 0);

        let logs2: Vec<String> = graph
            .borrow()
            .bfs_history()
            .map(|(_,n)| n.commit_hash.to_string())
            .collect();
        assert_eq!(logs.len(), logs2.len());
    }
}
