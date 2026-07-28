//! This module owns typed cargo-fuzz command plans.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::time::Duration;

use super::policy::CampaignPolicy;
use super::profile::CampaignProfile;
use super::target::FuzzTarget;

const CMIN_FAILURE_MARKER: &str = "Failed to minimize corpus:";

#[derive(Clone, Copy)]
pub(super) enum CampaignOperation {
    Build,
    Minimize,
    Run(CampaignProfile),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutputMode {
    Capture,
    Inherit,
}

pub(super) struct CommandPlan {
    target: FuzzTarget,
    arguments: Vec<OsString>,
    deadline: Option<Duration>,
    output_mode: OutputMode,
    refused_output_marker: Option<&'static str>,
}

impl CommandPlan {
    pub(super) fn new(
        policy: &CampaignPolicy,
        operation: CampaignOperation,
        target: FuzzTarget,
    ) -> Self {
        let mut arguments = vec![
            OsString::from(format!("+{}", policy.toolchain())),
            OsString::from("fuzz"),
        ];
        let (deadline, output_mode, refused_output_marker) = match operation {
            CampaignOperation::Build => {
                arguments.extend([OsString::from("build"), OsString::from(target.as_str())]);
                (None, OutputMode::Inherit, None)
            }
            CampaignOperation::Minimize => {
                arguments.extend([OsString::from("cmin"), OsString::from(target.as_str())]);
                push_fuzzer_arguments(&mut arguments, policy.cmin_seconds(), policy);
                (
                    Some(Duration::from_secs(policy.cmin_seconds())),
                    OutputMode::Capture,
                    Some(CMIN_FAILURE_MARKER),
                )
            }
            CampaignOperation::Run(profile) => {
                arguments.extend([OsString::from("run"), OsString::from(target.as_str())]);
                push_fuzzer_arguments(&mut arguments, policy.seconds_per_target(profile), policy);
                (None, OutputMode::Inherit, None)
            }
        };
        Self {
            target,
            arguments,
            deadline,
            output_mode,
            refused_output_marker,
        }
    }

    pub(super) const fn target(&self) -> &FuzzTarget {
        &self.target
    }

    pub(super) fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    pub(super) const fn deadline(&self) -> Option<Duration> {
        self.deadline
    }

    pub(super) const fn output_mode(&self) -> OutputMode {
        self.output_mode
    }

    pub(super) const fn refused_output_marker(&self) -> Option<&'static str> {
        self.refused_output_marker
    }
}

fn push_fuzzer_arguments(arguments: &mut Vec<OsString>, seconds: u64, policy: &CampaignPolicy) {
    arguments.extend([
        OsString::from("--"),
        OsString::from(format!("-max_total_time={seconds}")),
        OsString::from(format!("-timeout={}", policy.input_timeout_seconds())),
        OsString::from(format!("-max_len={}", policy.max_input_bytes())),
        OsString::from(format!("-rss_limit_mb={}", policy.rss_limit_mb())),
        OsString::from("-print_final_stats=1"),
    ]);
}
