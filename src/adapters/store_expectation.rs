//! This module owns the state a caller expects before trusting an opened store.

use std::error::Error;
use std::fmt;

use crate::{CatalogDigest, CatalogGeneration};

/// The published catalog a caller recorded outside the store.
///
/// This is the caller's claim, not a verified digest: Keep only compares it
/// with what an opened store publishes. A publication receipt or a pinned view
/// supplies both values to [`Self::new`]. To carry the expectation into a later
/// process, record [`Self::generation`] as its `u64` and [`Self::digest_bytes`],
/// then rebuild it with [`Self::from_recorded`]. Keep stores neither value for
/// the caller; where they are kept, and how that record is protected, is the
/// caller's responsibility.
///
/// ```
/// use keep::{CatalogGeneration, ExpectedCatalog, StoreExpectation};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Recorded by an earlier process, from a publication receipt or a view.
/// let recorded_generation: u64 = 7;
/// let recorded_digest: [u8; 32] = [0x5a; 32];
///
/// let expected = ExpectedCatalog::from_recorded(
///     CatalogGeneration::new(recorded_generation)?,
///     recorded_digest,
/// );
/// let expectation = StoreExpectation::Catalog(expected);
/// assert_eq!(expected.generation().get(), recorded_generation);
/// assert_eq!(expected.digest_bytes(), recorded_digest);
/// # let _ = expectation;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedCatalog {
    generation: CatalogGeneration,
    digest: [u8; 32],
}

impl ExpectedCatalog {
    /// Names the exact catalog a receipt or view reported in this process.
    #[must_use]
    pub const fn new(generation: CatalogGeneration, digest: CatalogDigest) -> Self {
        Self {
            generation,
            digest: *digest.as_bytes(),
        }
    }

    /// Rebuilds an expectation from the two values a caller recorded earlier.
    ///
    /// Nothing is verified here. A wrong record refuses the right store, and a
    /// record an attacker can rewrite protects nothing.
    #[must_use]
    pub const fn from_recorded(generation: CatalogGeneration, digest: [u8; 32]) -> Self {
        Self { generation, digest }
    }

    /// The expected catalog generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// The expected catalog digest, as the exact bytes to record.
    #[must_use]
    pub const fn digest_bytes(self) -> [u8; 32] {
        self.digest
    }

    fn names(self, observed: ObservedCatalog) -> bool {
        self.generation == observed.generation && self.digest == *observed.digest.as_bytes()
    }
}

/// The catalog an opened store's checksummed `HEAD` names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedCatalog {
    generation: CatalogGeneration,
    digest: CatalogDigest,
}

impl ObservedCatalog {
    pub(super) const fn new(generation: CatalogGeneration, digest: CatalogDigest) -> Self {
        Self { generation, digest }
    }

    /// The published catalog generation.
    pub const fn generation(self) -> CatalogGeneration {
        self.generation
    }

    /// The published catalog digest, as `HEAD` names it.
    ///
    /// The comparison reads `HEAD` only, so the catalog this names may not
    /// have been opened or verified when a mismatch is reported.
    pub const fn digest(self) -> CatalogDigest {
        self.digest
    }
}

/// What a caller knows about the store it is opening.
///
/// A store's own records cannot show that it is the store the caller meant, or
/// that newer history existed, because a restored store carries consistent old
/// records. Only state kept outside the store can. Every normal open therefore
/// requires this choice, and it has no default: [`FilesystemStoreAdmission::reopen`],
/// [`FilesystemCatalogSnapshot::load`], [`FilesystemRetentionSnapshot::load`],
/// [`DurableSnapshot::open`], and [`DurableStore::open`]. Recovery tools,
/// verification loads, and initialization take none, so a caller of those routes
/// does not get this check.
///
/// # Keeping an expectation current
///
/// An expectation names one catalog, so it must follow every publication: after
/// a receipt, record the receipt's generation and digest before relying on the
/// old ones again. Publication and that record are two steps, and a process can
/// die between them. The store then holds the successor while the caller still
/// holds its predecessor, and every open refuses with
/// [`StoreExpectationMismatch`]. Keep cannot tell that successor from a
/// different store through this interface. Accepting what
/// [`StoreExpectationMismatch::observed`] reports is the caller's own trust
/// decision, made with evidence from outside the store; see "Recovering a stale
/// expectation" in `docs/formats/store/recovery.md`.
///
/// ```no_run
/// # #[cfg(any(target_os = "linux", target_os = "macos"))]
/// # fn later_open(
/// #     root: &std::path::Path,
/// #     policy: keep::CatalogRestartPolicy,
/// #     recorded: (u64, [u8; 32]),
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// use keep::{
///     CatalogGeneration, DurableSnapshot, DurableStoreError, ExpectedCatalog,
///     FilesystemRetentionSnapshotError, ReaderAttemptLimit, StoreExpectation,
/// };
///
/// let expected = ExpectedCatalog::from_recorded(CatalogGeneration::new(recorded.0)?, recorded.1);
/// let expectation = StoreExpectation::Catalog(expected);
/// match DurableSnapshot::open(root, policy, ReaderAttemptLimit::DEFAULT, expectation) {
///     // The store holds exactly the recorded catalog.
///     Ok(snapshot) => assert_eq!(snapshot.view().catalog_generation(), expected.generation()),
///     Err(DurableStoreError::Snapshot(error)) => match *error {
///         // Another catalog, or none. `source.observed()` reports which; whether
///         // to trust and record it is decided outside Keep.
///         FilesystemRetentionSnapshotError::Expectation { source } => return Err(source.into()),
///         other => return Err(other.into()),
///     },
///     Err(other) => return Err(other.into()),
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Limits
///
/// Two stores that publish the same catalog bytes are indistinguishable, and a
/// store expected to have published nothing has no representation here.
///
/// [`FilesystemStoreAdmission::reopen`]: crate::FilesystemStoreAdmission::reopen
/// [`FilesystemCatalogSnapshot::load`]: crate::FilesystemCatalogSnapshot::load
/// [`FilesystemRetentionSnapshot::load`]: crate::FilesystemRetentionSnapshot::load
/// [`DurableSnapshot::open`]: crate::DurableSnapshot::open
/// [`DurableStore::open`]: crate::DurableStore::open
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreExpectation {
    /// Refuse unless the store's published catalog is exactly this one.
    ///
    /// This does not detect a rollback that leaves the catalog unchanged, such as
    /// restoring only older retention state.
    Catalog(ExpectedCatalog),
    /// The caller has no expectation and accepts any valid store at this path,
    /// including a different store or an older state of the intended one.
    ///
    /// This is the honest choice for a first open, before anything is recorded.
    /// A store that has published no catalog still refuses on the read path,
    /// which has nothing to read.
    NoExpectation,
}

impl StoreExpectation {
    /// Compares the observed published catalog, `None` meaning none is published.
    pub(super) fn require(
        self,
        observed: Option<ObservedCatalog>,
    ) -> Result<(), StoreExpectationMismatch> {
        match (self, observed) {
            (Self::NoExpectation, _) => Ok(()),
            (Self::Catalog(expected), Some(found)) if expected.names(found) => Ok(()),
            (Self::Catalog(expected), observed) => {
                Err(StoreExpectationMismatch { expected, observed })
            }
        }
    }
}

/// An opened store does not hold the catalog its caller expected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreExpectationMismatch {
    expected: ExpectedCatalog,
    observed: Option<ObservedCatalog>,
}

impl StoreExpectationMismatch {
    /// The catalog the caller expected.
    #[must_use]
    pub const fn expected(self) -> ExpectedCatalog {
        self.expected
    }

    /// The catalog the store has published, or `None` when it has published none.
    #[must_use]
    pub const fn observed(self) -> Option<ObservedCatalog> {
        self.observed
    }
}

impl fmt::Display for StoreExpectationMismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expected published catalog generation {} digest ",
            self.expected.generation.get()
        )?;
        write_hex(formatter, &self.expected.digest)?;
        match self.observed {
            None => formatter.write_str(", but the store has published no catalog"),
            Some(observed) => {
                write!(
                    formatter,
                    ", but the store holds generation {} digest ",
                    observed.generation.get()
                )?;
                write_hex(formatter, observed.digest.as_bytes())
            }
        }
    }
}

fn write_hex(formatter: &mut fmt::Formatter<'_>, bytes: &[u8; 32]) -> fmt::Result {
    bytes
        .iter()
        .try_for_each(|byte| write!(formatter, "{byte:02x}"))
}

impl Error for StoreExpectationMismatch {}
