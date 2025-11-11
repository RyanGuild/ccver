#![cfg_attr(all(doc, feature = "documentation"), doc = simple_mermaid::mermaid!("../version-example.mmd"))]
#![doc = include_str!("../README.md")]
#![cfg_attr(all(doc, feature = "documentation"), doc = simple_mermaid::mermaid!("../git-log-flow.mmd"))]

pub mod args;
pub mod changelog;
pub mod git;
pub mod graph;
pub mod logs;
pub mod parser;
pub mod pattern_macros;
pub mod version;
pub mod version_format;

#[cfg(all(doc, feature = "documentation"))]
use embed_doc_image::embed_doc_image;
/// Performance profile visualization for the library.
///
/// ![E2E Performance Profile][e2e_perf]
#[cfg_attr(
    all(doc, feature = "documentation"),
    embed_doc_image("e2e_perf", "target/profiling/e2e.svg")
)]
#[cfg(all(doc, feature = "documentation"))]
pub struct LibraryPerfProfile;
