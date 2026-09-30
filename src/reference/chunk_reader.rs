//! A pull reader over one admitted layout: each chunk is looked up and
//! authenticated as it is first served, and served from the view's own
//! immutable bytes.

use std::io::{self, Read};

use super::chunk_verification::{ChunkSource, ChunkVerificationError, verified_chunk};
use crate::adapters::TransferSourceError;
use crate::{AdmittedLayout, LayoutId};

/// Serves one layout's bytes in order, one verified chunk at a time.
#[expect(
    clippy::redundant_pub_crate,
    reason = "reached from the pipeline and durable adapters only through the crate-private re-export"
)]
pub(crate) struct ChunkReader<'view, S: ?Sized> {
    source: &'view S,
    layout_id: LayoutId,
    layout: &'view AdmittedLayout,
    next_entry: usize,
    current: Option<(&'view [u8], usize)>,
    refusal: Option<ChunkVerificationError>,
}

impl<'view, S: ChunkSource + ?Sized> ChunkReader<'view, S> {
    pub(crate) const fn new(
        source: &'view S,
        layout_id: LayoutId,
        layout: &'view AdmittedLayout,
    ) -> Self {
        Self {
            source,
            layout_id,
            layout,
            next_entry: 0,
            current: None,
            refusal: None,
        }
    }

    /// The chunk refusal that stopped the reader, if one did, in the
    /// transfer-source vocabulary.
    pub(crate) fn transfer_refusal(&self) -> Option<TransferSourceError> {
        self.refusal.map(source_error)
    }

    #[cfg(test)]
    const fn refusal(&self) -> Option<ChunkVerificationError> {
        self.refusal
    }

    fn advance(&mut self) -> io::Result<bool> {
        if let Some(refusal) = self.refusal {
            return Err(refused(refusal));
        }
        let Some(entry) = self.layout.entries().get(self.next_entry) else {
            return Ok(false);
        };
        match verified_chunk(self.source, self.layout_id, self.next_entry, *entry) {
            Ok(bytes) => {
                self.current = Some((bytes, 0));
                self.next_entry = self.next_entry.saturating_add(1);
                Ok(true)
            }
            Err(refusal) => {
                self.refusal = Some(refusal);
                Err(refused(refusal))
            }
        }
    }
}

/// Maps a chunk refusal into the public transfer-source vocabulary.
const fn source_error(refusal: ChunkVerificationError) -> TransferSourceError {
    match refusal {
        ChunkVerificationError::Missing {
            layout,
            index,
            requested,
        } => TransferSourceError::ChunkMissing {
            layout,
            index,
            requested,
        },
        ChunkVerificationError::Hash {
            layout,
            index,
            source,
            ..
        } => TransferSourceError::ChunkHash {
            layout,
            index,
            source,
        },
        ChunkVerificationError::IdentityMismatch {
            layout,
            index,
            expected,
            observed,
        } => TransferSourceError::ChunkIdentityMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}

fn refused(refusal: ChunkVerificationError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{refusal:?}"))
}

impl<S: ChunkSource + ?Sized> Read for ChunkReader<'_, S> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        loop {
            let Some((bytes, served)) = self.current else {
                if self.advance()? {
                    continue;
                }
                return Ok(0);
            };
            let Some(remaining) = bytes.get(served..) else {
                self.current = None;
                continue;
            };
            if remaining.is_empty() {
                self.current = None;
                continue;
            }
            let take = remaining.len().min(buffer.len());
            let (head, _) = remaining.split_at(take);
            if let Some(slot) = buffer.get_mut(..take) {
                slot.copy_from_slice(head);
            }
            self.current = Some((bytes, served.saturating_add(take)));
            return Ok(take);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Read;

    use super::super::chunk_verification::{ChunkSource, ChunkVerificationError};
    use super::ChunkReader;
    use crate::{
        AdmittedLayout, BlobHasher, ChunkId, FastCdc, LayoutEntryLimit, RegisteredStorageProfile,
    };

    struct Chunks(BTreeMap<ChunkId, Vec<u8>>);

    impl ChunkSource for Chunks {
        fn chunk(&self, identity: ChunkId) -> Option<&[u8]> {
            self.0.get(&identity).map(Vec::as_slice)
        }
    }

    fn content() -> Vec<u8> {
        let mut state = 0x1357_9bdf_2468_ace0_u64;
        (0..300 * 1024)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                u8::try_from(state & 0xff).unwrap_or_default()
            })
            .collect()
    }

    fn layout_and_chunks(
        bytes: &[u8],
    ) -> Result<(AdmittedLayout, Chunks), Box<dyn std::error::Error>> {
        let mut hasher = BlobHasher::new();
        hasher.update(bytes)?;
        let mut detector = FastCdc::new();
        let mut spans = Vec::new();
        detector.feed(bytes, |span| spans.push(span))?;
        if let Some(span) = detector.finish()? {
            spans.push(span);
        }
        let mut chunks = BTreeMap::new();
        for span in &spans {
            let start = usize::try_from(span.offset().get())?;
            let end = usize::try_from(span.end().get())?;
            chunks.insert(span.id(), bytes.get(start..end).ok_or("span")?.to_vec());
        }
        let layout = AdmittedLayout::from_spans(
            hasher.finish(),
            RegisteredStorageProfile::FAST_CDC_64K_V1,
            spans,
            LayoutEntryLimit::MAXIMUM,
        )?;
        Ok((layout, Chunks(chunks)))
    }

    #[test]
    fn the_reader_serves_every_chunk_in_order_across_small_reads()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = content();
        let (layout, chunks) = layout_and_chunks(&bytes)?;
        let layout_id = layout.encode_record()?.id();
        let mut reader = ChunkReader::new(&chunks, layout_id, &layout);
        let mut output = Vec::new();
        let mut buffer = [0_u8; 1000];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            output.extend_from_slice(buffer.get(..read).ok_or("read")?);
        }
        assert_eq!(output, bytes);
        assert!(reader.refusal().is_none());
        Ok(())
    }

    #[test]
    fn a_chunk_that_does_not_hash_to_its_identity_refuses_at_its_boundary()
    -> Result<(), Box<dyn std::error::Error>> {
        let bytes = content();
        let (layout, mut chunks) = layout_and_chunks(&bytes)?;
        let second = layout.entries().get(1).ok_or("second entry")?.chunk_id();
        if let Some(stored) = chunks.0.get_mut(&second)
            && let Some(first) = stored.first_mut()
        {
            *first = first.wrapping_add(1);
        }
        let layout_id = layout.encode_record()?.id();
        let mut reader = ChunkReader::new(&chunks, layout_id, &layout);
        let mut output = Vec::new();
        let Err(refusal) = reader.read_to_end(&mut output) else {
            return Err("expected a chunk refusal".into());
        };
        assert_eq!(refusal.kind(), std::io::ErrorKind::InvalidData);
        let first_length = usize::try_from(
            layout
                .entries()
                .get(1)
                .ok_or("second entry")?
                .offset()
                .get(),
        )?;
        assert_eq!(output.len(), first_length);
        assert!(matches!(
            reader.refusal(),
            Some(ChunkVerificationError::IdentityMismatch { index: 1, .. })
        ));
        Ok(())
    }
}
