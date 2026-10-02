//! This module owns complete, ordered baseline row and decimal admission.

use std::str::Lines;

use super::BenchmarkBaselineError;
use super::report_schema::{
    METADATA_KEYS, PROFILE_HEADER, PROFILE_PREFIXES, SCENARIO_HEADER, SCENARIO_PREFIXES,
};

pub(super) fn admit(report: &str) -> Result<(), BenchmarkBaselineError> {
    let mut lines = report.lines();
    exact(&mut lines, "schema\tkeep.streaming-cas-baseline/v1")?;
    for key in METADATA_KEYS {
        metadata(&mut lines, key)?;
    }
    exact(&mut lines, SCENARIO_HEADER)?;
    for prefix in SCENARIO_PREFIXES {
        metrics(&mut lines, prefix, 28)?;
    }
    exact(&mut lines, PROFILE_HEADER)?;
    for prefix in PROFILE_PREFIXES {
        metrics(&mut lines, prefix, 15)?;
    }
    exact(&mut lines, "threshold-header\tmetric\tstatus\trationale")?;
    exact(
        &mut lines,
        "threshold\tall-performance-metrics\tunconfigured\trequires-controlled-baseline-history",
    )?;
    lines
        .next()
        .map_or(Ok(()), |observed| invalid("end of report", observed))
}

fn exact(lines: &mut Lines<'_>, expected: &'static str) -> Result<(), BenchmarkBaselineError> {
    let observed = lines.next().unwrap_or_default();
    if observed == expected {
        Ok(())
    } else {
        invalid(expected, observed)
    }
}

fn metadata(lines: &mut Lines<'_>, key: &'static str) -> Result<(), BenchmarkBaselineError> {
    let observed = lines.next().unwrap_or_default();
    let mut fields = observed.split('\t');
    if fields.next() != Some("metadata") || fields.next() != Some(key) {
        return invalid(key, observed);
    }
    let value = fields.next().unwrap_or_default();
    if value.is_empty() || value.chars().any(char::is_control) || fields.next().is_some() {
        return invalid("one nonempty metadata value", observed);
    }
    if matches!(key, "logical-cpu-count" | "sample-count" | "warmup-count") {
        decimal(value)?;
    }
    super::metadata_policy::admit(key, value)
}

fn metrics(
    lines: &mut Lines<'_>,
    prefix: &'static str,
    width: usize,
) -> Result<(), BenchmarkBaselineError> {
    let observed = lines.next().unwrap_or_default();
    let Some(values) = observed
        .strip_prefix(prefix)
        .and_then(|tail| tail.strip_prefix('\t'))
    else {
        return invalid(prefix, observed);
    };
    let mut fields = values.split('\t');
    let sample_count = fields.next().unwrap_or_default();
    decimal(sample_count)?;
    super::metadata_policy::require(super::metadata_policy::SAMPLE_COUNT, sample_count)?;
    let mut count = 1_usize;
    for value in fields {
        decimal(value)?;
        count = count
            .checked_add(1)
            .ok_or_else(|| BenchmarkBaselineError::InvalidReportRow {
                expected: "bounded metric count",
                observed: observed.to_owned(),
            })?;
    }
    if count == width {
        Ok(())
    } else {
        invalid("complete metric row", observed)
    }
}

fn decimal(value: &str) -> Result<(), BenchmarkBaselineError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
        || value.parse::<u128>().is_err()
    {
        invalid("canonical unsigned decimal", value)
    } else {
        Ok(())
    }
}

fn invalid<T>(expected: &'static str, observed: &str) -> Result<T, BenchmarkBaselineError> {
    Err(BenchmarkBaselineError::InvalidReportRow {
        expected,
        observed: observed.to_owned(),
    })
}
