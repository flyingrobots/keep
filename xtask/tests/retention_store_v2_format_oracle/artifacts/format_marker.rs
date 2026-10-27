// This included source owns construction of the format marker.

fn build_format_marker(definition_digest: [u8; 32]) -> Result<Artifact, String> {
    let mut bytes = Vec::with_capacity(96);
    bytes.extend_from_slice(b"KEEP:STORE:V2\0\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 96);
    push_u32(&mut bytes, 0);
    bytes.extend_from_slice(&definition_digest);
    push_u32(&mut bytes, 4_096);
    push_u32(&mut bytes, 0);
    require_length(&bytes, 64, "format-marker checksum preimage")?;
    let checksum = hash(b"keep.segment-store-marker-checksum/v2\0", &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 96, "format marker")?;
    let marker_digest = hash(b"keep.store-format-marker/v2\0", &[&bytes]);
    Ok(Artifact {
        case_name: "format-marker",
        kind: "format-marker",
        generation: "-",
        entry_count: "-",
        bound_digest: marker_digest,
        final_checksum: checksum,
        fixture: "format-marker.hex",
        bytes,
    })
}
