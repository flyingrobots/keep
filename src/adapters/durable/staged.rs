//! This boundary module owns a durable staging awaiting commit: the sealed
//! stage's selection input, the identities it established, and the
//! generation it was verified against.

use std::io;

use super::DurableIngestionError as Error;
use super::ingestion_receipt::{DurableIngestionReceipt, IngestionAccounting};
use super::writer::DurableWriter;
use crate::adapters::filesystem_catalog_publisher::{CURRENT_SEGMENT, ClosedSelection};
use crate::adapters::{
    AdmittedSegment, CanonicalCatalog, CatalogPublicationExpectation, FilesystemCatalogSnapshot,
    filesystem_exact_record as exact_record, publish_catalog_generation,
};
use crate::{BlobId, CatalogGeneration, LayoutId};

/// Content staged into a durable store but not yet visible.
///
/// Dropping the value without committing publishes nothing; the sealed
/// `staging/current.seg`, when one was written, is left for the recovery
/// protocol to discard.
#[must_use = "staged work remains invisible until commit is called"]
pub struct DurableStagedBlob<'writer> {
    writer: &'writer mut DurableWriter,
    target: BlobId,
    layout_id: LayoutId,
    generation: CatalogGeneration,
    closed: Option<ClosedSelection>,
    accounting: IngestionAccounting,
}

impl<'writer> DurableStagedBlob<'writer> {
    pub(super) const fn new(
        writer: &'writer mut DurableWriter,
        identities: (BlobId, LayoutId),
        generation: CatalogGeneration,
        closed: Option<ClosedSelection>,
        accounting: IngestionAccounting,
    ) -> Self {
        Self {
            writer,
            target: identities.0,
            layout_id: identities.1,
            generation,
            closed,
            accounting,
        }
    }

    /// The complete source identity.
    #[must_use]
    pub const fn target(&self) -> BlobId {
        self.target
    }

    /// The canonical layout the staging will commit under.
    #[must_use]
    pub const fn layout_id(&self) -> LayoutId {
        self.layout_id
    }

    /// The exact byte accounting so far; the receipt repeats it.
    #[must_use]
    pub const fn accounting(&self) -> IngestionAccounting {
        self.accounting
    }

    /// Whether the pinned catalog already held every chunk and the layout,
    /// so commit publishes nothing.
    #[must_use]
    pub const fn is_already_visible(&self) -> bool {
        self.closed.is_none()
    }

    /// Publishes the sealed segment and a catalog successor naming every
    /// current segment and the new one, or publishes nothing when the
    /// catalog already held everything.
    ///
    /// # Errors
    ///
    /// Returns [`DurableIngestionError`](Error) at the exact catalog,
    /// selection, successor, or publication refusal; a publication refusal
    /// leaves the completed phases' residue for the recovery protocol.
    pub fn commit(self) -> Result<DurableIngestionReceipt, Error> {
        let Self {
            writer,
            target,
            layout_id,
            generation,
            closed,
            accounting,
        } = self;
        let current = FilesystemCatalogSnapshot::load(&writer.store_root, writer.policy)
            .map_err(|source| Error::Catalog(Box::new(source)))?;
        let snapshot = current
            .snapshot()
            .map_err(|source| Error::Catalog(Box::new(source)))?;
        if snapshot.generation() != generation {
            return Err(Error::CatalogMoved {
                staged: generation,
                observed: snapshot.generation(),
            });
        }
        let Some(closed) = closed else {
            return Ok(DurableIngestionReceipt::new(
                target,
                layout_id,
                None,
                (snapshot.generation(), snapshot.catalog_digest()),
                accounting,
            ));
        };
        let new_bytes = read_stage(writer, &closed)?;
        let policy = writer.policy.segment_read();
        let new_segment = AdmittedSegment::decode(&new_bytes, policy)
            .map_err(|source| Error::Segment(Box::new(source)))?;
        let mut segments = Vec::new();
        for loaded in current.loaded_segments() {
            segments.push(
                AdmittedSegment::decode(loaded.encoded(), policy)
                    .map_err(|source| Error::Segment(Box::new(source)))?,
            );
        }
        segments.push(
            AdmittedSegment::decode(&new_bytes, policy)
                .map_err(|source| Error::Segment(Box::new(source)))?,
        );
        let selection = writer
            .publisher
            .select_closed(closed, &new_segment)
            .map_err(Error::Selection)?;
        let successor = CanonicalCatalog::from_segments(
            generation.successor().map_err(Error::Generation)?,
            Some(snapshot.catalog_digest()),
            &segments,
        )
        .map_err(|source| Error::Successor(Box::new(source)))?;
        let receipt = publish_catalog_generation(
            &mut writer.publisher,
            CatalogPublicationExpectation::successor_of(&snapshot),
            selection,
            &successor,
            &segments,
        )
        .map_err(|source| Error::Publish(Box::new(source)))?;
        Ok(DurableIngestionReceipt::new(
            target,
            layout_id,
            Some(new_segment.digest()),
            (receipt.generation(), receipt.catalog_digest()),
            accounting,
        ))
    }
}

fn read_stage(writer: &DurableWriter, closed: &ClosedSelection) -> Result<Vec<u8>, Error> {
    let length = usize::try_from(closed.segment_length()).map_err(|source| Error::ReadStage {
        source: io::Error::new(io::ErrorKind::InvalidData, source),
    })?;
    exact_record::read_exact_regular(&writer.publisher.staging, CURRENT_SEGMENT, length).map_err(
        |source| Error::ReadStage {
            source: match source {
                exact_record::ExactRecordError::Io(source) => source,
                refused @ exact_record::ExactRecordError::Refused(_) => {
                    io::Error::new(io::ErrorKind::InvalidData, refused)
                }
            },
        },
    )
}
