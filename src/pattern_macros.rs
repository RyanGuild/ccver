#[macro_export]
macro_rules! release_branches {
    () => {
        "main" | "master" | "release"
    };
}

#[macro_export]
macro_rules! rc_branches {
    () => {
        "staging" | "rc"
    };
}

#[macro_export]
macro_rules! beta_branches {
    () => {
        "development" | "develop" | "beta"
    };
}

#[macro_export]
macro_rules! alpha_branches {
    () => {
        "next" | "alpha"
    };
}

#[macro_export]
macro_rules! patch_commit_types {
    () => {
        "fix" | "bug" | "patch"
    };
}

#[macro_export]
macro_rules! minor_commit_types {
    () => {
        "feat" | "feature" | "minor"
    };
}

#[macro_export]
macro_rules! major_commit_types {
    () => {
        "breaking" | "major"
    };
}

#[macro_export]
macro_rules! major_conventional_subject {
    () => {
        $crate::logs::ConventionalSubject { breaking: true, .. }
            | $crate::logs::ConventionalSubject {
                commit_type: major_commit_types!(),
                ..
            }
    };
}

#[macro_export]
macro_rules! major_subject {
    () => {
        $crate::logs::Subject::Conventional(major_conventional_subject!())
    };
}

#[macro_export]
macro_rules! patch_conventional_subject {
    () => {
        $crate::logs::ConventionalSubject {
            commit_type: patch_commit_types!(),
            ..
        }
    };
}

#[macro_export]
macro_rules! patch_subject {
    () => {
        $crate::logs::Subject::Conventional(patch_conventional_subject!())
    };
}

#[macro_export]
macro_rules! minor_conventional_subject {
    () => {
        $crate::logs::ConventionalSubject {
            commit_type: minor_commit_types!(),
            ..
        }
    };
}

#[macro_export]
macro_rules! minor_subject {
    () => {
        $crate::logs::Subject::Conventional(minor_conventional_subject!())
    };
}

#[macro_export]
macro_rules! semver_advancing_conventional_subject {
    () => {
        major_conventional_subject!()
            | minor_conventional_subject!()
            | patch_conventional_subject!()
    };
}

#[macro_export]
macro_rules! semver_advancing_subject {
    () => {
        $crate::logs::Subject::Conventional(semver_advancing_conventional_subject!())
    };
}
