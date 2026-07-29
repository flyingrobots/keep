use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;

const CI: &str = include_str!("../../../.github/workflows/ci.yml");
const SCHEDULED: &str = include_str!("../../../.github/workflows/fuzz-scheduled.yml");

#[test]
fn checkout_pin_is_consistent_across_fuzz_workflows() {
    let ci = checkout_references(CI);
    let scheduled = checkout_references(SCHEDULED);
    assert_eq!(ci.len(), 1);
    assert_eq!(scheduled, ci);
}

#[test]
fn every_fuzz_workflow_action_uses_an_immutable_commit() -> Result<(), Box<dyn Error>> {
    for workflow in workflow_sources()? {
        let references = action_references(&workflow).collect::<Vec<_>>();
        if references.is_empty() {
            return Err("workflow has no third-party action references".into());
        }
        for (_, reference) in references {
            if reference.len() != 40
                || !reference
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(format!("workflow action is not pinned: {reference}").into());
            }
        }
    }
    Ok(())
}

#[test]
fn scheduled_campaign_is_restricted_to_main() {
    assert!(SCHEDULED.contains("  schedule:\n"));
    assert!(SCHEDULED.contains("  workflow_dispatch:\n"));
    assert!(SCHEDULED.contains("    if: github.ref == 'refs/heads/main'\n"));
}

#[test]
fn refused_corpus_restore_removes_a_root_link() -> Result<(), Box<dyn Error>> {
    let discard = step(SCHEDULED, "Discard a refused corpus restore")?;
    assert!(discard.contains("test -L fuzz/corpus"));
    assert!(discard.contains("find -P fuzz/corpus -depth -delete"));
    Ok(())
}

#[test]
fn only_successfully_minimized_corpora_are_retained() -> Result<(), Box<dyn Error>> {
    for name in [
        "Save non-authoritative evolving corpus",
        "Retain minimized corpus evidence",
    ] {
        let retained = step(SCHEDULED, name)?;
        assert!(retained.contains("steps.campaign.outcome == 'success'"));
        assert!(retained.contains("steps.minimize.outcome == 'success'"));
        assert!(retained.contains("steps.retained_corpus.outcome == 'success'"));
    }
    Ok(())
}

#[test]
fn fuzz_workflows_delegate_campaign_policy_and_execution_to_xtask() -> Result<(), Box<dyn Error>> {
    let (_, ci_fuzz) = CI
        .split_once("  fuzz-smoke:\n")
        .ok_or("CI has no fuzz-smoke job")?;
    let (ci_fuzz, _) = ci_fuzz
        .split_once("\n  dependency-policy:")
        .ok_or("CI fuzz-smoke job has no closing job")?;
    for workflow in [ci_fuzz, SCHEDULED] {
        assert!(!workflow.contains("python"));
        assert!(!workflow.contains(".py"));
        assert!(workflow.contains("cargo xtask fuzz github-env"));
        let build = workflow
            .find("cargo xtask fuzz build")
            .ok_or("fuzz workflow does not build targets")?;
        let run = workflow
            .find("cargo xtask fuzz run")
            .ok_or("fuzz workflow does not run targets")?;
        assert!(build < run);
    }
    assert!(SCHEDULED.contains("cargo xtask fuzz check-corpus"));
    assert!(SCHEDULED.contains("cargo xtask fuzz minimize"));
    Ok(())
}

fn checkout_references(workflow: &str) -> BTreeSet<&str> {
    action_references(workflow)
        .filter_map(|(action, reference)| (action == "actions/checkout").then_some(reference))
        .collect()
}

fn action_references(workflow: &str) -> impl Iterator<Item = (&str, &str)> {
    workflow.lines().filter_map(|line| {
        line.trim()
            .strip_prefix("uses: ")
            .and_then(|reference| reference.split_once('@'))
            .and_then(|(action, reference)| {
                reference
                    .split_whitespace()
                    .next()
                    .map(|reference| (action, reference))
            })
    })
}

fn workflow_sources() -> Result<Vec<String>, Box<dyn Error>> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository_root = manifest.parent().map_or(manifest, |parent| parent);
    let directory = repository_root.join(".github/workflows");
    let mut sources = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let extension = path.extension().and_then(|value| value.to_str());
        if !matches!(extension, Some("yaml" | "yml")) {
            continue;
        }
        if !entry.file_type()?.is_file() {
            return Err(format!("workflow is not a regular file: {}", path.display()).into());
        }
        sources.push((entry.file_name(), fs::read_to_string(path)?));
    }
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    if sources.is_empty() {
        return Err("repository workflow corpus is empty".into());
    }
    Ok(sources.into_iter().map(|(_, source)| source).collect())
}

fn step<'a>(workflow: &'a str, name: &str) -> Result<&'a str, Box<dyn Error>> {
    let marker = format!("      - name: {name}\n");
    let (_, tail) = workflow
        .split_once(&marker)
        .ok_or_else(|| format!("workflow has no step named {name}"))?;
    Ok(tail.split("\n      - name:").next().unwrap_or(tail))
}
