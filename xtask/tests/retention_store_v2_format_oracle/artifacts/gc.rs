// This included source owns construction of the GC retirement intent and
// receipt records over the accepted version-1 and version-2 fixtures.

const GC_CANDIDATE_SET_DOMAIN: &[u8] = b"keep.gc-candidate-set/v2\0";
const GC_INTENT_DOMAIN: &[u8] = b"keep.gc-retirement-intent/v2\0";
const GC_INTENT_CHECKSUM_DOMAIN: &[u8] = b"keep.gc-retirement-intent-checksum/v2\0";
const GC_RECEIPT_CHECKSUM_DOMAIN: &[u8] = b"keep.gc-retirement-receipt-checksum/v2\0";
const EMPTY_DISPOSITION_SET_DOMAIN: &[u8] = b"keep.empty-disposition-set/v2\0";
const DISPOSITION_CHECKSUM_DOMAIN: &[u8] = b"keep.recovery-disposition-receipt-checksum/v2\0";
const DISPOSITION_ARTIFACT_DOMAIN: &[u8] = b"keep.recovery-disposition-artifact/v2\0";

struct GcSource {
    candidate_segment_digest: [u8; 32],
    candidate_segment_length: u64,
    candidate_evidence_digest: [u8; 32],
    catalog_digest: [u8; 32],
    catalog_proof_digest: [u8; 32],
    pool_state_digest: [u8; 32],
}

/// Reads every GC coordinate from accepted fixtures at fixed offsets: the
/// one-zero segment supplies the candidate (its digest, length, and record
/// checksum as fixture-only verification evidence); the generation-two
/// catalog and head supply the successor coordinates; the empty segment's
/// digest stands in for the post-retirement pool state.
fn gc_source() -> Result<GcSource, String> {
    let segment = decode_hex(V1_SEGMENT)?;
    let catalog_two = decode_hex(V1_CATALOG_TWO)?;
    let head_two = decode_hex(V1_HEAD_TWO)?;
    let empty_segment = decode_hex(V1_EMPTY_SEGMENT)?;
    require_length(&segment, 337, "version-1 source segment")?;
    require_length(&catalog_two, 352, "version-1 generation-two catalog")?;
    require_length(&head_two, 128, "version-1 generation-two head")?;
    require_length(&empty_segment, 192, "version-1 empty segment")?;
    Ok(GcSource {
        candidate_segment_digest: array_32(&segment, 273)?,
        candidate_segment_length: 337,
        candidate_evidence_digest: array_32(&segment, 177)?,
        catalog_digest: array_32(&catalog_two, 320)?,
        catalog_proof_digest: array_32(&head_two, 96)?,
        pool_state_digest: array_32(&empty_segment, 128)?,
    })
}

fn build_gc_intent(
    profile_digest: [u8; 32],
    inventory_digest: [u8; 32],
    manifest: &ManifestArtifact,
) -> Result<Artifact, String> {
    let source = gc_source()?;
    let mut candidates = Vec::with_capacity(72);
    candidates.extend_from_slice(&source.candidate_segment_digest);
    push_u64(&mut candidates, source.candidate_segment_length);
    candidates.extend_from_slice(&source.candidate_evidence_digest);
    require_length(&candidates, 72, "GC candidate entry")?;
    let candidate_set_digest =
        hash(GC_CANDIDATE_SET_DOMAIN, &[&1_u32.to_be_bytes(), &candidates]);
    let empty_dispositions = hash(EMPTY_DISPOSITION_SET_DOMAIN, &[]);

    let mut bytes = Vec::with_capacity(320 + 72 + 64);
    bytes.extend_from_slice(b"KEEP:GC:INTENT2\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 320);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, 320 + 72 + 64);
    push_u64(&mut bytes, 1);
    push_u16(&mut bytes, 72);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 1);
    push_u64(&mut bytes, 1);
    bytes.extend_from_slice(&manifest.digest);
    push_u64(&mut bytes, 2);
    bytes.extend_from_slice(&source.catalog_digest);
    push_u32(&mut bytes, 1);
    push_u32(&mut bytes, 1);
    bytes.extend_from_slice(&profile_digest);
    bytes.extend_from_slice(&source.catalog_proof_digest);
    bytes.extend_from_slice(&inventory_digest);
    bytes.extend_from_slice(&empty_dispositions);
    push_u64(&mut bytes, 4);
    push_u64(&mut bytes, 5);
    push_u64(&mut bytes, 6);
    bytes.extend_from_slice(&candidate_set_digest);
    require_length(&bytes, 320, "GC intent header")?;
    bytes.extend_from_slice(&candidates);
    let intent_digest = hash(GC_INTENT_DOMAIN, &[&bytes]);
    bytes.extend_from_slice(&intent_digest);
    let checksum = hash(GC_INTENT_CHECKSUM_DOMAIN, &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 456, "GC intent")?;
    Ok(Artifact {
        case_name: "one-candidate-gc-intent",
        kind: "gc-intent",
        generation: "1",
        entry_count: "1",
        bound_digest: intent_digest,
        final_checksum: checksum,
        fixture: "one-candidate-gc-intent.hex",
        bytes,
    })
}

fn build_gc_receipt(intent: &Artifact) -> Result<Artifact, String> {
    let source = gc_source()?;
    let mut bytes = Vec::with_capacity(320);
    bytes.extend_from_slice(b"KEEP:GC:RECEIPT2");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 320);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, u64_at(&intent.bytes, 32)?);
    bytes.extend_from_slice(&intent.bound_digest);
    bytes.extend_from_slice(&array_32(&intent.bytes, 288)?);
    bytes.extend_from_slice(&source.pool_state_digest);
    push_u64(&mut bytes, u64_at(&intent.bytes, 48)?);
    bytes.extend_from_slice(&array_32(&intent.bytes, 56)?);
    push_u64(&mut bytes, u64_at(&intent.bytes, 88)?);
    bytes.extend_from_slice(&array_32(&intent.bytes, 96)?);
    push_u64(&mut bytes, u64_at(&intent.bytes, 264)?);
    push_u64(&mut bytes, u64_at(&intent.bytes, 272)?);
    push_u64(&mut bytes, u64_at(&intent.bytes, 280)?);
    push_u64(&mut bytes, 1);
    bytes.extend_from_slice(&[0; 48]);
    require_length(&bytes, 288, "GC receipt checksum preimage")?;
    let checksum = hash(GC_RECEIPT_CHECKSUM_DOMAIN, &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 320, "GC receipt")?;
    Ok(Artifact {
        case_name: "one-candidate-gc-receipt",
        kind: "gc-receipt",
        generation: "1",
        entry_count: "1",
        bound_digest: intent.bound_digest,
        final_checksum: checksum,
        fixture: "one-candidate-gc-receipt.hex",
        bytes,
    })
}

/// The disposition that retires the one-zero segment as a complete orphan
/// under the generation-two catalog and head, the generation-one manifest,
/// and the fixture-only reader-lock coordinates the GC intent uses.
fn build_recovery_disposition(manifest: &ManifestArtifact) -> Result<Artifact, String> {
    let source = gc_source()?;
    let segment = decode_hex(V1_SEGMENT)?;
    let head_two = decode_hex(V1_HEAD_TWO)?;
    let content_digest = hash(DISPOSITION_ARTIFACT_DOMAIN, &[&segment]);
    let mut bytes = Vec::with_capacity(320);
    bytes.extend_from_slice(b"KEEP:REC:DISP2\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 320);
    push_u32(&mut bytes, 0);
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u64(&mut bytes, source.candidate_segment_length);
    bytes.extend_from_slice(&source.candidate_segment_digest);
    bytes.extend_from_slice(&content_digest);
    push_u64(&mut bytes, u64_at(&head_two, 24)?);
    bytes.extend_from_slice(&source.catalog_proof_digest);
    push_u64(&mut bytes, 2);
    bytes.extend_from_slice(&source.catalog_digest);
    push_u64(&mut bytes, 1);
    bytes.extend_from_slice(&manifest.digest);
    push_u64(&mut bytes, 4);
    push_u64(&mut bytes, 5);
    push_u64(&mut bytes, 6);
    bytes.extend_from_slice(&source.candidate_evidence_digest);
    bytes.extend_from_slice(&[0; 8]);
    require_length(&bytes, 288, "recovery disposition checksum preimage")?;
    let checksum = hash(DISPOSITION_CHECKSUM_DOMAIN, &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 320, "recovery disposition receipt")?;
    Ok(Artifact {
        case_name: "one-orphan-retire-disposition",
        kind: "recovery-disposition",
        generation: "2",
        entry_count: "-",
        bound_digest: source.candidate_segment_digest,
        final_checksum: checksum,
        fixture: "one-orphan-retire-disposition.hex",
        bytes,
    })
}
