//! This module owns precise failures of benchmark metric relationships.

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use crate::diagnostic::escaped_controls;

pub(crate) enum ReportMetricError {
    Mismatch {
        metric: &'static str,
        expected: u128,
        observed: u128,
    },
    Bound {
        metric: &'static str,
        maximum: u128,
        observed: u128,
    },
    Arithmetic {
        bytes: u128,
        samples: u128,
        duration: u128,
    },
    Encoding {
        observed: String,
        source: ParseIntError,
    },
}

impl fmt::Debug for ReportMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ReportMetricError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mismatch {
                metric,
                expected,
                observed,
            } => write!(
                formatter,
                "benchmark metric `{metric}` expected {expected}, observed {observed}"
            ),
            Self::Bound {
                metric,
                maximum,
                observed,
            } => write!(
                formatter,
                "benchmark metric `{metric}` exceeds {maximum}, observed {observed}"
            ),
            Self::Arithmetic {
                bytes,
                samples,
                duration,
            } => write!(
                formatter,
                "benchmark throughput arithmetic refused: bytes {bytes}, samples {samples}, duration {duration}"
            ),
            Self::Encoding { observed, .. } => {
                formatter.write_str("benchmark metric cannot decode `")?;
                escaped_controls(formatter, observed)?;
                formatter.write_str("`")
            }
        }
    }
}

impl Error for ReportMetricError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encoding { source, .. } => Some(source),
            Self::Mismatch { .. } | Self::Bound { .. } | Self::Arithmetic { .. } => None,
        }
    }
}
