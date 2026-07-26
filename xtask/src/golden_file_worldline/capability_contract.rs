use std::collections::BTreeSet;

use super::canonical_value::{decimal, unique};
use super::corpus_protocol::U16_MAX;
use super::{Corpus, GoldenError};

const CAPABILITY_COLUMNS: [&str; 5] = [
    "capability",
    "posture",
    "first_milestone",
    "owning_issues",
    "claim",
];
const CAPABILITY_CONTRACTS: [CapabilityContract; 16] = [
    CapabilityContract::required("keep.identity.canonical/v1", 1, &[2, 6]),
    CapabilityContract::required("keep.identity.partition-invariant/v1", 1, &[6]),
    CapabilityContract::required("keep.model.exact-immutable-map/v1", 1, &[5]),
    CapabilityContract::required("keep.cdc.profile.canonical/v1", 2, &[7]),
    CapabilityContract::future("keep.content.exact-public-read/v1", 2, &[13]),
    CapabilityContract::future("keep.cdc.nearby-state-reuse/v1", 2, &[7, 8, 13]),
    CapabilityContract::future("keep.ingest.bounded-stream/v1", 2, &[13]),
    CapabilityContract::future("keep.range.minimal-overlap/v1", 2, &[11]),
    CapabilityContract::future("keep.segment.verified-read/v1", 3, &[14, 15]),
    CapabilityContract::future("keep.restart.lawful-recovery/v1", 3, &[17]),
    CapabilityContract::future("keep.retention.both-states/v1", 4, &[18, 19]),
    CapabilityContract::future("keep.verification.precise-refusal/v1", 4, &[20]),
    CapabilityContract::future("keep.compaction.identity-stable/v1", 4, &[21]),
    CapabilityContract::future("keep.echo.identity-agreement/v1", 5, &[22, 23]),
    CapabilityContract::future("keep.graft.golden-worldline/v1", 5, &[24]),
    CapabilityContract::future("keep.git-cas.import/v1", 5, &[25]),
];

pub(super) fn check(corpus: &Corpus) -> Result<(), GoldenError> {
    let rows = corpus.rows(
        "capabilities.tsv",
        "# keep.golden-file-worldline.capabilities/v1",
        &CAPABILITY_COLUMNS,
    )?;
    let mut seen = BTreeSet::new();
    for row in rows {
        let capability = row.field("capability")?;
        if !is_capability_coordinate(capability) {
            return Err(GoldenError::violation(format!(
                "capabilities.tsv: invalid coordinate {capability:?}"
            )));
        }
        unique(capability, &mut seen, "capabilities.tsv")?;
        let milestone = row.field("first_milestone")?;
        let Some(milestone_number) = milestone.strip_prefix('M') else {
            return Err(GoldenError::violation(format!(
                "{capability}: malformed milestone"
            )));
        };
        let observed_milestone = decimal(
            milestone_number,
            &format!("{capability} milestone"),
            U16_MAX,
        )?;
        let observed_issues = issue_numbers(row.field("owning_issues")?, capability)?;
        let expected = CAPABILITY_CONTRACTS
            .iter()
            .find(|contract| contract.capability == capability)
            .ok_or_else(|| {
                GoldenError::violation(format!("{capability}: capability contract moved"))
            })?;
        if row.field("posture")? != expected.posture
            || observed_milestone != expected.milestone
            || observed_issues != expected.issues
        {
            return Err(GoldenError::violation(format!(
                "{capability}: capability contract moved"
            )));
        }
        let claim = row.field("claim")?;
        if claim.is_empty() || claim.trim() != claim {
            return Err(GoldenError::violation(format!(
                "{capability}: claim is empty or noncanonical"
            )));
        }
    }
    if CAPABILITY_CONTRACTS
        .iter()
        .all(|contract| seen.contains(contract.capability))
        && seen.len() == CAPABILITY_CONTRACTS.len()
    {
        Ok(())
    } else {
        Err(GoldenError::violation(
            "capabilities.tsv: required v1 capability set moved",
        ))
    }
}

struct CapabilityContract {
    capability: &'static str,
    posture: &'static str,
    milestone: u64,
    issues: &'static [u64],
}

impl CapabilityContract {
    const fn required(capability: &'static str, milestone: u64, issues: &'static [u64]) -> Self {
        Self {
            capability,
            posture: "required",
            milestone,
            issues,
        }
    }

    const fn future(capability: &'static str, milestone: u64, issues: &'static [u64]) -> Self {
        Self {
            capability,
            posture: "declared-future",
            milestone,
            issues,
        }
    }
}

fn issue_numbers(value: &str, capability: &str) -> Result<Vec<u64>, GoldenError> {
    let issues = value
        .split(',')
        .map(|part| decimal(part, &format!("{capability} issue"), u64::MAX))
        .collect::<Result<Vec<_>, _>>()?;
    let mut canonical = issues.clone();
    canonical.sort_unstable();
    canonical.dedup();
    if issues.is_empty() || issues.contains(&0) || issues != canonical {
        Err(GoldenError::violation(format!(
            "{capability}: owning issues are empty, duplicate, or unordered"
        )))
    } else {
        Ok(issues)
    }
}

fn is_capability_coordinate(value: &str) -> bool {
    let Some((prefix, version)) = value.rsplit_once("/v") else {
        return false;
    };
    if !canonical_positive_decimal(version) {
        return false;
    }
    let mut segments = prefix.split('.');
    if segments.next() != Some("keep") {
        return false;
    }
    let remaining = segments.collect::<Vec<_>>();
    !remaining.is_empty()
        && remaining.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
}

fn canonical_positive_decimal(value: &str) -> bool {
    !value.is_empty()
        && value
            .as_bytes()
            .first()
            .is_some_and(|byte| matches!(byte, b'1'..=b'9'))
        && value.bytes().all(|byte| byte.is_ascii_digit())
}
