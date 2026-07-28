// Test-only catalog and publication-head construction.

struct Catalog {
    bytes: Vec<u8>,
    digest: [u8; 32],
    checksum: [u8; 32],
    generation: u64,
}

fn build_catalog(
    segment_digest: [u8; 32],
    record_checksum: [u8; 32],
) -> Result<Catalog, &'static str> {
    build_catalog_generation(segment_digest, record_checksum, 1, [0; 32])
}

fn build_catalog_generation(
    segment_digest: [u8; 32],
    record_checksum: [u8; 32],
    generation: u64,
    previous_catalog_digest: [u8; 32],
) -> Result<Catalog, &'static str> {
    let mut bytes = Vec::with_capacity(352);
    bytes.extend_from_slice(b"KEEP:CATALOG:V1\0");
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 128);
    push_u16(&mut bytes, 160);
    push_u64(&mut bytes, generation);
    bytes.extend_from_slice(&previous_catalog_digest);
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
        generation,
    })
}

fn build_head(
    digest: [u8; 32],
    length: u64,
    generation: u64,
) -> Result<(Vec<u8>, [u8; 32]), &'static str> {
    let mut bytes = Vec::with_capacity(128);
    bytes.extend_from_slice(b"KEEP:CATHEAD:V1\0");
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 128);
    bytes.push(1);
    bytes.push(1);
    push_u64(&mut bytes, generation);
    push_u64(&mut bytes, length);
    bytes.extend_from_slice(&digest);
    bytes.extend_from_slice(&[0; 24]);
    let checksum = framed_digest(b"KEEP:CATHEAD:SUM\0", &bytes)?;
    bytes.extend_from_slice(&checksum);
    Ok((bytes, checksum))
}

fn build_head_for_catalog(catalog: &Catalog) -> Result<(Vec<u8>, [u8; 32]), &'static str> {
    let length =
        u64::try_from(catalog.bytes.len()).map_err(|_| "constructed catalog length overflow")?;
    build_head(catalog.digest, length, catalog.generation)
}
