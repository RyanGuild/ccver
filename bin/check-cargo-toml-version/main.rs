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

    let logs = Logs::from_path(&std::env::current_dir().unwrap()).unwrap();
    let graph = MemoizedCommitGraph::new(logs, &ccver_format);
    let target_version = graph.head().unwrap().as_existing_version().unwrap();

    assert!(
        existing_version == target_version,
        "Existing version does not match target version"
    );
}
