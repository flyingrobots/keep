//! This module owns raw segment and filesystem catalog verification ingress.
#![expect(
    clippy::result_large_err,
    reason = "preserve bounded full refusal coordinates without extra allocations"
)]

use super::verification_failure_class::FailureClass;
use super::{verification_admission, verification_failure_class};
use crate::{
    AdmittedSegment, CatalogRestartError, CatalogRestartPhase, CatalogRestartPolicy,
    FilesystemCatalogSnapshot, SegmentReadPolicy, VerificationDepth, VerificationError,
    VerificationRefusal, VerificationReport, VerificationSource, VerificationSubject,
};
use std::io::ErrorKind;
use std::path::Path;

/// Admits one complete immutable segment and reports its requested evidence.
///
/// Even a framing request performs complete prerequisite admission, including
/// record checksums and identities. Allocation is the bounded duplicate index
/// required by `AdmittedSegment::decode`; work scans the supplied bytes. The
/// operation performs no I/O, mutation, repair or synchronization.
///
/// # Errors
///
/// Returns typed corruption with the original segment cause, operational
/// resource failures, or an unsupported subject/depth request. No failed
/// admission produces a partial report.
pub fn verify_segment(
    encoded: &[u8],
    policy: SegmentReadPolicy,
    requested: VerificationDepth,
) -> Result<VerificationReport, VerificationError> {
    let segment = AdmittedSegment::decode(encoded, policy).map_err(|source| {
        let class = verification_failure_class::segment_class(&source);
        classified(
            VerificationSubject::SegmentInput,
            VerificationSource::Segment(source),
            class,
        )
    })?;
    segment.verify(requested).map_err(Into::into)
}

impl FilesystemCatalogSnapshot {
    /// Loads a read-only, owned catalog view with verification failure classes.
    ///
    /// This uses the same capability-relative, no-follow exact reads as `load`;
    /// segment bytes are retained within `policy`, and catalog bytes/indexes are
    /// bounded by their protocol ceiling. It blocks on filesystem I/O and
    /// performs no write, sync, recovery or repair. This API admits catalog
    /// evidence, not platform durability or retention authority.
    ///
    /// # Errors
    ///
    /// A missing selected head/pool artifact is distinct from demonstrated
    /// malformed bytes and operational I/O/resource failure. The original
    /// `CatalogRestartError` preserves all available phases and coordinates.
    pub fn load_for_verification(
        root: &Path,
        policy: CatalogRestartPolicy,
    ) -> Result<Self, VerificationError> {
        Self::load(root, policy).map_err(catalog_error)
    }

    /// Reports the requested catalog evidence over this exact owned view.
    ///
    /// Re-admits retained bytes through `snapshot` before reporting; its
    /// bounded indexes and decoding cost are included. No filesystem I/O,
    /// mutation or synchronization occurs, and no logical closure is implied.
    ///
    /// # Errors
    ///
    /// Preserves exact admission causes or an unsupported request; a failure
    /// returns no shallower success report.
    pub fn verify(
        &self,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        self.snapshot()
            .map_err(catalog_error)?
            .verify(requested)
            .map_err(Into::into)
    }

    /// Verifies a complete or explicitly shallower blob claim in this owned view.
    ///
    /// Costs include `snapshot` re-admission and `CatalogSnapshot::verify_blob`;
    /// no new whole-blob buffer, filesystem read, write or synchronization occurs.
    ///
    /// # Errors
    ///
    /// Preserves catalog admission and logical verification failures without
    /// returning partial evidence or acquiring writer authority.
    pub fn verify_blob(
        &self,
        blob: crate::BlobId,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        self.snapshot()
            .map_err(catalog_error)?
            .verify_blob(blob, requested)
    }
}

pub(super) fn catalog_error(source: CatalogRestartError) -> VerificationError {
    if let CatalogRestartError::CatalogAdmission { source: nested } = &source
        && let crate::CatalogAdmissionError::MissingSegment { digest } = nested.as_ref()
    {
        return VerificationError::Refused {
            refusal: VerificationRefusal::Missing {
                subject: VerificationSubject::Segment { digest: *digest },
            },
            source: Some(Box::new(VerificationSource::Catalog(source))),
        };
    }

    if matches!(&source, CatalogRestartError::Io { phase: CatalogRestartPhase::OpenHead | CatalogRestartPhase::OpenCatalogDirectory | CatalogRestartPhase::OpenCatalog | CatalogRestartPhase::OpenSegmentDirectory | CatalogRestartPhase::OpenSegment, source } if source.kind() == ErrorKind::NotFound)
    {
        return VerificationError::Refused {
            refusal: VerificationRefusal::Missing {
                subject: VerificationSubject::PublishedCatalog,
            },
            source: Some(Box::new(VerificationSource::Catalog(source))),
        };
    }
    let class = verification_failure_class::catalog_class(&source);
    classified(
        VerificationSubject::PublishedCatalog,
        VerificationSource::Catalog(source),
        class,
    )
}

fn classified(
    subject: VerificationSubject,
    source: VerificationSource,
    class: FailureClass,
) -> VerificationError {
    if matches!(class, FailureClass::Operational) {
        VerificationError::Operational {
            source: Box::new(source),
        }
    } else {
        VerificationError::Refused {
            refusal: verification_admission::structural(subject),
            source: Some(Box::new(source)),
        }
    }
}
