//! This module owns fixed recovery-stage identities and bounds.

use std::fmt;

use super::segment_header;
use crate::CatalogLength;

/// One fixed mutable artifact retained for explicit recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryStage {
    /// `staging/current.seg`.
    Segment,
    /// `staging/current.cat`.
    Catalog,
    /// Root `head.next`.
    NextHead,
}

impl RecoveryStage {
    pub(super) const fn file_name(self) -> &'static str {
        match self {
            Self::Segment => "current.seg",
            Self::Catalog => "current.cat",
            Self::NextHead => "head.next",
        }
    }

    /// Returns the name-selected version-1 maximum byte length.
    #[must_use]
    pub const fn maximum_length(self) -> u64 {
        match self {
            Self::Segment => segment_header::MAXIMUM_SEGMENT_LENGTH,
            Self::Catalog => CatalogLength::MAXIMUM.get(),
            Self::NextHead => 128_u64,
        }
    }
}

impl fmt::Display for RecoveryStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.file_name())
    }
}
