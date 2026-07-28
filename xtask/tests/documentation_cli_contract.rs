//! Subprocess contract for repository-wide verification with hermetic tools.

#![cfg(all(feature = "repository-tasks", unix))]

#[path = "cli_contract/documentation_tools.rs"]
mod documentation_tools;

use std::io;

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
