use crate::logs::{ConventionalSubject, Subject};

pub macro release_branches() {
    "main" | "master" | "release"
}

pub macro rc_branches() {
    "staging" | "rc"
}

pub macro beta_branches() {
    "development" | "beta"
}

pub macro alpha_branches() {
    "next" | "alpha"
}

pub macro patch_commit_types() {
    "fix" | "bug" | "patch"
}

pub macro minor_commit_types() {
    "feat" | "feature" | "minor"
}

pub macro major_commit_types() {
    "breaking" | "major"
}

pub macro major_conventional_subject() {
    ConventionalSubject { breaking: true, .. }
        | ConventionalSubject {
            commit_type: major_commit_types!(),
            ..
        }
}

pub macro major_subject() {
    Subject::Conventional(major_conventional_subject!())
}

pub macro patch_conventional_subject() {
    ConventionalSubject {
        commit_type: patch_commit_types!(),
        ..
    }
}

pub macro patch_subject() {
    Subject::Conventional(patch_conventional_subject!())
}

pub macro minor_conventional_subject() {
    ConventionalSubject {
        commit_type: minor_commit_types!(),
        ..
    }
}

pub macro minor_subject() {
    Subject::Conventional(minor_conventional_subject!())
}

pub macro semver_advancing_conventional_subject() {
    major_conventional_subject!() | minor_conventional_subject!() | patch_conventional_subject!()
}

pub macro semver_advancing_subject() {
    Subject::Conventional(semver_advancing_conventional_subject!())
}
