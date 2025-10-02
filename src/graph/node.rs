use std::sync::{Arc, Mutex};

use crate::logs::LogEntry;
use crate::version::Version;

pub type CommitGraphNodeWeight<'a> = Arc<Mutex<CommitGraphNodeData<'a>>>;

#[derive(Debug, Clone)]
pub struct CommitGraphNodeData<'a> {
    pub log_entry: LogEntry<'a>,
    pub version: Option<Version>,
}

impl<'a> From<LogEntry<'a>> for CommitGraphNodeData<'a> {
    fn from(log_entry: LogEntry<'a>) -> Self {
        CommitGraphNodeData {
            log_entry,
            version: None,
        }
    }
}
