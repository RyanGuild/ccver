/// Shared test fixtures for benchmarking

/// Generate a mock git log string with the specified number of commits
pub fn generate_mock_git_log(num_commits: usize) -> String {
    let mut log = String::new();

    for i in 0..num_commits {
        let commit_hash = format!("{:040x}", i);
        let parent_hash = if i > 0 {
            format!("{:040x}", i - 1)
        } else {
            String::new()
        };

        let subject = match i % 5 {
            0 => format!("feat: add feature {}", i),
            1 => format!("fix: fix bug {}", i),
            2 => format!("chore: update dependency {}", i),
            3 => format!("docs: update documentation {}", i),
            _ => format!("refactor: refactor code {}", i),
        };

        let decoration = if i == num_commits - 1 {
            " (HEAD -> main)"
        } else if i % 10 == 0 {
            " (tag: v0.1.0)"
        } else {
            ""
        };

        // Format matching git log --format
        log.push_str(&format!(
            "name=\nTest-User-{}\nbranch=\nmain\ncommit=\n{}\ncommit-time=\n2024-01-01T00:00:00Z\ndec=\n{}\nparent=\n{}\nsub=\n{}\ntrailers=\n\n",
            i, commit_hash, decoration, parent_hash, subject
        ));
    }

    log
}

/// Version format strings of varying complexity
pub const SIMPLE_FORMAT: &str = "vCC.CC.CC";
#[allow(dead_code)]
pub const MEDIUM_FORMAT: &str = "vCC.CC.CC-alpha.CC";
#[allow(dead_code)]
pub const COMPLEX_FORMAT: &str = "vYY.CC.CC-alpha.CC";

#[allow(dead_code)]
pub const VERY_COMPLEX_FORMAT: &str = "vYYYYMMDD.CC.CC-alpha.CC+<short-sha>";

/// Sample conventional commit messages
#[allow(dead_code)]
pub const CONVENTIONAL_COMMITS: &[&str] = &[
    "feat: add new feature",
    "fix: resolve critical bug",
    "feat!: breaking change",
    "chore: update dependencies",
    "docs: improve documentation",
    "refactor: restructure code",
    "test: add unit tests",
    "ci: update workflow",
];

/// Sample non-conventional commit messages
#[allow(dead_code)]
pub const NON_CONVENTIONAL_COMMITS: &[&str] = &[
    "update readme",
    "WIP: work in progress",
    "Merge branch 'feature'",
    "Initial commit",
    "bugfix",
];

/// Sample version strings
#[allow(dead_code)]
pub const VERSION_STRINGS: &[&str] = &[
    "v1.0.0",
    "v2.3.4-alpha.1",
    "v10.20.30-rc.5+build.123",
    "v0.0.1",
];
