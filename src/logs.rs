use std::{env::current_dir, path::PathBuf, process::Command};
use crate::parser::{parse, CCVerLog};
use crate::graph::{CCVerCommitGraph, CCVerCommitGraphData};



pub const GIT_FORMAT_ARG: &str = "--format=name=%n%f%ncommit=%n%H%ncommit-time=%n%cI%ndec=%n%d%nparent=%n%P%nsub=%n%s%ntrailers=%n%(trailers:only)%n";

#[derive(Debug)]
pub struct Logs<'a> {
    raw: &'a str,
    parsed: Option<CCVerLog<'a>>,
    graph: Option<CCVerCommitGraph<'a>>
}

impl<'a> Logs<'a> {
    pub fn new(dir: PathBuf) -> Logs<'a> {
        let output = Command::new("git")
            .args(["log", GIT_FORMAT_ARG])
            .current_dir(dir)
            .output()
            .expect("error getting command output");
        let output_str = String::from_utf8(output.stdout).expect("error parsing utf8");
        Logs {
            raw: Box::leak(output_str.into_boxed_str()),
            parsed: None,
            graph: None
        }
    }

    pub fn get_raw(&self) -> &'a str {
        self.raw
    }

    pub fn get_parsed(&mut self) -> CCVerLog<'a> {
        if let Some(parsed) = self.parsed.clone() {
            parsed
        } else {
            let parsed = parse(self.raw).expect("could not parse raw logs");
            self.parsed = Some(parsed.clone());
            parsed
        }
        
    }

    pub fn get_graph(&mut self) -> CCVerCommitGraph<'a> {
        if let Some(graph) = self.graph.clone() {
            graph.clone()
        } else {
            let log = self.get_parsed();
            let graph = CCVerCommitGraphData::new(log).expect("could not parse commits to graph");
            self.graph = Some(graph.clone());
            graph
        }
    }
}

impl Default for Logs<'_> {
    fn default() -> Self {
        let dir = current_dir().unwrap();
        Logs::new(dir)
    }
}


#[cfg(test)]
mod logs_tests {

    use super::*;


    #[test]
    fn test_default() {
        Logs::default();
    }

    #[test]
    fn test_logs_parsed() {
        let mut logs = Logs::default();
        logs.get_parsed();
    }

    #[test]
    fn test_logs_graph(){

        let mut logs = Logs::default();
        logs.get_graph();
    }
}
