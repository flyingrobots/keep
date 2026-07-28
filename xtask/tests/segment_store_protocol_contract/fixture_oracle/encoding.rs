// Test-only fixed-width construction for the frozen golden artifacts.

const ONE_ZERO_CHUNK_DIGEST: [u8; 32] = [
    0x9b, 0x9c, 0x9a, 0x42, 0x91, 0x2a, 0x0e, 0xfd, 0xcd, 0x41, 0xe8, 0x3e, 0xa0, 0x24, 0xd7, 0x2f,
    0x10, 0xf2, 0x62, 0x7d, 0x23, 0x9e, 0x4e, 0xb2, 0x40, 0xdd, 0x53, 0xf3, 0x9c, 0xe0, 0xff, 0x62,
];

struct Segment {
    bytes: Vec<u8>,
    digest: [u8; 32],
    seal_checksum: [u8; 32],
    record_checksum: Option<[u8; 32]>,
}

struct Catalog {
    bytes: Vec<u8>,
    digest: [u8; 32],
    checksum: [u8; 32],
}

fn build_empty_segment() -> Result<Segment, &'static str> {
    finish_segment(segment_header(), 0, 0, None)
}

fn build_one_zero_segment() -> Result<Segment, &'static str> {
    let (record, checksum) = one_zero_record()?;
    let record_bytes = u64::try_from(record.len()).map_err(|_| "record length overflow")?;
    let mut prefix = segment_header();
    prefix.extend_from_slice(&record);
    finish_segment(prefix, 1, record_bytes, Some(checksum))
}

fn segment_header() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(b"KEEP:SEGMENT:V1\0");
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 64);
    push_u16(&mut bytes, 112);
    push_u16(&mut bytes, 128);
    push_u16(&mut bytes, 0);
    push_u64(&mut bytes, 67_108_864);
    push_u64(&mut bytes, 1_073_741_824);
    push_u32(&mut bytes, 1_048_576);
    bytes.push(1);
    bytes.push(1);
    bytes.extend_from_slice(&[0; 14]);
    bytes
}

fn one_zero_record() -> Result<(Vec<u8>, [u8; 32]), &'static str> {
    let mut header = Vec::with_capacity(112);
    header.extend_from_slice(b"KEEP:SEG:RECORD\0");
    push_u16(&mut header, 1);
    header.push(1);
    header.push(0);
    push_u16(&mut header, 112);
    push_u16(&mut header, 36);
    push_u64(&mut header, 1);
    push_u64(&mut header, 145);
    header.push(1);
    push_u16(&mut header, 1);
    header.push(1);
    header.extend_from_slice(&[0; 4]);
    push_u32(&mut header, 1);
    header.extend_from_slice(&ONE_ZERO_CHUNK_DIGEST);
    header.extend_from_slice(&[0; 24]);
    header.extend_from_slice(&[0; 4]);

    let mut checked_bytes = header;
    checked_bytes.push(0);
    let checksum = framed_digest(b"KEEP:SEG:RECORD:SUM\0", &checked_bytes)?;
    checked_bytes.extend_from_slice(&checksum);
    Ok((checked_bytes, checksum))
}

fn finish_segment(
    mut prefix: Vec<u8>,
    record_count: u32,
    record_bytes: u64,
    record_checksum: Option<[u8; 32]>,
) -> Result<Segment, &'static str> {
    let bytes_before_seal =
        u64::try_from(prefix.len()).map_err(|_| "segment prefix length overflow")?;
    let segment_length = bytes_before_seal
        .checked_add(128)
        .ok_or("segment length overflow")?;
    let mut seal = Vec::with_capacity(128);
    seal.extend_from_slice(b"KEEP:SEGMENT:END");
    push_u16(&mut seal, 1);
    push_u16(&mut seal, 0);
    push_u16(&mut seal, 128);
    push_u16(&mut seal, 0);
    push_u32(&mut seal, record_count);
    push_u32(&mut seal, 0);
    push_u64(&mut seal, bytes_before_seal);
    push_u64(&mut seal, segment_length);
    push_u64(&mut seal, record_bytes);
    seal.push(1);
    seal.push(1);
    seal.extend_from_slice(&[0; 6]);

    let mut digest_input = prefix.clone();
    digest_input.extend_from_slice(&seal);
    let digest = framed_digest(b"KEEP:SEGMENT:DIGEST\0", &digest_input)?;
    seal.extend_from_slice(&digest);
    let seal_checksum = framed_digest(b"KEEP:SEGMENT:SEAL:SUM\0", &seal)?;
    seal.extend_from_slice(&seal_checksum);
    prefix.extend_from_slice(&seal);

    Ok(Segment {
        bytes: prefix,
        digest,
        seal_checksum,
        record_checksum,
    })
}

fn build_catalog(
    segment_digest: [u8; 32],
    record_checksum: [u8; 32],
) -> Result<Catalog, &'static str> {
    let mut bytes = Vec::with_capacity(352);
    bytes.extend_from_slice(b"KEEP:CATALOG:V1\0");
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 128);
    push_u16(&mut bytes, 160);
    push_u64(&mut bytes, 1);
    bytes.extend_from_slice(&[0; 32]);
    push_u64(&mut bytes, 1);
    push_u64(&mut bytes, 352);
    bytes.push(1);
    bytes.push(1);
    bytes.extend_from_slice(&[0; 46]);

    bytes.push(1);
    bytes.push(0);
    push_u16(&mut bytes, 36);
    push_u32(&mut bytes, 1);
    bytes.extend_from_slice(&ONE_ZERO_CHUNK_DIGEST);
    bytes.extend_from_slice(&[0; 24]);
    bytes.extend_from_slice(&segment_digest);
    push_u64(&mut bytes, 64);
    push_u64(&mut bytes, 145);
    push_u64(&mut bytes, 1);
    bytes.extend_from_slice(&record_checksum);
    bytes.extend_from_slice(&[0; 8]);

    let checksum = framed_digest(b"KEEP:CATALOG:SUM\0", &bytes)?;
    bytes.extend_from_slice(&checksum);
    let digest = framed_digest(b"KEEP:CATALOG:DIGEST\0", &bytes)?;
    bytes.extend_from_slice(&digest);

    Ok(Catalog {
        bytes,
        digest,
        checksum,
    })
}

fn build_head(digest: [u8; 32], length: u64) -> Result<(Vec<u8>, [u8; 32]), &'static str> {
    let mut bytes = Vec::with_capacity(128);
    bytes.extend_from_slice(b"KEEP:CATHEAD:V1\0");
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 128);
    bytes.push(1);
    bytes.push(1);
    push_u64(&mut bytes, 1);
    push_u64(&mut bytes, length);
    bytes.extend_from_slice(&digest);
    bytes.extend_from_slice(&[0; 24]);
    let checksum = framed_digest(b"KEEP:CATHEAD:SUM\0", &bytes)?;
    bytes.extend_from_slice(&checksum);
    Ok((bytes, checksum))
}

fn framed_digest(domain: &[u8], bytes: &[u8]) -> Result<[u8; 32], &'static str> {
    let length = u64::try_from(bytes.len()).map_err(|_| "digest input length overflow")?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&1u16.to_be_bytes());
    hasher.update(&[1]);
    hasher.update(bytes);
    hasher.update(&length.to_be_bytes());
    Ok(*hasher.finalize().as_bytes())
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}
