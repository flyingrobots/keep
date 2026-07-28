//! This module owns documentation-job action-step policy regressions.

use super::super::{CI_PATH, DocumentationError, admit};
use super::WORKFLOW;

const CHECKOUT_STEP: &str = concat!(
    "      - name: Check out repository\n",
    "        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n",
    "        with:\n",
    "          persist-credentials: false\n"
);
const NODE_STEP: &str = concat!(
    "      - name: Install pinned Node.js\n",
    "        uses: actions/setup-node@820762786026740c76f36085b0efc47a31fe5020\n",
    "        with:\n",
    "          node-version: 24.18.0\n"
);

#[test]
fn checkout_of_an_unreviewed_revision_does_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "          persist-credentials: false",
        "          persist-credentials: false\n          ref: main",
    );
    assert!(matches!(
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
            requirement: "documentation checkout configuration is exact",
        })
    ));
}

#[test]
fn drifted_checkout_action_does_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
        "actions/checkout@0123456789abcdef0123456789abcdef01234567",
    );
    assert!(matches!(
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
            requirement: "documentation checkout action is pinned",
        })
    ));
}

#[test]
fn unreviewed_action_steps_do_not_satisfy_the_contract() {
    let workflow = WORKFLOW.replace(
        "      - name: Install Rust\n",
        concat!(
            "      - name: Unreviewed action\n",
            "        uses: example/action@0123456789abcdef0123456789abcdef01234567\n",
            "      - name: Install Rust\n"
        ),
    );
    assert!(matches!(
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
            requirement: "documentation job action steps are reviewed",
        })
    ));
}

#[test]
fn documentation_actions_execute_in_reviewed_order() {
    let workflow = WORKFLOW
        .replace(CHECKOUT_STEP, "CHECKOUT_STEP_PLACEHOLDER\n")
        .replace(NODE_STEP, CHECKOUT_STEP)
        .replace("CHECKOUT_STEP_PLACEHOLDER\n", NODE_STEP);
    assert!(matches!(
        admit(&workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
            requirement: "documentation job actions execute in reviewed order",
        })
    ));
}
