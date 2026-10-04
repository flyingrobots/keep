// This included source owns construction of retention root, manifest, and head records.

const NAMESPACE: &[u8] = &[0x00, 0x2f, 0xff];
const BLOB_ID: [u8; 59] = [
    0x4b, 0x45, 0x45, 0x50, 0x3a, 0x42, 0x4c, 0x4f, 0x42, 0x3a, 0x49, 0x44, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x1c,
    0xfb, 0x8f, 0xa9, 0xe9, 0x17, 0xab, 0xa1, 0x5a, 0x1f, 0x59, 0x20, 0x95, 0xf3, 0x77,
    0xff, 0x18, 0x07, 0x55, 0xfe, 0x12, 0x12, 0xb0, 0xd7, 0xd2, 0xec, 0x75, 0x0b, 0xd1,
    0x28, 0xb6, 0x06,
];
const LAYOUT_ID: [u8; 60] = [
    0x4b, 0x45, 0x45, 0x50, 0x3a, 0x4c, 0x41, 0x59, 0x4f, 0x55, 0x54, 0x3a, 0x49, 0x44,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc,
    0x88, 0x7d, 0xa2, 0x3f, 0x1a, 0x74, 0x83, 0x35, 0x9a, 0x78, 0xfc, 0x9a, 0x7f, 0xde,
    0x80, 0x03, 0x0e, 0xc2, 0xc4, 0x69, 0x06, 0x03, 0x80, 0x3f, 0x0a, 0xb7, 0xd0, 0xed,
    0xb5, 0x65, 0x75, 0xb8,
];

fn build_retention_root(profile_digest: [u8; 32]) -> Result<RootArtifact, String> {
    let mut anchor = Vec::with_capacity(119);
    anchor.extend_from_slice(&BLOB_ID);
    anchor.extend_from_slice(&LAYOUT_ID);
    require_length(&anchor, 119, "retention anchor")?;
    let anchor_count = 1u32.to_be_bytes();
    let anchor_set_digest = hash(
        b"keep.retention-anchor-set/v2\0",
        &[&anchor_count, &anchor],
    );
    let mut header = root_header(profile_digest, anchor_set_digest)?;
    let mut body = NAMESPACE.to_vec();
    body.extend_from_slice(&anchor);
    let digest = hash(b"keep.retention-root/v2\0", &[&header, &body]);
    let checksum = hash(
        b"keep.retention-root-checksum/v2\0",
        &[&header, &body, &digest],
    );
    header.extend_from_slice(&body);
    header.extend_from_slice(&digest);
    header.extend_from_slice(&checksum);
    require_length(&header, 378, "retention root")?;
    let namespace_length =
        u16::try_from(NAMESPACE.len()).map_err(|_| "namespace length overflow".to_owned())?;
    let namespace_digest = hash(
        b"keep.retention-namespace/v1\0",
        &[&namespace_length.to_be_bytes(), NAMESPACE],
    );
    Ok(RootArtifact {
        bytes: header,
        digest,
        checksum,
        namespace_digest,
    })
}

fn root_header(
    profile_digest: [u8; 32],
    anchor_set_digest: [u8; 32],
) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(192);
    bytes.extend_from_slice(b"KEEP:RET:ROOT2\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 192);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, 378);
    push_u64(&mut bytes, 1);
    push_u16(&mut bytes, 3);
    push_u16(&mut bytes, 119);
    push_u32(&mut bytes, 1);
    push_u32(&mut bytes, 1);
    push_u32(&mut bytes, 1);
    bytes.extend_from_slice(&profile_digest);
    push_u64(&mut bytes, 4);
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 0);
    push_u64(&mut bytes, 4_096);
    push_u64(&mut bytes, 4_096);
    bytes.extend_from_slice(&[0; 32]);
    bytes.extend_from_slice(&anchor_set_digest);
    bytes.extend_from_slice(&[0; 12]);
    require_length(&bytes, 192, "retention-root header")?;
    Ok(bytes)
}

fn build_retention_manifest(root: &RootArtifact) -> Result<ManifestArtifact, String> {
    let mut entry = Vec::with_capacity(72);
    entry.extend_from_slice(&root.namespace_digest);
    push_u64(&mut entry, 1);
    entry.extend_from_slice(&root.digest);
    require_length(&entry, 72, "retention manifest entry")?;
    let entry_count = 1u32.to_be_bytes();
    let entry_set_digest = hash(
        b"keep.retention-manifest-entries/v2\0",
        &[&entry_count, &entry],
    );
    let mut header = manifest_header(entry_set_digest)?;
    let digest = hash(b"keep.retention-manifest/v2\0", &[&header, &entry]);
    let checksum = hash(
        b"keep.retention-manifest-checksum/v2\0",
        &[&header, &entry, &digest],
    );
    header.extend_from_slice(&entry);
    header.extend_from_slice(&digest);
    header.extend_from_slice(&checksum);
    require_length(&header, 296, "retention manifest")?;
    Ok(ManifestArtifact {
        bytes: header,
        digest,
        checksum,
    })
}

fn manifest_header(entry_set_digest: [u8; 32]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(160);
    bytes.extend_from_slice(b"KEEP:RET:LIVE2\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 160);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, 296);
    push_u64(&mut bytes, 1);
    push_u16(&mut bytes, 72);
    push_u16(&mut bytes, 0);
    push_u32(&mut bytes, 1);
    bytes.extend_from_slice(&[0; 32]);
    bytes.extend_from_slice(&entry_set_digest);
    bytes.extend_from_slice(&[0; 48]);
    require_length(&bytes, 160, "retention-manifest header")?;
    Ok(bytes)
}

fn build_retention_head(manifest: &ManifestArtifact) -> Result<Artifact, String> {
    let mut bytes = Vec::with_capacity(144);
    bytes.extend_from_slice(b"KEEP:RET:HEAD2\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 144);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, 1);
    push_u64(&mut bytes, 296);
    bytes.extend_from_slice(&manifest.digest);
    bytes.extend_from_slice(&[0; 32]);
    push_u64(&mut bytes, 0);
    require_length(&bytes, 112, "retention-head checksum preimage")?;
    let checksum = hash(b"keep.retention-head-checksum/v2\0", &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 144, "retention head")?;
    Ok(Artifact {
        case_name: "one-root-head",
        kind: "retention-head",
        generation: "1",
        entry_count: "1",
        bound_digest: manifest.digest,
        final_checksum: checksum,
        fixture: "one-root-head.hex",
        bytes,
    })
}
