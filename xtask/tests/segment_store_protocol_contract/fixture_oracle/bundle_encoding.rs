// Cross-kind golden construction for one chunk and its canonical layout.

const ONE_ZERO_LAYOUT_HEX: &str =
    include_str!("../../../../conformance/layout/v1/one-zero.layout.hex");
const ONE_ZERO_LAYOUT_ID_HEX: &str = "\
    4b4545503a4c41594f55543a49440000\
    0001000100000000000000dc\
    887da23f1a7483359a78fc9a7fde80030ec2c4690603803f0ab7d0edb56575b8\n";

struct Bundle {
    segment: Segment,
    chunk_checksum: [u8; 32],
    layout_checksum: [u8; 32],
}

fn build_one_zero_bundle_segment() -> Result<Bundle, &'static str> {
    let (chunk_record, chunk_checksum) = one_zero_record()?;
    let (layout_record, layout_checksum) = one_zero_layout_record()?;
    let record_bytes = chunk_record
        .len()
        .checked_add(layout_record.len())
        .ok_or("bundle record length overflow")?;
    let record_bytes =
        u64::try_from(record_bytes).map_err(|_| "bundle record length overflow")?;
    let mut prefix = segment_header();
    prefix.extend_from_slice(&chunk_record);
    prefix.extend_from_slice(&layout_record);
    let segment = finish_segment(prefix, 2, record_bytes, None)?;
    Ok(Bundle {
        segment,
        chunk_checksum,
        layout_checksum,
    })
}

fn one_zero_layout_record() -> Result<(Vec<u8>, [u8; 32]), &'static str> {
    let payload = decode_hex(ONE_ZERO_LAYOUT_HEX)?;
    let identity = decode_hex(ONE_ZERO_LAYOUT_ID_HEX)?;
    if payload.len() != 220 || identity.len() != 60 {
        return Err("one-zero layout fixture has an unexpected length");
    }

    let mut header = Vec::with_capacity(112);
    header.extend_from_slice(b"KEEP:SEG:RECORD\0");
    push_u16(&mut header, 1);
    header.push(2);
    header.push(0);
    push_u16(&mut header, 112);
    push_u16(&mut header, 60);
    push_u64(&mut header, 220);
    push_u64(&mut header, 364);
    header.push(1);
    push_u16(&mut header, 1);
    header.push(1);
    header.extend_from_slice(&[0; 4]);
    header.extend_from_slice(&identity);
    header.extend_from_slice(&[0; 4]);

    let mut checked_bytes = header;
    checked_bytes.extend_from_slice(&payload);
    let checksum = framed_digest(b"KEEP:SEG:RECORD:SUM\0", &checked_bytes)?;
    checked_bytes.extend_from_slice(&checksum);
    Ok((checked_bytes, checksum))
}

fn build_bundle_catalog(
    segment_digest: [u8; 32],
    chunk_checksum: [u8; 32],
    layout_checksum: [u8; 32],
) -> Result<Catalog, &'static str> {
    let mut bytes = catalog_header(2, 512);
    append_chunk_entry(&mut bytes, segment_digest, chunk_checksum);
    append_layout_entry(&mut bytes, segment_digest, layout_checksum)?;
    finish_catalog(bytes)
}

fn catalog_header(entry_count: u64, catalog_length: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(512);
    bytes.extend_from_slice(b"KEEP:CATALOG:V1\0");
    push_u16(&mut bytes, 1);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 128);
    push_u16(&mut bytes, 160);
    push_u64(&mut bytes, 1);
    bytes.extend_from_slice(&[0; 32]);
    push_u64(&mut bytes, entry_count);
    push_u64(&mut bytes, catalog_length);
    bytes.push(1);
    bytes.push(1);
    bytes.extend_from_slice(&[0; 46]);
    bytes
}

fn append_chunk_entry(
    bytes: &mut Vec<u8>,
    segment_digest: [u8; 32],
    record_checksum: [u8; 32],
) {
    bytes.push(1);
    bytes.push(0);
    push_u16(bytes, 36);
    push_u32(bytes, 1);
    bytes.extend_from_slice(&ONE_ZERO_CHUNK_DIGEST);
    bytes.extend_from_slice(&[0; 24]);
    bytes.extend_from_slice(&segment_digest);
    push_u64(bytes, 64);
    push_u64(bytes, 145);
    push_u64(bytes, 1);
    bytes.extend_from_slice(&record_checksum);
    bytes.extend_from_slice(&[0; 8]);
}

fn append_layout_entry(
    bytes: &mut Vec<u8>,
    segment_digest: [u8; 32],
    record_checksum: [u8; 32],
) -> Result<(), &'static str> {
    let identity = decode_hex(ONE_ZERO_LAYOUT_ID_HEX)?;
    bytes.push(2);
    bytes.push(0);
    push_u16(bytes, 60);
    bytes.extend_from_slice(&identity);
    bytes.extend_from_slice(&segment_digest);
    push_u64(bytes, 209);
    push_u64(bytes, 364);
    push_u64(bytes, 220);
    bytes.extend_from_slice(&record_checksum);
    bytes.extend_from_slice(&[0; 8]);
    Ok(())
}

fn finish_catalog(mut bytes: Vec<u8>) -> Result<Catalog, &'static str> {
    let checksum = framed_digest(b"KEEP:CATALOG:SUM\0", &bytes)?;
    bytes.extend_from_slice(&checksum);
    let digest = framed_digest(b"KEEP:CATALOG:DIGEST\0", &bytes)?;
    bytes.extend_from_slice(&digest);
    Ok(Catalog {
        bytes,
        digest,
        checksum,
        generation: 1,
    })
}

fn decode_hex(input: &str) -> Result<Vec<u8>, &'static str> {
    let canonical = input
        .strip_suffix('\n')
        .ok_or("hex fixture must end in one LF")?;
    if canonical.contains(char::is_whitespace) {
        return Err("hex fixture contains noncanonical whitespace");
    }
    let mut pairs = canonical.as_bytes().chunks_exact(2);
    let mut decoded = Vec::with_capacity(pairs.len());
    for pair in &mut pairs {
        let [high, low] = pair else {
            return Err("hex pair width is not two");
        };
        decoded.push((hex_nibble(*high)? << 4) | hex_nibble(*low)?);
    }
    if !pairs.remainder().is_empty() {
        return Err("hex fixture has odd length");
    }
    Ok(decoded)
}

fn hex_nibble(byte: u8) -> Result<u8, &'static str> {
    const ERROR: &str = "hex fixture contains a nonhexadecimal byte";

    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0').ok_or(ERROR),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|digit| digit.checked_add(10))
            .ok_or(ERROR),
        _ => Err(ERROR),
    }
}
