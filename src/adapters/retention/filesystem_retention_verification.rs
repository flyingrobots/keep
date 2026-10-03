//! This module owns verification of one publication-selected retained namespace.
#![expect(
    clippy::result_large_err,
    reason = "preserve bounded full refusal coordinates without extra allocations"
)]

use super::{AdmittedRetentionRoot, RetentionSelectedRootRefusal};
use crate::adapters::filesystem_exact_record::ExactRecordError;
use crate::adapters::{verification_admission, verification_ingress};
use crate::{
    CatalogRestartPolicy, FilesystemRetentionSnapshot, FilesystemRetentionSnapshotError,
    ReaderAttemptLimit, RetentionNamespaceDigest, RetentionRootDecodeError, VerificationDepth,
    VerificationError, VerificationRefusal, VerificationReport, VerificationSource,
    VerificationSubject,
};
use std::io;

impl FilesystemRetentionSnapshot {
    /// Loads an existing fenced retention view with verification classifications.
    ///
    /// Performs the same bounded filesystem observations and fence acquisition
    /// as `load`; it never publishes, repairs or runs recovery. Catalog bytes
    /// and segments are retained under `policy`; the shared fence is held for
    /// this owner's lifetime. Observation failures retain their original causes.
    ///
    /// # Errors
    ///
    /// Missing catalog evidence is distinct from admitted corruption and an
    /// inconclusive I/O, resource or fence failure. Exhausted moving-view
    /// collection returns ambiguity with the last exact conflicting pair.
    pub fn load_for_verification(
        root: &std::path::Path,
        policy: CatalogRestartPolicy,
        limit: ReaderAttemptLimit,
    ) -> Result<Self, VerificationError> {
        Self::load_with(root, policy, limit, |source, limit| {
            super::collect_verification_view(source, limit)?
                .map_err(verification_ingress::catalog_error)
        })
    }

    /// Verifies the exact root selected for `namespace` in this fenced view.
    ///
    /// Reads only the manifest-selected root, without following links, within
    /// the root format bound. It checks namespace, generation and digest, then
    /// re-admits the owned catalog and applies `AdmittedRetentionRoot::verify`.
    /// Costs include one bounded root buffer, its decoded anchors and catalog
    /// indexes; closure adds its root-bounded member index and one layout.
    /// This synchronous operation performs reads but no writes, sync, repair,
    /// recovery or publication; the existing reader fence remains held.
    ///
    /// # Errors
    ///
    /// Returns exact missing namespace/member, unsupported depth, corruption,
    /// or operational failure with original typed causes. No partial report
    /// is returned and no alternate root or catalog is substituted.
    pub fn verify_retention(
        &self,
        namespace: RetentionNamespaceDigest,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        let subject = VerificationSubject::RetentionNamespace { namespace };
        let bytes = self
            .retained_root(namespace)
            .map_err(|source| root_error(subject, source))?
            .ok_or(VerificationRefusal::Missing { subject })?;
        let root = AdmittedRetentionRoot::decode(&bytes)
            .map_err(|source| decode_error(subject, source))?;
        if root.root().namespace().digest() != namespace {
            let source = io::Error::new(
                io::ErrorKind::InvalidData,
                RetentionSelectedRootRefusal::Namespace {
                    expected: namespace,
                    observed: root.root().namespace().digest(),
                },
            );
            return Err(root_error(
                subject,
                FilesystemRetentionSnapshotError::Root { source },
            ));
        }
        let catalog = self
            .catalog()
            .snapshot()
            .map_err(verification_ingress::catalog_error)?;
        root.verify(&catalog, requested)
            .map(|report| report.in_retention(self.retention_head().copied()))
    }
}

fn decode_error(
    subject: VerificationSubject,
    source: RetentionRootDecodeError,
) -> VerificationError {
    if matches!(source, RetentionRootDecodeError::Allocation { .. }) {
        return VerificationError::Operational {
            source: Box::new(VerificationSource::Root(source)),
        };
    }
    VerificationError::Refused {
        refusal: verification_admission::structural(subject),
        source: Some(Box::new(VerificationSource::Root(source))),
    }
}

fn view_error(source: FilesystemRetentionSnapshotError) -> VerificationError {
    if let FilesystemRetentionSnapshotError::Catalog { source } = source {
        return verification_ingress::catalog_error(source);
    }
    VerificationError::Operational {
        source: Box::new(VerificationSource::Retention(source)),
    }
}

fn root_error(
    subject: VerificationSubject,
    error: FilesystemRetentionSnapshotError,
) -> VerificationError {
    let FilesystemRetentionSnapshotError::Root { source } = &error else {
        return view_error(error);
    };
    let cause = source.get_ref();
    let selected = cause.and_then(|cause| cause.downcast_ref::<RetentionSelectedRootRefusal>());
    if source.kind() == io::ErrorKind::NotFound
        || matches!(selected, Some(RetentionSelectedRootRefusal::Absent))
    {
        return VerificationError::Refused {
            refusal: VerificationRefusal::Missing { subject },
            source: Some(Box::new(VerificationSource::Retention(error))),
        };
    }
    let root = cause.and_then(|cause| cause.downcast_ref::<RetentionRootDecodeError>());
    let exact = cause.and_then(|cause| cause.downcast_ref::<ExactRecordError>());
    let corrupt = matches!(
        selected,
        Some(
            RetentionSelectedRootRefusal::Length { .. }
                | RetentionSelectedRootRefusal::Coordinate { .. }
                | RetentionSelectedRootRefusal::Namespace { .. }
        )
    ) || root
        .is_some_and(|source| !matches!(source, RetentionRootDecodeError::Allocation { .. }))
        || matches!(exact, Some(ExactRecordError::Refused(_)));
    if corrupt {
        VerificationError::Refused {
            refusal: verification_admission::structural(subject),
            source: Some(Box::new(VerificationSource::Retention(error))),
        }
    } else {
        VerificationError::Operational {
            source: Box::new(VerificationSource::Retention(error)),
        }
    }
}

impl From<FilesystemRetentionSnapshotError> for VerificationError {
    fn from(source: FilesystemRetentionSnapshotError) -> Self {
        view_error(source)
    }
}
