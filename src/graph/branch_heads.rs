use crate::logs::{Decoration, Logs};
use std::collections::HashMap;

pub trait AsBranchHeadCommits<'a> {
    fn as_branch_heads(&'_ self) -> HashMap<&'a str, &'a str>;

    fn as_current_head(&'_ self) -> Option<&'a str>;
}

impl<'a, 'b> AsBranchHeadCommits<'a> for &'b Logs<'a>
where
    'b: 'a,
{
    fn as_current_head(&'_ self) -> Option<&'a str> {
        self.iter().find_map(|l| {
            l.decorations.iter().find_map(|d| match d {
                Decoration::HeadIndicator(b) => Some(*b),
                _ => None,
            })
        })
    }

    fn as_branch_heads(&'_ self) -> HashMap<&'a str, &'a str>
    where
        'b: 'a,
    {
        self.iter()
            .filter_map(|l| {
                l.decorations.iter().find_map(|d| match d {
                    Decoration::Branch(b) => Some((*b, l.commit_hash)),
                    Decoration::RemoteBranch((_, b)) => Some((*b, l.commit_hash)),
                    Decoration::HeadIndicator(b) => Some((*b, l.commit_hash)),
                    _ => None,
                })
            })
            .collect()
    }
}
