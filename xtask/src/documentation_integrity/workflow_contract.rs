//! This module owns the CI documentation-job execution contract.

use yaml_rust2::{Yaml, YamlLoader};

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;

const CI_PATH: &str = ".github/workflows/ci.yml";
const CHECKOUT_ACTION: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const CHECKOUT_ACTION_PREFIX: &str = "actions/checkout@";
const MALFORMED_INPUT_COMMAND: &str = r"cargo test --locked --package xtask \
  documentation_integrity::execution::external_tests -- --ignored";
const SETUP_NODE_ACTION: &str = "actions/setup-node@820762786026740c76f36085b0efc47a31fe5020";
const SETUP_NODE_ACTION_PREFIX: &str = "actions/setup-node@";
const NODE_VERSION: &str = "24.18.0";
const XTASK_COMMAND: &str = "cargo xtask documentation-integrity-check";
const REVIEWED_RUNS: &[&str] = &[
    "rustup show",
    r#"documentation_tools="$RUNNER_TEMP/documentation-tools"
scripts/install_documentation_tools.sh "$documentation_tools"
printf '%s\n' \
  "$documentation_tools/bin" \
  "$documentation_tools/npm/node_modules/.bin" >> "$GITHUB_PATH""#,
    MALFORMED_INPUT_COMMAND,
    XTASK_COMMAND,
    r#"git diff --check "$(git hash-object -t tree /dev/null)" HEAD"#,
];

pub(super) fn check(repository_root: &RepositoryRoot) -> Result<(), DocumentationError> {
    let workflow = repository_text::read(repository_root, CI_PATH)?;
    admit(&workflow)
}

fn admit(workflow: &str) -> Result<(), DocumentationError> {
    let runs = documentation_runs(workflow)?;
    if !runs_are_reviewed(&runs) {
        return Err(contract(concat!(
            "documentation job run commands are reviewed and required ",
            "commands execute once"
        )));
    }
    Ok(())
}

fn documentation_runs(workflow: &str) -> Result<Vec<String>, DocumentationError> {
    let documents = YamlLoader::load_from_str(workflow).map_err(|source| {
        DocumentationError::RepositoryYaml {
            path: CI_PATH,
            source,
        }
    })?;
    let [document] = documents.as_slice() else {
        return Err(contract("workflow contains exactly one YAML document"));
    };
    reviewed_runs(documentation_steps(document)?)
}

fn documentation_steps(document: &Yaml) -> Result<&Vec<Yaml>, DocumentationError> {
    let job = &document["jobs"]["documentation"];
    if !job["if"].is_badvalue() {
        return Err(contract("documentation job is unguarded"));
    }
    if !job["continue-on-error"].is_badvalue() {
        return Err(contract("documentation job is failure-intolerant"));
    }
    let Some(steps) = job["steps"].as_vec() else {
        return Err(contract("workflow defines documentation job steps"));
    };
    Ok(steps)
}

fn reviewed_runs(steps: &[Yaml]) -> Result<Vec<String>, DocumentationError> {
    let mut runs = Vec::new();
    let mut actions = Vec::new();
    for step in steps {
        actions.extend(admit_action(step)?);
        runs.extend(admit_run(step)?);
    }
    if actions
        .iter()
        .filter(|action| **action == DocumentationAction::Node)
        .count()
        != 1
    {
        return Err(contract(
            "documentation job installs pinned Node.js exactly once",
        ));
    }
    if actions.as_slice() != REVIEWED_ACTIONS {
        return Err(contract(
            "documentation job actions execute in reviewed order",
        ));
    }
    Ok(runs)
}

fn admit_action(step: &Yaml) -> Result<Option<DocumentationAction>, DocumentationError> {
    let uses = &step["uses"];
    if uses.is_badvalue() {
        return Ok(None);
    }
    let Some(action) = uses.as_str() else {
        return Err(contract("documentation job action values are strings"));
    };
    if action.starts_with(CHECKOUT_ACTION_PREFIX) {
        return admit_checkout(step, action).map(Some);
    }
    if action.starts_with(SETUP_NODE_ACTION_PREFIX) {
        return admit_node_setup(step, action).map(Some);
    }
    Err(contract("documentation job action steps are reviewed"))
}

fn admit_checkout(step: &Yaml, action: &str) -> Result<DocumentationAction, DocumentationError> {
    if action != CHECKOUT_ACTION {
        return Err(contract("documentation checkout action is pinned"));
    }
    admit_action_execution(
        step,
        "documentation checkout is unguarded",
        "documentation checkout is failure-intolerant",
    )?;
    let exact = step["with"]
        .as_hash()
        .is_some_and(|configuration| configuration.len() == 1)
        && step["with"]["persist-credentials"].as_bool() == Some(false);
    if !exact {
        return Err(contract("documentation checkout configuration is exact"));
    }
    Ok(DocumentationAction::Checkout)
}

fn admit_node_setup(step: &Yaml, action: &str) -> Result<DocumentationAction, DocumentationError> {
    if action != SETUP_NODE_ACTION {
        return Err(contract("documentation Node.js setup action is pinned"));
    }
    admit_action_execution(
        step,
        "documentation Node.js setup is unguarded",
        "documentation Node.js setup is failure-intolerant",
    )?;
    let exact = step["with"]
        .as_hash()
        .is_some_and(|configuration| configuration.len() == 1)
        && step["with"]["node-version"].as_str() == Some(NODE_VERSION);
    if !exact {
        return Err(contract("documentation Node.js version is 24.18.0"));
    }
    Ok(DocumentationAction::Node)
}

fn admit_action_execution(
    step: &Yaml,
    guard_requirement: &'static str,
    failure_requirement: &'static str,
) -> Result<(), DocumentationError> {
    if !step["if"].is_badvalue() {
        return Err(contract(guard_requirement));
    }
    if !step["continue-on-error"].is_badvalue() {
        return Err(contract(failure_requirement));
    }
    if !step["run"].is_badvalue() {
        return Err(contract("documentation action steps do not define run"));
    }
    Ok(())
}

fn admit_run(step: &Yaml) -> Result<Option<String>, DocumentationError> {
    let run = &step["run"];
    if run.is_badvalue() {
        return Ok(None);
    }
    if !step["if"].is_badvalue() {
        return Err(contract("documentation job run steps are unguarded"));
    }
    if !step["continue-on-error"].is_badvalue() {
        return Err(contract(
            "documentation job run steps are failure-intolerant",
        ));
    }
    let Some(run) = run.as_str() else {
        return Err(contract("documentation job run values are strings"));
    };
    Ok(Some(run.trim_end_matches('\n').to_owned()))
}

fn runs_are_reviewed(runs: &[String]) -> bool {
    runs.len() == REVIEWED_RUNS.len()
        && REVIEWED_RUNS
            .iter()
            .all(|required| runs.iter().filter(|run| run.as_str() == *required).count() == 1)
}

const REVIEWED_ACTIONS: &[DocumentationAction] =
    &[DocumentationAction::Checkout, DocumentationAction::Node];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DocumentationAction {
    Checkout,
    Node,
}

const fn contract(requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract {
        path: CI_PATH,
        requirement,
    }
}

#[cfg(test)]
#[path = "workflow_contract/tests.rs"]
mod tests;
