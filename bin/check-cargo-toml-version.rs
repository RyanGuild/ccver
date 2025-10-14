use std::path::PathBuf;

use ccver::{
    graph::{MemoizedCommitGraph, version::ExistingVersionExt as _},
    logs::Logs,
    parser,
    version_format::{PreTagFormat, VersionFormat, VersionNumberFormat},
};

fn main() {
    let ccver_format = VersionFormat {
        v_prefix: false,
        major: VersionNumberFormat::CCVer,
        minor: VersionNumberFormat::CCVer,
        patch: VersionNumberFormat::CCVer,
        prerelease: Some(PreTagFormat::Build(VersionNumberFormat::CCVer)),
    };
    let existing_version = parser::parse_version(
        &std::env::var("CARGO_PKG_VERSION").unwrap(),
        ccver_format.clone(),
    )
    .unwrap();

    let logs =
        Logs::from_path(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))).unwrap();
    println!("Logs Count : {}", logs.len());
    let graph = MemoizedCommitGraph::new(logs, &ccver_format);
    println!("Graph Head: {:#?}", graph.head().unwrap());
    let target_version = graph.head().unwrap().as_existing_version().unwrap();

    println!("Target version: {}", target_version);
    println!("Existing version: {}", existing_version);
    println!(
        "Existing version == Target version: {}",
        existing_version == target_version
    );

    assert!(
        existing_version == target_version,
        "Existing version does not match target version"
    );
}
