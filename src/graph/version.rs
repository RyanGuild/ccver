use tracing::{debug, instrument, warn};

use crate::{
    graph::{CommitGraphNodeWeight, node::CommitGraphNodeData},
    logs::{Decoration, LogEntry, Tag},
    version::Version,
    version_format::VersionFormat,
};

pub trait TaggedVersionExt<'a> {
    fn as_tagged_version(&'a self) -> Option<&'a Version>;
}

impl<'a> TaggedVersionExt<'a> for LogEntry<'a> {
    fn as_tagged_version(&'a self) -> Option<&'a Version> {
        for decoration in self.decorations.iter() {
            if let Decoration::Tag(Tag::Version(version)) = decoration {
                return Some(version);
            }
        }
        None
    }
}

pub trait ExistingVersionExt {
    fn as_existing_version(&self) -> Option<Version>;
}

pub trait SetVersionExt {
    fn set_version(&mut self, version: Version);
}

impl<'a> SetVersionExt for CommitGraphNodeWeight<'a> {
    fn set_version(&mut self, version: Version) {
        let mut guard = self.lock().unwrap();
        guard.version = Some(version);
    }
}

impl<'a> ExistingVersionExt for LogEntry<'a> {
    fn as_existing_version(&self) -> Option<Version> {
        self.as_tagged_version().cloned()
    }
}

impl<'a> ExistingVersionExt for CommitGraphNodeData<'a> {
    #[instrument(skip(self))]
    fn as_existing_version(&self) -> Option<Version> {
        let tagged_version = self.log_entry.as_tagged_version();
        let existing_version = self.version.as_ref();
        let result = match (tagged_version, existing_version) {
            (Some(tagged), Some(existing)) => Some(tagged.max(existing).clone()),
            (Some(tagged), None) => Some(tagged.clone()),
            (None, Some(existing)) => Some(existing.clone()),
            (None, None) => None,
        };

        if let Some(ref result) = result {
            debug!(version = %result);
        } else {
            warn!(commit_hash = %self.log_entry.commit_hash, branch = %self.log_entry.branch, "No existing version found");
        }
        result
    }
}

pub trait NextVersionExt<'a> {
    fn as_next_version(&'a self, max_parent: &Version, version_format: &VersionFormat) -> Version;
}

impl<'a> NextVersionExt<'a> for &CommitGraphNodeWeight<'a> {
    fn as_next_version(&'a self, max_parent: &Version, version_format: &VersionFormat) -> Version {
        let data = self.lock().unwrap();
        let existing_version = data.log_entry.as_existing_version();
        let prev_version = existing_version.unwrap_or(max_parent.clone());
        prev_version.next_version(&data.log_entry, version_format)
    }
}

impl<'a> NextVersionExt<'a> for &CommitGraphNodeData<'a> {
    fn as_next_version(&'a self, max_parent: &Version, version_format: &VersionFormat) -> Version {
        let existing_version = self.as_existing_version();
        let prev_version = existing_version.unwrap_or(max_parent.clone());
        prev_version.next_version(&self.log_entry, version_format)
    }
}

impl<'a> NextVersionExt<'a> for LogEntry<'a> {
    fn as_next_version(&'a self, max_parent: &Version, version_format: &VersionFormat) -> Version {
        let existing_version = self.as_existing_version();
        let prev_version = existing_version.unwrap_or(max_parent.clone());
        prev_version.next_version(self, version_format)
    }
}
