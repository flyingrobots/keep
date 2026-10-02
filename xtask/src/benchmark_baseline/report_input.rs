//! This module owns byte-size and UTF-8 admission before report parsing.

use std::error::Error;
use std::fmt;
use std::str::Utf8Error;

use super::BenchmarkBaselineError;

pub(super) const MAXIMUM_REPORT_BYTES: usize = 1_048_576;

pub(crate) enum ReportInputError {
    Bound { maximum: usize, observed: usize },
    Encoding { source: Utf8Error },
}

pub(super) fn decode(bytes: &[u8]) -> Result<&str, BenchmarkBaselineError> {
    if bytes.len() > MAXIMUM_REPORT_BYTES {
        return Err(BenchmarkBaselineError::ReportInput(
            ReportInputError::Bound {
                maximum: MAXIMUM_REPORT_BYTES,
                observed: bytes.len(),
            },
        ));
    }
    std::str::from_utf8(bytes).map_err(|source| {
        BenchmarkBaselineError::ReportInput(ReportInputError::Encoding { source })
    })
}

impl fmt::Debug for ReportInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ReportInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bound { maximum, observed } => write!(
                formatter,
                "benchmark report exceeds {maximum} bytes, observed {observed}"
            ),
            Self::Encoding { .. } => formatter.write_str("benchmark report is not UTF-8"),
        }
    }
}

impl Error for ReportInputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encoding { source } => Some(source),
            Self::Bound { .. } => None,
        }
    }
}
