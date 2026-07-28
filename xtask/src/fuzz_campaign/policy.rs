//! This module owns strict admission of the checked-in fuzz campaign policy.

mod error;
mod syntax;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

use super::profile::CampaignProfile;
pub(crate) use error::PolicyError;
use syntax::{bounded, is_dated_nightly, is_exact_version, parse_assignments, value};

pub(super) struct CampaignPolicy {
    cargo_fuzz_version: String,
    toolchain: String,
    build_timeout_seconds: u64,
    input_timeout_seconds: u64,
    max_input_bytes: u64,
    process_grace_seconds: u64,
    rss_limit_mb: u64,
    smoke_seconds_per_target: u64,
    scheduled_seconds_per_target: u64,
    cmin_seconds_per_target: u64,
    smoke_failure_retention_days: u64,
    scheduled_failure_retention_days: u64,
    corpus_retention_days: u64,
    corpus_max_files: u64,
    corpus_max_bytes: u64,
}

impl CampaignPolicy {
    pub(super) fn load(repository_root: &Path) -> Result<Self, PolicyError> {
        let path = repository_root.join("fuzz/campaign.env");
        let raw = fs::read_to_string(&path).map_err(|source| PolicyError::Read { path, source })?;
        Self::parse(&raw)
    }

    fn parse(raw: &str) -> Result<Self, PolicyError> {
        let values = parse_assignments(raw)?;
        let cargo_fuzz_version = value(&values, "CARGO_FUZZ_VERSION")?.to_owned();
        let toolchain = value(&values, "FUZZ_TOOLCHAIN")?.to_owned();
        if !is_exact_version(&cargo_fuzz_version) {
            return Err(PolicyError::InvalidVersion);
        }
        if !is_dated_nightly(&toolchain) {
            return Err(PolicyError::InvalidToolchain);
        }
        let policy = Self::from_values(&values, cargo_fuzz_version, toolchain)?;
        policy.validate_relationships()?;
        Ok(policy)
    }

    pub(super) fn environment(&self, profile: CampaignProfile) -> BTreeMap<&'static str, String> {
        [
            ("CARGO_FUZZ_VERSION", self.cargo_fuzz_version.clone()),
            (
                "FUZZ_BUILD_TIMEOUT_SECONDS",
                self.build_timeout_seconds.to_string(),
            ),
            (
                "FUZZ_CMIN_SECONDS_PER_TARGET",
                self.cmin_seconds_per_target.to_string(),
            ),
            ("FUZZ_CORPUS_MAX_BYTES", self.corpus_max_bytes.to_string()),
            ("FUZZ_CORPUS_MAX_FILES", self.corpus_max_files.to_string()),
            (
                "FUZZ_CORPUS_RETENTION_DAYS",
                self.corpus_retention_days.to_string(),
            ),
            (
                "FUZZ_INPUT_TIMEOUT_SECONDS",
                self.input_timeout_seconds.to_string(),
            ),
            ("FUZZ_MAX_INPUT_BYTES", self.max_input_bytes.to_string()),
            (
                "FUZZ_PROCESS_GRACE_SECONDS",
                self.process_grace_seconds.to_string(),
            ),
            ("FUZZ_RSS_LIMIT_MB", self.rss_limit_mb.to_string()),
            (
                "FUZZ_SCHEDULED_FAILURE_RETENTION_DAYS",
                self.scheduled_failure_retention_days.to_string(),
            ),
            (
                "FUZZ_SECONDS_PER_TARGET",
                self.seconds_per_target(profile).to_string(),
            ),
            (
                "FUZZ_SMOKE_FAILURE_RETENTION_DAYS",
                self.smoke_failure_retention_days.to_string(),
            ),
            ("FUZZ_TOOLCHAIN", self.toolchain.clone()),
        ]
        .into_iter()
        .collect()
    }

    pub(super) fn toolchain(&self) -> &str {
        &self.toolchain
    }

    pub(super) const fn cmin_seconds(&self) -> u64 {
        self.cmin_seconds_per_target
    }

    pub(super) const fn build_timeout(&self) -> Duration {
        Duration::from_secs(self.build_timeout_seconds)
    }

    pub(super) fn run_timeout(&self, profile: CampaignProfile) -> Result<Duration, PolicyError> {
        self.seconds_per_target(profile)
            .checked_add(self.process_grace_seconds)
            .map(Duration::from_secs)
            .ok_or(PolicyError::CampaignDeadline)
    }

    pub(super) const fn corpus_max_bytes(&self) -> u64 {
        self.corpus_max_bytes
    }

    pub(super) const fn corpus_max_files(&self) -> u64 {
        self.corpus_max_files
    }

    pub(super) const fn input_timeout_seconds(&self) -> u64 {
        self.input_timeout_seconds
    }

    pub(super) const fn max_input_bytes(&self) -> u64 {
        self.max_input_bytes
    }

    pub(super) const fn rss_limit_mb(&self) -> u64 {
        self.rss_limit_mb
    }

    fn from_values(
        values: &BTreeMap<&str, &str>,
        cargo_fuzz_version: String,
        toolchain: String,
    ) -> Result<Self, PolicyError> {
        Ok(Self {
            cargo_fuzz_version,
            toolchain,
            build_timeout_seconds: bounded(values, "FUZZ_BUILD_TIMEOUT_SECONDS", 60, 3_600)?,
            input_timeout_seconds: bounded(values, "FUZZ_INPUT_TIMEOUT_SECONDS", 1, 60)?,
            max_input_bytes: bounded(values, "FUZZ_MAX_INPUT_BYTES", 1, 1_048_576)?,
            process_grace_seconds: bounded(values, "FUZZ_PROCESS_GRACE_SECONDS", 1, 600)?,
            rss_limit_mb: bounded(values, "FUZZ_RSS_LIMIT_MB", 128, 8_192)?,
            smoke_seconds_per_target: bounded(values, "FUZZ_SMOKE_SECONDS_PER_TARGET", 1, 60)?,
            scheduled_seconds_per_target: bounded(
                values,
                "FUZZ_SCHEDULED_SECONDS_PER_TARGET",
                60,
                3_600,
            )?,
            cmin_seconds_per_target: bounded(values, "FUZZ_CMIN_SECONDS_PER_TARGET", 1, 600)?,
            smoke_failure_retention_days: bounded(
                values,
                "FUZZ_SMOKE_FAILURE_RETENTION_DAYS",
                1,
                90,
            )?,
            scheduled_failure_retention_days: bounded(
                values,
                "FUZZ_SCHEDULED_FAILURE_RETENTION_DAYS",
                1,
                90,
            )?,
            corpus_retention_days: bounded(values, "FUZZ_CORPUS_RETENTION_DAYS", 1, 90)?,
            corpus_max_files: bounded(values, "FUZZ_CORPUS_MAX_FILES", 1, 100_000)?,
            corpus_max_bytes: bounded(values, "FUZZ_CORPUS_MAX_BYTES", 1, 1_073_741_824)?,
        })
    }

    const fn validate_relationships(&self) -> Result<(), PolicyError> {
        if self.scheduled_seconds_per_target <= self.smoke_seconds_per_target {
            return Err(PolicyError::CampaignOrder);
        }
        if self.corpus_max_bytes < self.max_input_bytes {
            return Err(PolicyError::CorpusCapacity);
        }
        Ok(())
    }

    pub(super) const fn seconds_per_target(&self, profile: CampaignProfile) -> u64 {
        match profile {
            CampaignProfile::Scheduled => self.scheduled_seconds_per_target,
            CampaignProfile::Smoke => self.smoke_seconds_per_target,
        }
    }
}
