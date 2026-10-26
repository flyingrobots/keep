//! This boundary module owns copying one blob between stores without
//! buffering it.

use std::error::Error;
use std::fmt;

use super::{TransferSource, TransferSourceError};
use crate::{BlobId, CommitReceipt, ContentStaging, LayoutId, StagedContent, StagingLimits};

/// What one copy established: the identity the destination committed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the receipt records the identity the destination committed"]
pub struct CopyReceipt {
    target: BlobId,
    layout_id: LayoutId,
}

impl CopyReceipt {
    /// The blob the destination committed; equal to the source's.
    #[must_use]
    pub const fn target(self) -> BlobId {
        self.target
    }

    /// The layout the destination committed under; the destination's own
    /// chunking decides it.
    #[must_use]
    pub const fn layout_id(self) -> LayoutId {
        self.layout_id
    }
}

/// Copies the source's committed layout `layout_id` into `destination`.
///
/// The source streams its chunks, authenticating each as it is served;
/// the destination stages the stream under `limits` with
/// `stage_expected`, so the complete identity is verified before anything
/// becomes visible, and commits. No more than the destination's own
/// staging scratch holds the blob at any point.
///
/// # Errors
///
/// Returns [`CopyError`] at the exact source, staging, or commit refusal;
/// nothing becomes visible on failure.
pub fn copy_layout<S, D>(
    source: &S,
    layout_id: LayoutId,
    destination: &mut D,
    limits: StagingLimits,
) -> Result<CopyReceipt, CopyError>
where
    S: TransferSource + ?Sized,
    D: ContentStaging,
{
    let mut outcome = None;
    source
        .stream_layout(layout_id, &mut |target, reader| {
            outcome = Some(stage_and_commit(destination, reader, target, limits));
        })
        .map_err(|source| CopyError::Source(Box::new(source)))?;
    outcome.unwrap_or(Err(CopyError::NotStreamed))
}

fn stage_and_commit<D: ContentStaging>(
    destination: &mut D,
    reader: &mut dyn std::io::Read,
    target: BlobId,
    limits: StagingLimits,
) -> Result<CopyReceipt, CopyError> {
    let staged = destination
        .stage_expected(reader, target, limits)
        .map_err(|source| CopyError::Stage(Box::new(source)))?;
    let receipt = staged
        .commit()
        .map_err(|source| CopyError::Commit(Box::new(source)))?;
    Ok(CopyReceipt {
        target: receipt.target(),
        layout_id: receipt.layout_id(),
    })
}

/// Why a copy committed nothing.
#[derive(Debug)]
pub enum CopyError {
    /// The source refused before or while streaming.
    Source(Box<TransferSourceError>),
    /// The destination refused the staging.
    Stage(Box<dyn Error + 'static>),
    /// The destination refused the commit.
    Commit(Box<dyn Error + 'static>),
    /// The source returned without calling the consumer.
    NotStreamed,
}

impl fmt::Display for CopyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(source) => write!(formatter, "source: {source}"),
            Self::Stage(source) => write!(formatter, "destination staging: {source}"),
            Self::Commit(source) => write!(formatter, "destination commit: {source}"),
            Self::NotStreamed => formatter.write_str("the source streamed nothing"),
        }
    }
}

impl Error for CopyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Source(source) => Some(source.as_ref()),
            Self::Stage(source) | Self::Commit(source) => Some(source.as_ref()),
            Self::NotStreamed => None,
        }
    }
}
