//! This module owns documentation-job Node.js setup policy regressions.

use super::super::{CI_PATH, DocumentationError, admit};
use super::WORKFLOW;

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
        admit(&missing),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
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
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
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
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
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
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
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
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
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
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
            requirement: "documentation Node.js version is 24.18.0",
        })
    ));
}
