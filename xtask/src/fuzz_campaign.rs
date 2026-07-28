//! This module owns the repository fuzz campaign command boundary.

mod command;
mod error;
mod execution;
mod policy;
mod process;
mod profile;
mod target;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use command::{CampaignOperation, CommandPlan};
pub(crate) use error::FuzzCampaignError;
use policy::CampaignPolicy;
use profile::CampaignProfile;

enum Operation {
    Build,
    Describe,
    GitHubEnvironment,
    List,
    Minimize,
    Run,
}

pub(super) fn run(
    repository_root: &Path,
    mut arguments: impl Iterator<Item = OsString>,
    output: &mut impl Write,
) -> Result<(), FuzzCampaignError> {
    let operation = parse_operation(next_argument(&mut arguments)?)?;
    let policy = CampaignPolicy::load(repository_root)?;
    match operation {
        Operation::Build => execute(
            repository_root,
            &policy,
            CampaignOperation::Build,
            "build",
            &mut arguments,
        ),
        Operation::Describe => {
            write_environment(&policy, parse_profile(&mut arguments)?, ": ", output)
        }
        Operation::GitHubEnvironment => {
            write_environment(&policy, parse_profile(&mut arguments)?, "=", output)
        }
        Operation::List => {
            refuse_extra(&mut arguments)?;
            for target in target::registered(repository_root, &policy)? {
                writeln!(output, "{}", target.as_str()).map_err(FuzzCampaignError::Output)?;
            }
            Ok(())
        }
        Operation::Minimize => execute(
            repository_root,
            &policy,
            CampaignOperation::Minimize,
            "corpus minimization",
            &mut arguments,
        ),
        Operation::Run => {
            let profile = parse_profile(&mut arguments)?;
            execute_plans(
                repository_root,
                &policy,
                CampaignOperation::Run(profile),
                "campaign",
            )
        }
    }
}

fn execute(
    repository_root: &Path,
    policy: &CampaignPolicy,
    operation: CampaignOperation,
    name: &'static str,
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<(), FuzzCampaignError> {
    let _profile = parse_profile(arguments)?;
    execute_plans(repository_root, policy, operation, name)
}

fn execute_plans(
    repository_root: &Path,
    policy: &CampaignPolicy,
    operation: CampaignOperation,
    name: &'static str,
) -> Result<(), FuzzCampaignError> {
    let plans = target::registered(repository_root, policy)?
        .into_iter()
        .map(|target| CommandPlan::new(policy, operation, target))
        .collect::<Vec<_>>();
    execution::run(repository_root, name, &plans)?;
    Ok(())
}

fn write_environment(
    policy: &CampaignPolicy,
    profile: CampaignProfile,
    separator: &str,
    output: &mut impl Write,
) -> Result<(), FuzzCampaignError> {
    for (key, value) in policy.environment(profile) {
        writeln!(output, "{key}{separator}{value}").map_err(FuzzCampaignError::Output)?;
    }
    Ok(())
}

fn parse_operation(argument: String) -> Result<Operation, FuzzCampaignError> {
    match argument.as_str() {
        "build" => Ok(Operation::Build),
        "describe" => Ok(Operation::Describe),
        "github-env" => Ok(Operation::GitHubEnvironment),
        "list" => Ok(Operation::List),
        "minimize" => Ok(Operation::Minimize),
        "run" => Ok(Operation::Run),
        _ => Err(FuzzCampaignError::UnknownOperation(argument)),
    }
}

fn parse_profile(
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<CampaignProfile, FuzzCampaignError> {
    let Some(flag) = arguments.next() else {
        return Ok(CampaignProfile::Smoke);
    };
    let flag = into_string(flag)?;
    if flag != "--profile" {
        return Err(FuzzCampaignError::UnexpectedArgument(flag));
    }
    let profile = CampaignProfile::parse(next_argument(arguments)?)?;
    refuse_extra(arguments)?;
    Ok(profile)
}

fn next_argument(
    arguments: &mut impl Iterator<Item = OsString>,
) -> Result<String, FuzzCampaignError> {
    let argument = arguments.next().ok_or(FuzzCampaignError::Usage)?;
    into_string(argument)
}

fn refuse_extra(arguments: &mut impl Iterator<Item = OsString>) -> Result<(), FuzzCampaignError> {
    let Some(extra) = arguments.next() else {
        return Ok(());
    };
    Err(FuzzCampaignError::UnexpectedArgument(into_string(extra)?))
}

fn into_string(argument: OsString) -> Result<String, FuzzCampaignError> {
    argument
        .into_string()
        .map_err(|_| FuzzCampaignError::InvalidArgumentEncoding)
}
