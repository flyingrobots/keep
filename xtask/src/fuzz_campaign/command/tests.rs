use std::error::Error;
use std::path::Path;
use std::time::Duration;

use super::{CampaignOperation, CommandPlan};
use crate::fuzz_campaign::policy::CampaignPolicy;
use crate::fuzz_campaign::profile::CampaignProfile;
use crate::fuzz_campaign::target::FuzzTarget;

#[test]
fn run_plan_preserves_every_reviewed_resource_bound() -> Result<(), Box<dyn Error>> {
    let policy = policy()?;
    let target = FuzzTarget::admit("segment_format".to_owned())?;
    let plan = CommandPlan::new(
        &policy,
        CampaignOperation::Run(CampaignProfile::Smoke),
        target,
    );
    assert_eq!(
        plan.arguments(),
        [
            "+nightly-2026-07-24",
            "fuzz",
            "run",
            "segment_format",
            "--",
            "-max_total_time=15",
            "-timeout=5",
            "-max_len=1048576",
            "-rss_limit_mb=1024",
            "-print_final_stats=1",
        ]
    );
    assert_eq!(plan.deadline(), None);
    Ok(())
}

#[test]
fn minimization_plan_has_an_external_deadline_and_failure_marker() -> Result<(), Box<dyn Error>> {
    let policy = policy()?;
    let target = FuzzTarget::admit("blob_hasher".to_owned())?;
    let plan = CommandPlan::new(&policy, CampaignOperation::Minimize, target);
    assert_eq!(plan.deadline(), Some(Duration::from_mins(2)));
    assert_eq!(
        plan.refused_output_marker(),
        Some("Failed to minimize corpus:")
    );
    Ok(())
}

fn policy() -> Result<CampaignPolicy, Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    Ok(CampaignPolicy::load(root)?)
}
