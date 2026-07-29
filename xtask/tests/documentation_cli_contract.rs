//! Subprocess contract for repository-wide verification with hermetic tools.

#![cfg(all(feature = "repository-tasks", unix))]

#[path = "cli_contract/documentation_tools.rs"]
mod documentation_tools;

use std::io;

const DOCUMENTATION_ERROR_DISPLAY: &str =
    include_str!("../src/documentation_integrity/error/display.rs");
const WORKFLOW_CONTRACT: &str = include_str!("../src/documentation_integrity/workflow_contract.rs");

#[test]
fn documentation_error_formatter_stays_below_the_hard_function_limit() -> Result<(), &'static str> {
    let (_, after_signature) = DOCUMENTATION_ERROR_DISPLAY
        .split_once("    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {")
        .ok_or("display implementation must retain its formatter")?;
    let (body, _) = after_signature
        .split_once("\n    }\n}\n\nfn ")
        .ok_or("display formatter must remain a directly inspectable function")?;
    assert!(
        body.lines().count() <= 59,
        "DocumentationError::fmt exceeds the 60-line hard limit"
    );
    Ok(())
}

#[test]
fn workflow_admission_parser_stays_below_the_hard_function_limit() -> Result<(), &'static str> {
    let (_, after_signature) = WORKFLOW_CONTRACT
        .split_once(
            "fn admitted_steps(workflow: &str) -> Result<Vec<DocumentationStep>, DocumentationError> {",
        )
        .ok_or("workflow contract must retain its parser")?;
    let (body, _) = after_signature
        .split_once("\n}\n\nfn triggers_are_reviewed")
        .ok_or("workflow parser must remain a directly inspectable function")?;
    assert!(
        body.lines().count() <= 59,
        "admitted_steps exceeds the 60-line hard limit"
    );
    Ok(())
}

#[test]
fn successful_verification_runs_every_documentation_tool_silently() -> Result<(), io::Error> {
    let tools = documentation_tools::DocumentationTools::create()?;
    let output = tools.invoke(&["verify"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    tools.require_every_tool()?;
    tools.close()?;
    Ok(())
}
