//! Catalog-ceiling evidence at the public admission and verification boundary.

use keep::{
    AdmittedSegment, AdmittedSegmentRecord, CanonicalCatalog, CanonicalPublicationHead,
    CatalogGeneration, ChecksummedPublicationHead, SegmentReadPolicy, SegmentRecordLimit,
    SegmentStage, StagedSegment, VerificationDepth, VerificationSubject,
};
use std::error::Error;
use std::io::{self, Write};

// Size: small by resource topology (one thread, no I/O), heavyweight by memory
// and latency. Oracle: v1's 1048576-entry ceiling, exact named sample bytes,
// and the documented 1 GiB incremental admission/report allocation ceiling.
// Delete if the format ceiling changes or stronger bounded admission covers it.
#[test]
fn the_catalog_entry_ceiling_verifies_within_its_declared_memory_bound()
-> Result<(), Box<dyn Error>> {
    let bytes = ceiling_segment()?;
    let segments = [AdmittedSegment::decode(&bytes, SegmentReadPolicy::MAXIMUM)?];
    let catalog = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
    let head = CanonicalPublicationHead::for_catalog(catalog.checksummed());
    let mut outcome = None;
    let allocation = allocation_counter::measure(|| {
        outcome = Some(verify_ceiling(
            head.encoded(),
            catalog.checksummed(),
            &segments,
        ));
    });
    let report = outcome.ok_or("verification did not run")??;
    assert_eq!(report.requested(), VerificationDepth::CatalogReachability);
    assert_eq!(
        report
            .subjects()
            .first()
            .ok_or("missing catalog evidence")?
            .subject(),
        VerificationSubject::Catalog {
            generation: CatalogGeneration::new(1)?,
            digest: catalog.checksummed().digest()
        }
    );
    assert!(
        allocation.bytes_max <= 1_073_741_824,
        "catalog-ceiling admission/report memory exceeded 1 GiB: {allocation:?}"
    );
    Ok(())
}

fn verify_ceiling(
    head: &[u8],
    catalog: keep::ChecksummedCatalog<'_>,
    segments: &[AdmittedSegment<'_>],
) -> Result<keep::VerificationReport, Box<dyn Error>> {
    let snapshot = ChecksummedPublicationHead::decode(head)?.admit(catalog.admit(segments)?)?;
    assert_eq!(
        snapshot.record_count(),
        1_048_576,
        "the admitted catalog must contain the full protocol domain"
    );
    for value in [0_u64, 524_288, 1_048_575] {
        let expected = value.to_be_bytes();
        let identity = keep::SegmentRecordIdentity::Chunk(keep::ChunkId::hash_bytes(&expected)?);
        assert_eq!(
            snapshot
                .record(identity)
                .ok_or("named ceiling-domain record absent")?
                .payload(),
            expected,
            "logical lookup must return the named sample's exact bytes"
        );
    }
    Ok(snapshot.verify(VerificationDepth::CatalogReachability)?)
}

fn ceiling_segment() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    let mut writer = StagedSegment::begin(MemorySegment(&mut bytes), SegmentRecordLimit::MAXIMUM)?;
    for value in 0_u64..1_048_576 {
        writer = writer.append(AdmittedSegmentRecord::for_chunk(&value.to_be_bytes())?)?;
    }
    let sealed = writer.seal()?;
    let _closed = sealed.close();
    Ok(bytes)
}

struct MemorySegment<'a>(&'a mut Vec<u8>);
impl Write for MemorySegment<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl SegmentStage for MemorySegment<'_> {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
