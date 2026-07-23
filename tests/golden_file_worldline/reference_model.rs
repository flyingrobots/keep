//! Capacity-bounded exact-byte reference model.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use keep::{BlobHashError, BlobId};

pub(super) const MODEL_CAPACITY_BYTES: usize = 2_097_152;

pub(super) struct ReferenceModel {
    blobs: BTreeMap<BlobId, Vec<u8>>,
    materialized_bytes: usize,
}

impl ReferenceModel {
    pub(super) const fn new() -> Self {
        Self {
            blobs: BTreeMap::new(),
            materialized_bytes: 0,
        }
    }

    pub(super) fn admit_exact_materialized(
        &mut self,
        bytes: &[u8],
    ) -> Result<BlobId, ReferenceModelError> {
        let identity = BlobId::hash_bytes(bytes).map_err(ReferenceModelError::Hash)?;
        self.admit_claimed_materialized(identity, bytes)?;
        Ok(identity)
    }

    pub(super) fn admit_claimed_materialized(
        &mut self,
        expected: BlobId,
        bytes: &[u8],
    ) -> Result<(), ReferenceModelError> {
        let observed = BlobId::hash_bytes(bytes).map_err(ReferenceModelError::Hash)?;
        if observed != expected {
            return Err(ReferenceModelError::ContentMismatch { expected, observed });
        }
        if let Some(existing) = self.blobs.get(&expected) {
            if existing == bytes {
                return Ok(());
            }
            return Err(ReferenceModelError::IdentityCollision { identity: expected });
        }
        let attempted = self.materialized_bytes.checked_add(bytes.len()).ok_or(
            ReferenceModelError::CapacityExceeded {
                capacity: MODEL_CAPACITY_BYTES,
                attempted: usize::MAX,
            },
        )?;
        if attempted > MODEL_CAPACITY_BYTES {
            return Err(ReferenceModelError::CapacityExceeded {
                capacity: MODEL_CAPACITY_BYTES,
                attempted,
            });
        }
        self.blobs.insert(expected, bytes.to_vec());
        self.materialized_bytes = attempted;
        Ok(())
    }

    pub(super) fn read_exact_materialized(
        &self,
        requested: BlobId,
    ) -> Result<&[u8], ReferenceModelError> {
        self.blobs
            .get(&requested)
            .map(Vec::as_slice)
            .ok_or(ReferenceModelError::Absent { requested })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReferenceModelError {
    Hash(BlobHashError),
    ContentMismatch { expected: BlobId, observed: BlobId },
    IdentityCollision { identity: BlobId },
    Absent { requested: BlobId },
    CapacityExceeded { capacity: usize, attempted: usize },
}

impl fmt::Display for ReferenceModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hash(source) => source.fmt(formatter),
            Self::ContentMismatch { expected, observed } => {
                write!(
                    formatter,
                    "content mismatch: expected {expected}, observed {observed}"
                )
            }
            Self::IdentityCollision { identity } => {
                write!(formatter, "identity collision for {identity}")
            }
            Self::Absent { requested } => write!(formatter, "blob {requested} is absent"),
            Self::CapacityExceeded {
                capacity,
                attempted,
            } => write!(
                formatter,
                "reference model capacity {capacity} exceeded by {attempted} materialized bytes"
            ),
        }
    }
}

impl Error for ReferenceModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Hash(source) => Some(source),
            _ => None,
        }
    }
}
