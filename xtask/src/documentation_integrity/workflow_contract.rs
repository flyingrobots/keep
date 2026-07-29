//! This module owns the CI documentation-job execution contract.

mod reviewed_step;

use yaml_rust2::{Yaml, YamlLoader};

use crate::repository_file::RepositoryRoot;

use super::error::DocumentationError;
use super::repository_text;
use reviewed_step::{DocumentationStep, REVIEWED_STEPS, steps_have_reviewed_membership};

const CI_PATH: &str = ".github/workflows/ci.yml";
const CHECKOUT_ACTION: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const CHECKOUT_ACTION_PREFIX: &str = "actions/checkout@";
const DOCUMENTATION_JOB_FIELDS: &[&str] = &["name", "runs-on", "timeout-minutes", "steps"];
const DOCUMENTATION_JOB_NAME: &str = "Documentation and workflow integrity";
const DOCUMENTATION_RUNNER: &str = "ubuntu-latest";
const DOCUMENTATION_TIMEOUT_MINUTES: i64 = 10;
const SETUP_NODE_ACTION: &str = "actions/setup-node@820762786026740c76f36085b0efc47a31fe5020";
const SETUP_NODE_ACTION_PREFIX: &str = "actions/setup-node@";
const NODE_VERSION: &str = "24.18.0";

pub(super) fn check(repository_root: &RepositoryRoot) -> Result<(), DocumentationError> {
    let workflow = repository_text::read(repository_root, CI_PATH)?;
    admit(&workflow)
}

fn admit(workflow: &str) -> Result<(), DocumentationError> {
    let steps = admitted_steps(workflow)?;
    if steps.as_slice() == REVIEWED_STEPS {
        return Ok(());
    }
    if steps_have_reviewed_membership(&steps) {
        Err(contract(
            "documentation job steps execute in reviewed order",
        ))
    } else {
        Err(contract(concat!(
            "documentation job run commands are reviewed and required ",
            "commands execute once"
        )))
    }
}

fn admitted_steps(workflow: &str) -> Result<Vec<DocumentationStep>, DocumentationError> {
    let documents = YamlLoader::load_from_str(workflow).map_err(|source| {
        DocumentationError::RepositoryYaml {
            path: CI_PATH,
            source,
        }
    })?;
    let [document] = documents.as_slice() else {
        return Err(contract("workflow contains exactly one YAML document"));
    };
    if !triggers_are_reviewed(document) {
        return Err(contract(
            "workflow runs on reviewed push and pull request triggers",
        ));
    }
    reviewed_steps(documentation_steps(document)?)
}

fn triggers_are_reviewed(document: &Yaml) -> bool {
    let triggers = &document["on"];
    let push = &triggers["push"];
    let branches = &push["branches"];
    mapping_has_exact_fields(triggers, &["push", "pull_request"])
        && mapping_has_exact_fields(push, &["branches"])
        && matches!(branches.as_vec().map(Vec::as_slice), Some([branch]) if branch.as_str() == Some("main"))
        && triggers["pull_request"].is_null()
}

fn documentation_steps(document: &Yaml) -> Result<&Vec<Yaml>, DocumentationError> {
    if !document["defaults"].is_badvalue() || !document["env"].is_badvalue() {
        return Err(contract(
            "workflow does not override documentation execution",
        ));
    }
    let job = &document["jobs"]["documentation"];
    if !job["if"].is_badvalue() {
        return Err(contract("documentation job is unguarded"));
    }
    if !job["continue-on-error"].is_badvalue() {
        return Err(contract("documentation job is failure-intolerant"));
    }
    if job["name"].as_str() != Some(DOCUMENTATION_JOB_NAME) {
        return Err(contract("documentation job name is reviewed"));
    }
    if job["runs-on"].as_str() != Some(DOCUMENTATION_RUNNER) {
        return Err(contract("documentation job uses ubuntu-latest"));
    }
    if job["timeout-minutes"].as_i64() != Some(DOCUMENTATION_TIMEOUT_MINUTES) {
        return Err(contract("documentation job timeout is ten minutes"));
    }
    if !mapping_has_exact_fields(job, DOCUMENTATION_JOB_FIELDS) {
        return Err(contract("documentation job fields are reviewed"));
    }
    let Some(steps) = job["steps"].as_vec() else {
        return Err(contract("workflow defines documentation job steps"));
    };
    Ok(steps)
}

fn reviewed_steps(steps: &[Yaml]) -> Result<Vec<DocumentationStep>, DocumentationError> {
    let mut admitted = Vec::new();
    let mut actions = Vec::new();
    for step in steps {
        if let Some(action) = admit_action(step)? {
            actions.push(action);
            admitted.push(match action {
                DocumentationAction::Checkout => DocumentationStep::Checkout,
                DocumentationAction::Node => DocumentationStep::Node,
            });
        } else if let Some(run) = admit_run(step)? {
            admitted.push(run);
        }
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
    Ok(admitted)
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
    if !mapping_has_exact_fields(step, &["name", "uses", "with"]) {
        return Err(contract("documentation action step fields are reviewed"));
    }
    Ok(())
}

fn admit_run(step: &Yaml) -> Result<Option<DocumentationStep>, DocumentationError> {
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
    if !mapping_has_exact_fields(step, &["name", "run"]) {
        return Err(contract("documentation job run step fields are reviewed"));
    }
    let Some(run) = run.as_str() else {
        return Err(contract("documentation job run values are strings"));
    };
    let Some(admitted) = DocumentationStep::from_run(run.trim_end_matches('\n')) else {
        return Err(contract(concat!(
            "documentation job run commands are reviewed and required ",
            "commands execute once"
        )));
    };
    Ok(Some(admitted))
}

fn mapping_has_exact_fields(mapping: &Yaml, fields: &[&str]) -> bool {
    mapping.as_hash().is_some_and(|mapping| {
        mapping.len() == fields.len()
            && mapping
                .keys()
                .all(|field| field.as_str().is_some_and(|field| fields.contains(&field)))
    })
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
