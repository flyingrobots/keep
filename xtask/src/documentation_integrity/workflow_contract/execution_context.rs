//! This module owns documentation-job execution-context regressions.

use super::super::{CI_PATH, DocumentationError, admit};
use super::WORKFLOW;

#[test]
fn custom_run_shell_cannot_impersonate_a_required_command() {
    let workflow = WORKFLOW.replace(
        "      - name: Verify\n        run:",
        "      - name: Verify\n        shell: \"echo {0}\"\n        run:",
    );

    assert_contract(&workflow, "documentation job run step fields are reviewed");
}

#[test]
fn workflow_run_defaults_cannot_replace_required_commands() {
    let workflow = WORKFLOW.replace(
        "jobs:\n",
        "defaults:\n  run:\n    shell: \"echo {0}\"\njobs:\n",
    );

    assert_contract(
        &workflow,
        "workflow does not override documentation execution",
    );
}

#[test]
fn job_run_defaults_cannot_replace_required_commands() {
    let workflow = WORKFLOW.replace(
        "    steps:\n",
        "    defaults:\n      run:\n        shell: \"echo {0}\"\n    steps:\n",
    );

    assert_contract(&workflow, "documentation job fields are reviewed");
}

#[test]
fn documentation_job_cannot_move_to_an_unreviewed_runner() {
    let workflow = WORKFLOW.replace("runs-on: ubuntu-latest", "runs-on: self-hosted");

    assert_contract(&workflow, "documentation job uses ubuntu-latest");
}

#[test]
fn documentation_job_cannot_extend_its_reviewed_deadline() {
    let workflow = WORKFLOW.replace("timeout-minutes: 10", "timeout-minutes: 60");

    assert_contract(&workflow, "documentation job timeout is ten minutes");
}

#[test]
fn action_environment_cannot_change_reviewed_execution() {
    let workflow = WORKFLOW.replace(
        "      - name: Install pinned Node.js\n        uses:",
        "      - name: Install pinned Node.js\n        env:\n          NODE_OPTIONS: --require=./hook.js\n        uses:",
    );

    assert_contract(&workflow, "documentation action step fields are reviewed");
}

fn assert_contract(workflow: &str, requirement: &'static str) {
    assert!(matches!(
        admit(workflow),
        Err(DocumentationError::RepositoryContract {
            path: CI_PATH,
            requirement: observed,
        }) if observed == requirement
    ));
}
