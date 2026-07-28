//! Laws for the machine-readable benchmark evidence artifact.

use std::error::Error;
use std::num::NonZeroUsize;

use crate::{
    BaselineEnvironment, BaselineReport, BuildProfile, MeasurementSettings, ReportError,
    SourceTreeState,
};

const REFERENCE_BASELINE: &str = include_str!("../baselines/159863e-aarch64-apple-darwin.tsv");

#[test]
fn report_environment_rejects_ambiguous_tsv_fields() {
    let result = BaselineEnvironment::new(
        String::from("0123456789abcdef0123456789abcdef01234567"),
        SourceTreeState::Clean,
        String::from("rustc 1.96.0\nforged"),
        String::from("aarch64-apple-darwin"),
        NonZeroUsize::MIN,
    );
    assert!(matches!(
        result,
        Err(ReportError::InvalidEnvironmentField {
            field: "rustc-version"
        })
    ));
}

#[test]
fn build_profile_never_mislabels_debug_diagnostics_as_optimized() {
    let profile = BuildProfile::current();
    if cfg!(debug_assertions) {
        assert_eq!(profile.name(), "debug-diagnostics");
        assert!(matches!(
            profile.require_optimized(),
            Err(ReportError::DebugBuild)
        ));
    } else {
        assert_eq!(profile.name(), "optimized-release");
        assert!(profile.require_optimized().is_ok());
    }
}

#[test]
fn tsv_report_names_every_metric_profile_and_unset_threshold() -> Result<(), Box<dyn Error>> {
    let environment = BaselineEnvironment::new(
        String::from("0123456789abcdef0123456789abcdef01234567"),
        SourceTreeState::Clean,
        String::from("rustc 1.96.0"),
        String::from("aarch64-apple-darwin"),
        NonZeroUsize::MIN,
    )?;
    let settings = MeasurementSettings::new(1, 0)?;
    let report = BaselineReport::collect(environment, settings)?;
    let mut output = Vec::new();
    report.write_tsv(&mut output)?;
    let output = String::from_utf8(output)?;

    assert!(output.starts_with("schema\tkeep.streaming-cas-baseline/v1\n"));
    assert!(output.contains("\nmetadata\tbuild-profile\t"));
    assert!(output.contains("\nmetadata\tgit-commit\t"));
    assert!(output.contains("\nmetadata\tpeak-memory\tincremental-live-heap\n"));
    assert!(output.contains("\nscenario-header\tname\tverification\t"));
    assert!(output.contains("\tread-amplification-numerator\t"));
    assert!(output.contains("\twrite-amplification-denominator\t"));
    assert!(output.contains("\tdeduplication-ratio-numerator\t"));
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("scenario\t"))
            .count(),
        13
    );
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("profile\t"))
            .count(),
        5
    );
    assert!(output.contains(
        "\nthreshold\tall-performance-metrics\tunconfigured\t\
         requires-controlled-baseline-history\n"
    ));
    Ok(())
}

#[test]
fn committed_reference_baseline_is_field_complete_and_source_bound() {
    let scenario_width = REFERENCE_BASELINE
        .lines()
        .find(|line| line.starts_with("scenario-header\t"))
        .map(|line| line.split('\t').count());
    let profile_width = REFERENCE_BASELINE
        .lines()
        .find(|line| line.starts_with("profile-header\t"))
        .map(|line| line.split('\t').count());

    assert_eq!(
        REFERENCE_BASELINE
            .lines()
            .filter(|line| line.starts_with("scenario\t"))
            .count(),
        13
    );
    assert_eq!(
        REFERENCE_BASELINE
            .lines()
            .filter(|line| line.starts_with("profile\t"))
            .count(),
        5
    );
    assert!(
        REFERENCE_BASELINE.lines().any(|line| {
            line == "metadata\tgit-commit\t159863e21f7de8b5ba76902f44568e545701ade9"
        })
    );
    assert!(REFERENCE_BASELINE.lines().any(|line| {
        line == "threshold\tall-performance-metrics\tunconfigured\t\
                 requires-controlled-baseline-history"
    }));
    assert!(REFERENCE_BASELINE.lines().all(|line| {
        if line.starts_with("scenario\t") {
            Some(line.split('\t').count()) == scenario_width
        } else if line.starts_with("profile\t") {
            Some(line.split('\t').count()) == profile_width
        } else {
            true
        }
    }));
}
