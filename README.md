# CCVer

> A zero dependency tool for conventional commits and semver

## Overview

CCVer is a command-line tool designed for automating version management in git repositories. It leverages conventional commit message conventions to help you:

- **Parse and validate commit messages**  
  Utilize a custom parser built with [Pest](https://pest.rs/) ([`src/parser/interpreter.rs`](src/parser/interpreter.rs) & [`src/parser/rules.pest`](src/parser/rules.pest)) to ensure commit messages adhere to a conventional format.

- **Automate semantic versioning**  
  Extract version and tagging information from commits to automatically bump version numbers following [semver](https://semver.org/) principles.


  ```mermaid
  flowchart TD
      A[git log] --> C[Parse raw logs with Pest]
      C --> D[Create DiGraph]
      D --> E[Construct Commit Graph<br>using `CommitGraphData::new`]
      E --> F[CommitGraph]
  ```

  ```mermaid
    gitGraph
        commit id: "initial commit" tag: "0.0.0"
        commit id: "unconventional commit" tag: "0.0.0-build.1"
        branch staging
        branch develop
        commit id: "feat: conventional commit" tag: "0.1.0-alpha.1"
        branch ryans-fix
        commit id: "chore: formatting" tag: "0.1.0-ryans-fix.1"
        checkout main
        merge ryans-fix id: "Merge branch 'ryans-fix'" tag: "0.1.0"
        checkout develop
        commit id: "fix: conventional commit" tag: "0.1.1-alpha.1"
        commit id: "whooops" tag: "0.1.1-alpha.2"
        checkout staging
        merge develop id: "Merge branch 'develop'" tag: "0.1.1-rc.1"
        merge main id: "Merge branch 'main'" tag: "0.1.1-rc.2"
        checkout main
        merge staging id: "Merge branch 'staging'" tag: "0.1.1"
        checkout develop
        commit type: HIGHLIGHT id: "uncommited changes" tag: "0.1.1-build.1"
  ```

- **Provide an extensible CLI**  
  Run various subcommands such as initializing (`Init`), installing hooks (`Install`), and tagging commits (`Tag`) to integrate version management into your workflow.

This tool is ideal for projects that want to maintain a clear commit history and manage releases automatically, all while ensuring that commit messages and version tags meet established conventions.

