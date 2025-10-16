use crate::git::is_dirty;
use crate::{args::CCVerArgs, graph::MemoizedCommitGraph, version_format::VersionFormat};
use eyre::{OptionExt as _, Result, eyre};
use tracing::{debug, info};

use crate::version::Version;

pub fn current_version_handler(
    graph: &MemoizedCommitGraph,
    args: &CCVerArgs,
    version_format: &VersionFormat,
) -> Result<Version> {
    debug!("Using default command to get current version");
    let v = match is_dirty(&args.path) {
        Result::Ok(dirty) => {
            if args.ci && dirty {
                Err(eyre!("Repo is dirty while ci is true"))
            } else if dirty {
                let head = graph.head();
                debug!("Head: {:#?}", head);
                let head = head.unwrap().lock().unwrap();
                let version = head
                    .version
                    .clone()
                    .ok_or_eyre(eyre!("Current Branch Head Was Not Assigned a Version"));
                version.map(|v| v.build(&head.log_entry, version_format))
            } else {
                let head = graph.head().ok_or_eyre("No Head Found")?;
                let head = head.lock().unwrap();
                let version = head
                    .version
                    .clone()
                    .ok_or_eyre("Current Branch Head Was Not Assigned a Version")?;
                Ok(version)
            }
        }
        Err(e) => Err(e),
    }?;

    info!("Version: {:?}", v);

    let result = if args.no_pre {
        let head_guard = graph.head().unwrap().lock().unwrap();
        v.release(&head_guard.log_entry, version_format)
    } else {
        v
    };

    Ok(result)
}
