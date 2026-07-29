//! Published restart filesystem phases.

use std::fmt;

/// Exact filesystem operation attempted during restart loading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogRestartPhase {
    /// Pin the selected store root.
    OpenRoot,
    /// Open `HEAD` without following links.
    OpenHead,
    /// Read the complete fixed-width head.
    ReadHead,
    /// Open the catalog pool without following links.
    OpenCatalogDirectory,
    /// Open the exact head-selected catalog.
    OpenCatalog,
    /// Read the exact declared catalog bytes.
    ReadCatalog,
    /// Open the segment pool without following links.
    OpenSegmentDirectory,
    /// Open one exact catalog-selected segment.
    OpenSegment,
    /// Read one bounded complete segment.
    ReadSegment,
}

impl fmt::Display for CatalogRestartPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
