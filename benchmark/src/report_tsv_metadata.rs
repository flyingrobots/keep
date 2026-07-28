//! Build, source, environment, and measurement-semantics metadata rows.

use std::io::{self, Write};

use crate::BaselineReport;

pub(super) fn write(writer: &mut impl Write, report: &BaselineReport) -> Result<(), io::Error> {
    let environment = &report.environment;
    for (key, value) in [
        ("build-profile", report.build_profile.name()),
        ("git-commit", &environment.git_commit),
        ("git-tree", environment.source_tree.name()),
        ("rustc-version", &environment.rustc_version),
        ("target-triple", &environment.target_triple),
        ("cpu-clock", "process"),
        ("peak-memory", "incremental-live-heap"),
        ("verification", "mandatory"),
        ("timing-unit", "nanoseconds"),
        ("byte-unit", "bytes"),
        ("ratio-encoding", "exact-numerator-denominator"),
    ] {
        writeln!(writer, "metadata\t{key}\t{value}")?;
    }
    writeln!(
        writer,
        "metadata\tlogical-cpu-count\t{}",
        environment.logical_cpu_count
    )?;
    writeln!(
        writer,
        "metadata\tsample-count\t{}",
        report.settings.sample_count
    )?;
    writeln!(
        writer,
        "metadata\twarmup-count\t{}",
        report.settings.warmup_count
    )
}
