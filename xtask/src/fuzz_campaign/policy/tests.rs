use super::{CampaignPolicy, PolicyError};

const POLICY: &str = include_str!("../../../../fuzz/campaign.env");

#[test]
fn repository_policy_preserves_every_reviewed_runtime_bound() -> Result<(), PolicyError> {
    let policy = CampaignPolicy::parse(POLICY)?;
    assert_eq!(policy.cargo_fuzz_version, "0.13.2");
    assert_eq!(policy.toolchain, "nightly-2026-07-24");
    assert_eq!(policy.smoke_seconds_per_target, 15);
    assert_eq!(policy.scheduled_seconds_per_target, 600);
    assert_eq!(policy.cmin_seconds_per_target, 120);
    assert_eq!(policy.input_timeout_seconds, 5);
    assert_eq!(policy.max_input_bytes, 1_048_576);
    assert_eq!(policy.rss_limit_mb, 1_024);
    assert_eq!(policy.corpus_max_files, 20_000);
    assert_eq!(policy.corpus_max_bytes, 536_870_912);
    Ok(())
}

#[test]
fn duplicate_unknown_empty_and_missing_keys_are_exact_refusals() {
    let duplicate = format!("{POLICY}\nCARGO_FUZZ_VERSION=0.13.2\n");
    assert!(matches!(
        CampaignPolicy::parse(&duplicate),
        Err(PolicyError::Key { .. })
    ));
    assert!(matches!(
        CampaignPolicy::parse(&POLICY.replace("CARGO_FUZZ_VERSION", "UNKNOWN")),
        Err(PolicyError::Key { .. })
    ));
    assert!(matches!(
        CampaignPolicy::parse(&POLICY.replace("CARGO_FUZZ_VERSION=0.13.2", "CARGO_FUZZ_VERSION=")),
        Err(PolicyError::Key { .. })
    ));
    assert!(matches!(
        CampaignPolicy::parse(&POLICY.replace("CARGO_FUZZ_VERSION=0.13.2\n", "")),
        Err(PolicyError::Missing(_))
    ));
}

#[test]
fn malformed_and_out_of_bound_values_are_exact_refusals() {
    assert!(matches!(
        CampaignPolicy::parse(&POLICY.replace("FUZZ_RSS_LIMIT_MB=1024", "FUZZ_RSS_LIMIT_MB=huge")),
        Err(PolicyError::InvalidInteger("FUZZ_RSS_LIMIT_MB"))
    ));
    assert!(matches!(
        CampaignPolicy::parse(
            &POLICY.replace("FUZZ_RSS_LIMIT_MB=1024", "FUZZ_RSS_LIMIT_MB=999999")
        ),
        Err(PolicyError::Bound {
            key: "FUZZ_RSS_LIMIT_MB",
            ..
        })
    ));
    assert!(matches!(
        CampaignPolicy::parse(
            &POLICY.replace("CARGO_FUZZ_VERSION=0.13.2", "CARGO_FUZZ_VERSION=latest")
        ),
        Err(PolicyError::InvalidVersion)
    ));
    assert!(matches!(
        CampaignPolicy::parse(&POLICY.replace(
            "FUZZ_TOOLCHAIN=nightly-2026-07-24",
            "FUZZ_TOOLCHAIN=nightly"
        )),
        Err(PolicyError::InvalidToolchain)
    ));
}

#[test]
fn cross_field_bounds_refuse_unsafe_campaign_relationships() {
    let reversed = POLICY
        .replace(
            "FUZZ_SMOKE_SECONDS_PER_TARGET=15",
            "FUZZ_SMOKE_SECONDS_PER_TARGET=60",
        )
        .replace(
            "FUZZ_SCHEDULED_SECONDS_PER_TARGET=600",
            "FUZZ_SCHEDULED_SECONDS_PER_TARGET=60",
        );
    assert!(matches!(
        CampaignPolicy::parse(&reversed),
        Err(PolicyError::CampaignOrder)
    ));
    let undersized = POLICY.replace("FUZZ_CORPUS_MAX_BYTES=536870912", "FUZZ_CORPUS_MAX_BYTES=1");
    assert!(matches!(
        CampaignPolicy::parse(&undersized),
        Err(PolicyError::CorpusCapacity)
    ));
}
