//! Complete benchmark evidence collection and build-profile labeling.

use std::io::Write;

use crate::report_profile::{ProfileMetrics, measure_profiles};
use crate::{
    BaselineEnvironment, BaselineMeasurements, BenchmarkCorpus, MeasurementSettings, ReportError,
};

/// Build mode attached to benchmark evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildProfile {
    /// Unoptimized diagnostic evidence that is not a regression baseline.
    DebugDiagnostics,
    /// Optimized evidence admitted as a baseline.
    OptimizedRelease,
}

/// Complete scenario, profile, source, and environment evidence.
pub struct BaselineReport {
    pub(super) environment: BaselineEnvironment,
    pub(super) build_profile: BuildProfile,
    pub(super) settings: MeasurementSettings,
    pub(super) scenarios: BaselineMeasurements,
    pub(super) profiles: Box<[ProfileMetrics]>,
}

impl BuildProfile {
    /// Returns the current compile-time measurement profile.
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(debug_assertions) {
            Self::DebugDiagnostics
        } else {
            Self::OptimizedRelease
        }
    }

    /// Returns the stable report coordinate.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DebugDiagnostics => "debug-diagnostics",
            Self::OptimizedRelease => "optimized-release",
        }
    }

    /// Refuses to publish debug diagnostics as an optimized baseline.
    ///
    /// # Errors
    ///
    /// Returns [`ReportError::DebugBuild`] when debug assertions are enabled.
    pub const fn require_optimized(self) -> Result<(), ReportError> {
        match self {
            Self::DebugDiagnostics => Err(ReportError::DebugBuild),
            Self::OptimizedRelease => Ok(()),
        }
    }
}

impl BaselineReport {
    /// Collects all required scenario and chunk-profile evidence.
    ///
    /// Debug builds remain labeled diagnostic evidence. Call
    /// [`BuildProfile::require_optimized`] before publishing a baseline.
    ///
    /// # Errors
    ///
    /// Returns [`ReportError`] for corpus, measurement, profile, clock,
    /// allocation, determinism, or checked-arithmetic failures.
    pub fn collect(
        environment: BaselineEnvironment,
        settings: MeasurementSettings,
    ) -> Result<Self, ReportError> {
        let corpus = BenchmarkCorpus::generate()?;
        let scenarios = BaselineMeasurements::measure(&corpus, settings)?;
        let profiles = measure_profiles(&corpus, settings)?;
        Ok(Self {
            environment,
            build_profile: BuildProfile::current(),
            settings,
            scenarios,
            profiles,
        })
    }

    /// Writes the canonical tab-separated report schema.
    ///
    /// # Errors
    ///
    /// Returns [`ReportError`] if the destination refuses report bytes.
    pub fn write_tsv(&self, writer: &mut impl Write) -> Result<(), ReportError> {
        crate::report_tsv::write(writer, self)
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
