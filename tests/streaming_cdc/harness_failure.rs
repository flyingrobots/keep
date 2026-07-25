//! Typed failures for the streaming CDC conformance harness.

use std::error::Error;
use std::fmt;

use keep::{ChunkHashError, ChunkingError};

#[derive(Debug)]
pub(super) enum HarnessFailure {
    Corpus { fact: &'static str },
    ChunkHash(ChunkHashError),
    Chunking(ChunkingError),
}

impl HarnessFailure {
    pub(super) const fn corpus(fact: &'static str) -> Self {
        Self::Corpus { fact }
    }
}

impl fmt::Display for HarnessFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corpus { fact } => write!(formatter, "invalid CDC corpus: {fact}"),
            Self::ChunkHash(source) => source.fmt(formatter),
            Self::Chunking(source) => source.fmt(formatter),
        }
    }
}

impl Error for HarnessFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ChunkHash(source) => Some(source),
            Self::Chunking(source) => Some(source),
            Self::Corpus { .. } => None,
        }
    }
}

impl From<ChunkHashError> for HarnessFailure {
    fn from(source: ChunkHashError) -> Self {
        Self::ChunkHash(source)
    }
}

impl From<ChunkingError> for HarnessFailure {
    fn from(source: ChunkingError) -> Self {
        Self::Chunking(source)
    }
}
