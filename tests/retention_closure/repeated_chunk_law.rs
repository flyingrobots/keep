//! Repeated logical chunk accounting law.

use std::cell::RefCell;
use std::error::Error;
use std::io::{self, Write};
use std::rc::Rc;

use keep::{
    AdmittedLayout, AdmittedSegment, AdmittedSegmentRecord, BlobId, CanonicalCatalog,
    CanonicalPublicationHead, CatalogGeneration, ChecksummedPublicationHead, FastCdc,
    LayoutEntryLimit, RegisteredRetentionProfile, RegisteredStorageProfile, RetentionAnchor,
    RetentionClosureLimits, RetentionNamespace, RetentionPolicy, RetentionRoot, RootGeneration,
    SegmentReadPolicy, SegmentRecordLimit, SegmentStage, StagedSegment, verify_retention_closure,
};

const REPETITIONS: usize = 3;
const RECORD_OVERHEAD: u64 = 144;

#[test]
fn repeated_chunk_occurrences_consume_physical_bytes_not_unique_nodes() -> Result<(), Box<dyn Error>>
{
    let repeated = repeated_source()?;
    let source = repeated.bytes;
    let spans = repeated.spans;
    let blob = BlobId::hash_bytes(&source)?;
    let layout = AdmittedLayout::from_spans(
        blob,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let canonical_layout = layout.encode_record()?;
    let chunk_length = source
        .len()
        .checked_div(REPETITIONS)
        .ok_or("repetition count is zero")?;
    let chunk = source
        .get(..chunk_length)
        .ok_or("repeated source omits its first chunk")?;
    let (stage, probe) = MemoryStage::new();
    let staged = StagedSegment::begin(stage, SegmentRecordLimit::MAXIMUM)?;
    let staged = staged.append(AdmittedSegmentRecord::for_chunk(chunk)?)?;
    let staged = staged.append(AdmittedSegmentRecord::for_layout(&canonical_layout)?)?;
    let _sealed = staged.seal()?;
    let segment_bytes = probe.bytes();
    let segment = AdmittedSegment::decode(&segment_bytes, maximum_policy())?;
    let segments = [segment];
    let canonical_catalog =
        CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let canonical_head = CanonicalPublicationHead::for_catalog(canonical_catalog.checksummed());
    let catalog = canonical_catalog.checksummed().admit(&segments)?;
    let head = ChecksummedPublicationHead::decode(canonical_head.encoded())?;
    let snapshot = head.admit(catalog)?;
    let encoded_bytes = u64::try_from(canonical_layout.bytes().len())?;
    let chunk_bytes = u64::try_from(chunk.len())?;
    let repetitions = u64::try_from(REPETITIONS)?;
    let physical_bytes = RECORD_OVERHEAD
        .checked_add(encoded_bytes)
        .and_then(|layout_record| {
            RECORD_OVERHEAD
                .checked_add(chunk_bytes)
                .and_then(|chunk_record| chunk_record.checked_mul(repetitions))
                .and_then(|chunks| layout_record.checked_add(chunks))
        })
        .ok_or("physical-byte oracle overflowed")?;
    let root = RetentionRoot::new(
        RetentionNamespace::try_from(b"repeated".as_slice())?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(2, 2, encoded_bytes, physical_bytes)?,
        ),
        None,
        vec![RetentionAnchor::new(blob, canonical_layout.id())],
    )?;

    let evidence = verify_retention_closure(&root, &snapshot)?;

    assert_eq!(evidence.usage().node_count(), 2);
    assert_eq!(evidence.usage().maximum_depth(), 2);
    assert_eq!(evidence.usage().encoded_bytes(), encoded_bytes);
    assert_eq!(evidence.usage().physical_bytes(), physical_bytes);
    Ok(())
}

struct RepeatedSource {
    bytes: Vec<u8>,
    spans: Vec<keep::ChunkSpan>,
}

fn repeated_source() -> Result<RepeatedSource, Box<dyn Error>> {
    let candidate_length = usize::try_from(
        RegisteredStorageProfile::FAST_CDC_64K_V1
            .maximum_chunk_length()
            .get(),
    )?;
    let candidate = vec![0_u8; candidate_length];
    let first_pass = detect(&candidate)?;
    let first = first_pass
        .first()
        .copied()
        .ok_or("profile emitted no candidate chunk")?;
    let chunk_length = usize::try_from(first.length().get())?;
    let chunk = candidate
        .get(..chunk_length)
        .ok_or("candidate omits its first emitted chunk")?;
    let source_length = chunk_length
        .checked_mul(REPETITIONS)
        .ok_or("repeated source length overflowed")?;
    let mut source = Vec::new();
    source.try_reserve_exact(source_length)?;
    for _index in 0..REPETITIONS {
        source.extend_from_slice(chunk);
    }
    let spans = detect(&source)?;
    if spans.len() != REPETITIONS || !spans.iter().all(|span| span.id() == first.id()) {
        return Err("repeated source did not reproduce one exact chunk identity".into());
    }
    Ok(RepeatedSource {
        bytes: source,
        spans,
    })
}

fn detect(bytes: &[u8]) -> Result<Vec<keep::ChunkSpan>, Box<dyn Error>> {
    let mut spans = Vec::new();
    let mut detector = FastCdc::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok(spans)
}

struct MemoryStage {
    bytes: Rc<RefCell<Vec<u8>>>,
}

struct MemoryProbe {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl MemoryStage {
    fn new() -> (Self, MemoryProbe) {
        let bytes = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                bytes: Rc::clone(&bytes),
            },
            MemoryProbe { bytes },
        )
    }
}

impl Write for MemoryStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for MemoryStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl MemoryProbe {
    fn bytes(&self) -> Vec<u8> {
        self.bytes.borrow().clone()
    }
}

const fn maximum_policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
