//! Canonical TSV report sequencing.

use std::io::Write;

use crate::{BaselineReport, ReportError};

pub(super) fn write(writer: &mut impl Write, report: &BaselineReport) -> Result<(), ReportError> {
    writeln!(writer, "schema\tkeep.streaming-cas-baseline/v1")?;
    crate::report_tsv_metadata::write(writer, report)?;
    crate::report_tsv_scenario::write(writer, report.scenarios.scenarios())?;
    crate::report_tsv_profile::write(writer, &report.profiles)?;
    writeln!(writer, "threshold-header\tmetric\tstatus\trationale")?;
    writeln!(
        writer,
        "threshold\tall-performance-metrics\tunconfigured\t\
         requires-controlled-baseline-history"
    )?;
    Ok(())
}
