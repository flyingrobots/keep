//! This module owns the CI documentation-job execution contract.

use yaml_rust2::{Yaml, YamlLoader};

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;

const CI_PATH: &str = ".github/workflows/ci.yml";
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
    let mut node_setup_seen = false;
    for step in steps {
        if admit_node_setup(step)?.is_some() {
            if node_setup_seen {
                return Err(contract(
                    "documentation job installs pinned Node.js exactly once",
                ));
            }
            node_setup_seen = true;
        }
        runs.extend(admit_run(step)?);
    }
    if !node_setup_seen {
        return Err(contract(
            "documentation job installs pinned Node.js exactly once",
        ));
    }
    Ok(runs)
}

fn admit_node_setup(step: &Yaml) -> Result<Option<()>, DocumentationError> {
    let action = step["uses"].as_str();
    if !action.is_some_and(|value| value.starts_with(SETUP_NODE_ACTION_PREFIX)) {
        return Ok(None);
    }
    if action != Some(SETUP_NODE_ACTION) {
        return Err(contract("documentation Node.js setup action is pinned"));
    }
    if !step["if"].is_badvalue() {
        return Err(contract("documentation Node.js setup is unguarded"));
    }
    if !step["continue-on-error"].is_badvalue() {
        return Err(contract(
            "documentation Node.js setup is failure-intolerant",
        ));
    }
    if step["with"]["node-version"].as_str() != Some(NODE_VERSION) {
        return Err(contract("documentation Node.js version is 24.18.0"));
    }
    Ok(Some(()))
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

const fn contract(requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract {
        path: CI_PATH,
        requirement,
    }
}

#[cfg(test)]
#[path = "workflow_contract/tests.rs"]
mod tests;
