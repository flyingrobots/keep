//! Laws for the machine-readable benchmark evidence artifact.

use std::error::Error;
use std::num::NonZeroUsize;

use crate::{
    BaselineEnvironment, BaselineReport, BuildProfile, HostDescription, MeasurementSettings,
    ReportError, SourceTreeState,
};

const REFERENCE_BASELINE: &str = include_str!("../baselines/c529c07-aarch64-apple-darwin.tsv");

#[test]
fn report_environment_rejects_ambiguous_tsv_fields() -> Result<(), ReportError> {
    let result = BaselineEnvironment::new(
        String::from("0123456789abcdef0123456789abcdef01234567"),
        SourceTreeState::Clean,
        String::from("rustc 1.96.0\nforged"),
        String::from("aarch64-apple-darwin"),
        host()?,
    );
    assert!(matches!(
        result,
        Err(ReportError::InvalidEnvironmentField {
            field: "rustc-version"
        })
    ));
    Ok(())
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
        host()?,
    )?;
    let settings = MeasurementSettings::new(1, 0)?;
    let report = BaselineReport::collect(environment, settings)?;
    let mut output = Vec::new();
    report.write_tsv(&mut output)?;
    let output = String::from_utf8(output)?;

    assert!(output.starts_with("schema\tkeep.streaming-cas-baseline/v1\n"));
    assert!(output.contains("\nmetadata\tbuild-profile\t"));
    assert!(output.contains("\nmetadata\tgit-commit\t"));
    assert!(output.contains("\nmetadata\tos-description\tDarwin 25.3.0 arm64\n"));
    assert!(output.contains("\nmetadata\tcpu-model\tApple M1 Pro\n"));
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

fn host() -> Result<HostDescription, ReportError> {
    HostDescription::new(
        String::from("Darwin 25.3.0 arm64"),
        String::from("Apple M1 Pro"),
        NonZeroUsize::MIN,
    )
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
            line == "metadata\tgit-commit\tc529c07f385b5bcd76a4e57c1987001d496f9135"
        })
    );
    assert_eq!(
        REFERENCE_BASELINE
            .lines()
            .find(|line| line.starts_with("scenario\twarm-ingest\t"))
            .and_then(|line| line.split('\t').nth(5)),
        Some("1048576")
    );
    for metadata in [
        "metadata\tgit-tree\tclean",
        "metadata\ttarget-triple\taarch64-apple-darwin",
        "metadata\tos-description\tDarwin 25.3.0 arm64",
        "metadata\tcpu-model\tApple M1 Pro",
        "metadata\tlogical-cpu-count\t10",
    ] {
        assert!(REFERENCE_BASELINE.lines().any(|line| line == metadata));
    }
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
