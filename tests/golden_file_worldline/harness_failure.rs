//! Typed failures for the Golden File Worldline harness.

use std::error::Error;
use std::fmt;

use keep::{
    BlobHashError, BlobId, BlobIdBinaryParseError, BlobIdTextParseError, BlobReadError,
    IngestionError, PublishError, ReconstructionError,
};

use super::reference_model::ReferenceModelError;

#[derive(Debug)]
pub(super) enum HarnessFailure {
    Corpus { fact: &'static str },
    NamedBytesMismatch { expected: BlobId, observed: BlobId },
    Hash(BlobHashError),
    Binary(BlobIdBinaryParseError),
    Text(BlobIdTextParseError),
    Read(BlobReadError),
    Ingestion(IngestionError),
    Publish(PublishError),
    Reconstruction(Box<ReconstructionError>),
    Model(ReferenceModelError),
}

impl HarnessFailure {
    pub(super) const fn corpus(fact: &'static str) -> Self {
        Self::Corpus { fact }
    }
}

impl fmt::Display for HarnessFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corpus { fact } => write!(formatter, "invalid conformance corpus: {fact}"),
            Self::NamedBytesMismatch { expected, observed } => write!(
                formatter,
                "named bytes mismatch: expected {expected}, observed {observed}"
            ),
            Self::Hash(source) => source.fmt(formatter),
            Self::Binary(source) => source.fmt(formatter),
            Self::Text(source) => source.fmt(formatter),
            Self::Read(source) => source.fmt(formatter),
            Self::Ingestion(source) => source.fmt(formatter),
            Self::Publish(source) => source.fmt(formatter),
            Self::Reconstruction(source) => source.fmt(formatter),
            Self::Model(source) => source.fmt(formatter),
        }
    }
}

impl Error for HarnessFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Hash(source) => Some(source),
            Self::Binary(source) => Some(source),
            Self::Text(source) => Some(source),
            Self::Read(source) => Some(source),
            Self::Ingestion(source) => Some(source),
            Self::Publish(source) => Some(source),
            Self::Reconstruction(source) => Some(source),
            Self::Model(source) => Some(source),
            Self::Corpus { .. } | Self::NamedBytesMismatch { .. } => None,
        }
    }
}

impl From<BlobHashError> for HarnessFailure {
    fn from(source: BlobHashError) -> Self {
        Self::Hash(source)
    }
}

impl From<BlobIdBinaryParseError> for HarnessFailure {
    fn from(source: BlobIdBinaryParseError) -> Self {
        Self::Binary(source)
    }
}

impl From<BlobIdTextParseError> for HarnessFailure {
    fn from(source: BlobIdTextParseError) -> Self {
        Self::Text(source)
    }
}

impl From<BlobReadError> for HarnessFailure {
    fn from(source: BlobReadError) -> Self {
        Self::Read(source)
    }
}

impl From<IngestionError> for HarnessFailure {
    fn from(source: IngestionError) -> Self {
        Self::Ingestion(source)
    }
}

impl From<PublishError> for HarnessFailure {
    fn from(source: PublishError) -> Self {
        Self::Publish(source)
    }
}

impl From<ReconstructionError> for HarnessFailure {
    fn from(source: ReconstructionError) -> Self {
        Self::Reconstruction(Box::new(source))
    }
}

impl From<ReferenceModelError> for HarnessFailure {
    fn from(source: ReferenceModelError) -> Self {
        Self::Model(source)
    }
}
