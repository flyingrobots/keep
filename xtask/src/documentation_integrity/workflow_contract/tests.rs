use std::path::Path;

use crate::repository_file::RepositoryRoot;

const WORKFLOW: &str = r#"name: CI
jobs:
  documentation:
    name: Documentation
    steps:
      - name: Install Rust
        run: rustup show
      - name: Install pinned Node.js
        uses: actions/setup-node@820762786026740c76f36085b0efc47a31fe5020
        with:
          node-version: 24.18.0
      - name: Install documentation tools
        run: |
          documentation_tools="$RUNNER_TEMP/documentation-tools"
          scripts/install_documentation_tools.sh "$documentation_tools"
          printf '%s\n' \
            "$documentation_tools/bin" \
            "$documentation_tools/npm/node_modules/.bin" >> "$GITHUB_PATH"
      - name: Verify malformed inputs
        run: |
          cargo test --locked --package xtask \
            documentation_integrity::execution::external_tests -- --ignored
      - name: Verify
        run: cargo xtask documentation-integrity-check
      - name: Check whitespace
        run: git diff --check "$(git hash-object -t tree /dev/null)" HEAD
  next-job:
    steps: []
"#;

#[test]
fn documentation_job_delegates_once_to_the_rust_boundary() {
    assert!(super::admit(WORKFLOW).is_ok());
}

#[test]
fn documentation_job_requires_the_malformed_input_regressions() {
    let runs = super::REVIEWED_RUNS
        .iter()
        .copied()
        .filter(|run| *run != super::MALFORMED_INPUT_COMMAND)
        .map(String::from)
        .collect::<Vec<_>>();

    assert!(!super::runs_are_reviewed(&runs));
}

#[test]
fn documentation_job_refuses_python_execution() {
    let workflow = WORKFLOW.replace(
        "  next-job:",
        "      - name: Legacy checker\n        run: python3 scripts/check_markdown.py\n  next-job:",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: concat!(
                "documentation job run commands are reviewed and required ",
                "commands execute once"
            ),
        })
    ));
}

#[test]
fn inert_yaml_cannot_impersonate_documentation_commands() {
    let workflow = r#"name: CI
jobs:
  documentation:
    steps:
      - name: Install pinned Node.js
        uses: actions/setup-node@820762786026740c76f36085b0efc47a31fe5020
        with:
          node-version: 24.18.0
      # run: rustup show
      - name: "run: cargo xtask documentation-integrity-check"
        uses: example/action@0123456789abcdef
"#;
    assert!(matches!(
        super::admit(workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: concat!(
                "documentation job run commands are reviewed and required ",
                "commands execute once"
            ),
        })
    ));
}

#[test]
fn non_string_run_values_are_refused() {
    let workflow = WORKFLOW.replace(
        "  next-job:",
        "      - name: Invalid executable\n        run: true\n  next-job:",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job run values are strings",
        })
    ));
}

#[test]
fn guarded_required_commands_do_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "      - name: Verify\n        run:",
        "      - name: Verify\n        if: false\n        run:",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job run steps are unguarded",
        })
    ));
}

#[test]
fn guarded_documentation_jobs_do_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace("  documentation:\n", "  documentation:\n    if: false\n");
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job is unguarded",
        })
    ));
}

#[test]
fn failure_tolerant_documentation_jobs_do_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "  documentation:\n",
        "  documentation:\n    continue-on-error: true\n",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job is failure-intolerant",
        })
    ));
}

#[test]
fn failure_tolerant_required_commands_do_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "      - name: Verify\n        run:",
        "      - name: Verify\n        continue-on-error: true\n        run:",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job run steps are failure-intolerant",
        })
    ));
}

#[test]
fn documentation_job_requires_the_pinned_node_action_once() {
    let setup = concat!(
        "      - name: Install pinned Node.js\n",
        "        uses: actions/setup-node@820762786026740c76f36085b0efc47a31fe5020\n",
        "        with:\n",
        "          node-version: 24.18.0\n"
    );
    let missing = WORKFLOW.replace(setup, "");
    assert!(matches!(
        super::admit(&missing),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation job installs pinned Node.js exactly once",
        })
    ));
}

#[test]
fn drifted_node_setup_does_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "actions/setup-node@820762786026740c76f36085b0efc47a31fe5020",
        "actions/setup-node@0123456789abcdef0123456789abcdef01234567",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation Node.js setup action is pinned",
        })
    ));
}

#[test]
fn additional_unpinned_node_setup_does_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "      - name: Install documentation tools\n",
        concat!(
            "      - name: Replace Node.js\n",
            "        uses: actions/setup-node@0123456789abcdef0123456789abcdef01234567\n",
            "        with:\n",
            "          node-version: 24.18.1\n",
            "      - name: Install documentation tools\n"
        ),
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation Node.js setup action is pinned",
        })
    ));
}

#[test]
fn guarded_node_setup_does_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "        uses: actions/setup-node@",
        "        if: false\n        uses: actions/setup-node@",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation Node.js setup is unguarded",
        })
    ));
}

#[test]
fn failure_tolerant_node_setup_does_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "        uses: actions/setup-node@",
        "        continue-on-error: true\n        uses: actions/setup-node@",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation Node.js setup is failure-intolerant",
        })
    ));
}

#[test]
fn documentation_job_requires_the_reviewed_node_version() {
    let workflow = WORKFLOW.replace(
        "          node-version: 24.18.0",
        "          node-version: 24.18.1",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: "documentation Node.js version is 24.18.0",
        })
    ));
}

#[test]
fn unreviewed_python_executables_are_refused() {
    let workflow = WORKFLOW.replace(
        "  next-job:",
        "      - name: Unreviewed executable\n        run: python --version\n  next-job:",
    );
    assert!(matches!(
        super::admit(&workflow),
        Err(super::DocumentationError::RepositoryContract {
            path: super::CI_PATH,
            requirement: concat!(
                "documentation job run commands are reviewed and required ",
                "commands execute once"
            ),
        })
    ));
}

#[test]
fn committed_documentation_job_uses_the_rust_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    let repository_root = RepositoryRoot::open(root)?;
    super::check(&repository_root)?;
    assert!(repository_root.is_current_path()?);
    Ok(())
}
