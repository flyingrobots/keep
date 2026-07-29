//! This module owns the exact documentation-job step sequence.

const MALFORMED_INPUT_COMMAND: &str = r"cargo test --locked --package xtask \
  documentation_integrity::execution::external_tests -- --ignored";
const INSTALL_TOOLS_COMMAND: &str = r#"documentation_tools="$RUNNER_TEMP/documentation-tools"
scripts/install_documentation_tools.sh "$documentation_tools"
printf '%s\n' \
  "$documentation_tools/bin" \
  "$documentation_tools/npm/node_modules/.bin" >> "$GITHUB_PATH""#;
const DIFF_CHECK_COMMAND: &str = r#"git diff --check "$(git hash-object -t tree /dev/null)" HEAD"#;

/// The only admitted documentation-job execution sequence.
pub(super) const REVIEWED_STEPS: &[DocumentationStep] = &[
    DocumentationStep::Checkout,
    DocumentationStep::Rustup,
    DocumentationStep::Node,
    DocumentationStep::InstallTools,
    DocumentationStep::MalformedInputs,
    DocumentationStep::Verify,
    DocumentationStep::DiffCheck,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// One semantically reviewed documentation-job step.
pub(super) enum DocumentationStep {
    /// Opens the exact repository revision without retained credentials.
    Checkout,
    /// Confirms the pinned Rust toolchain selected by repository policy.
    Rustup,
    /// Installs the pinned Node.js runtime.
    Node,
    /// Installs the lockfile-bound documentation tools.
    InstallTools,
    /// Runs malformed-input regression evidence.
    MalformedInputs,
    /// Runs the Rust documentation-integrity boundary.
    Verify,
    /// Refuses whitespace errors across the reviewed tree.
    DiffCheck,
}

impl DocumentationStep {
    /// Classifies one exact reviewed `run` body.
    pub(super) fn from_run(run: &str) -> Option<Self> {
        match run {
            "rustup show" => Some(Self::Rustup),
            INSTALL_TOOLS_COMMAND => Some(Self::InstallTools),
            MALFORMED_INPUT_COMMAND => Some(Self::MalformedInputs),
            "cargo xtask documentation-integrity-check" => Some(Self::Verify),
            DIFF_CHECK_COMMAND => Some(Self::DiffCheck),
            _ => None,
        }
    }
}

/// Reports whether every reviewed step occurs exactly once.
pub(super) fn steps_have_reviewed_membership(steps: &[DocumentationStep]) -> bool {
    steps.len() == REVIEWED_STEPS.len()
        && REVIEWED_STEPS
            .iter()
            .all(|required| steps.iter().filter(|step| *step == required).count() == 1)
}
