use std::collections::VecDeque;
use std::error::Error;
use std::io;
use std::path::Path;

use super::{CommandRunner, execute_all};
use crate::bounded_process::{ProcessError, ProcessOutput};
use crate::fuzz_campaign::command::{CampaignOperation, CommandPlan};
use crate::fuzz_campaign::policy::CampaignPolicy;
use crate::fuzz_campaign::profile::CampaignProfile;
use crate::fuzz_campaign::target::FuzzTarget;

#[test]
fn every_target_runs_after_an_earlier_failure() -> Result<(), Box<dyn Error>> {
    let plans = plans(CampaignOperation::Run(CampaignProfile::Smoke))?;
    let mut runner = ScriptedRunner::new([
        output(false, b"", b"first failed"),
        output(true, b"", b""),
        output(true, b"", b""),
    ]);
    let Err(error) = execute_all(Path::new("."), "campaign", &plans, &mut runner) else {
        return Err("the first scripted target must fail".into());
    };
    assert_eq!(runner.observed, ["first", "second", "third"]);
    let failed_targets = error
        .failures
        .iter()
        .map(|failure| failure.target.as_str())
        .collect::<Vec<_>>();
    assert_eq!(failed_targets, ["first"]);
    Ok(())
}

#[test]
fn swallowed_minimization_failure_is_refused() -> Result<(), Box<dyn Error>> {
    let policy = policy()?;
    let target = FuzzTarget::admit("first".to_owned())?;
    let plan = CommandPlan::new(&policy, CampaignOperation::Minimize, target)?;
    let mut runner =
        ScriptedRunner::new([output(true, b"Failed to minimize corpus: signal 6", b"")]);
    let Err(error) = execute_all(Path::new("."), "corpus minimization", &[plan], &mut runner)
    else {
        return Err("the cargo-fuzz failure marker must be refused".into());
    };
    let failed_targets = error
        .failures
        .iter()
        .map(|failure| failure.target.as_str())
        .collect::<Vec<_>>();
    assert_eq!(failed_targets, ["first"]);
    Ok(())
}

#[test]
fn aggregated_process_failures_preserve_a_typed_source() -> Result<(), Box<dyn Error>> {
    let plans = plans(CampaignOperation::Build)?;
    let mut runner = RefusingRunner;
    let Err(error) = execute_all(Path::new("."), "build", &plans, &mut runner) else {
        return Err("the scripted process failure must be aggregated".into());
    };
    let source = Error::source(&error).ok_or("aggregate has no process source")?;
    assert_eq!(source.to_string(), "cannot spawn cargo-fuzz process");
    Ok(())
}

fn plans(operation: CampaignOperation) -> Result<Vec<CommandPlan>, Box<dyn Error>> {
    let policy = policy()?;
    ["first", "second", "third"]
        .into_iter()
        .map(|name| {
            let target = FuzzTarget::admit(name.to_owned())?;
            Ok(CommandPlan::new(&policy, operation, target)?)
        })
        .collect()
}

fn policy() -> Result<CampaignPolicy, Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    Ok(CampaignPolicy::load(root)?)
}

fn output(succeeded: bool, stdout: &[u8], stderr: &[u8]) -> ProcessOutput {
    ProcessOutput {
        code: Some(i32::from(!succeeded)),
        succeeded,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

struct ScriptedRunner {
    outcomes: VecDeque<ProcessOutput>,
    observed: Vec<String>,
}

struct RefusingRunner;

impl CommandRunner for RefusingRunner {
    fn execute(
        &mut self,
        _repository_root: &Path,
        _plan: &CommandPlan,
    ) -> Result<ProcessOutput, ProcessError> {
        Err(ProcessError::Io {
            program: "cargo-fuzz",
            action: "spawn",
            source: io::Error::other("scripted refusal"),
        })
    }
}

impl ScriptedRunner {
    fn new(outcomes: impl IntoIterator<Item = ProcessOutput>) -> Self {
        Self {
            outcomes: outcomes.into_iter().collect(),
            observed: Vec::new(),
        }
    }
}

impl CommandRunner for ScriptedRunner {
    fn execute(
        &mut self,
        _repository_root: &Path,
        plan: &CommandPlan,
    ) -> Result<ProcessOutput, ProcessError> {
        self.observed.push(plan.target().as_str().to_owned());
        let Some(output) = self.outcomes.pop_front() else {
            return Err(ProcessError::MissingStream {
                program: "cargo-fuzz",
                stream: "scripted outcome",
            });
        };
        Ok(output)
    }
}
