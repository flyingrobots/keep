// This included source owns construction of the format marker and migration records.

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

fn build_migration_intent(source: &MigrationSource) -> Result<Artifact, String> {
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(b"KEEP:MIG:INT2\0\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 256);
    push_u32(&mut bytes, 0);
    push_u64(&mut bytes, source.catalog_generation);
    push_u64(&mut bytes, source.catalog_length);
    bytes.extend_from_slice(&source.catalog_digest);
    bytes.extend_from_slice(&source.predecessor_digest);
    bytes.extend_from_slice(&source.inventory_digest);
    push_u64(&mut bytes, source.root_device);
    push_u64(&mut bytes, source.root_mount);
    push_u64(&mut bytes, source.root_file);
    bytes.extend_from_slice(&source.definition_digest);
    bytes.extend_from_slice(&source.store_id);
    require_length(&bytes, 224, "migration-intent checksum preimage")?;
    let checksum = hash(b"keep.store-migration-intent-checksum/v2\0", &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 256, "migration intent")?;
    let intent_digest = hash(b"keep.store-migration-intent/v2\0", &[&bytes]);
    Ok(Artifact {
        case_name: "migration-intent",
        kind: "migration-intent",
        generation: "1",
        entry_count: "2",
        bound_digest: intent_digest,
        final_checksum: checksum,
        fixture: "migration-intent.hex",
        bytes,
    })
}

fn build_migration_receipt(
    source: &MigrationSource,
    format: &Artifact,
    intent: &Artifact,
) -> Result<Artifact, String> {
    let initial_retention = hash(b"keep.initial-retention-state/v2\0", &[]);
    let initial_gc = hash(b"keep.initial-gc-state/v2\0", &[]);
    let empty_dispositions = hash(b"keep.empty-disposition-set/v2\0", &[]);
    let mut bytes = Vec::with_capacity(256);
    bytes.extend_from_slice(b"KEEP:MIG:REC2\0\0\0");
    push_u16(&mut bytes, 2);
    push_u16(&mut bytes, 256);
    push_u32(&mut bytes, 0);
    bytes.extend_from_slice(&intent.bound_digest);
    bytes.extend_from_slice(&source.store_id);
    bytes.extend_from_slice(&format.bound_digest);
    bytes.extend_from_slice(&initial_retention);
    bytes.extend_from_slice(&initial_gc);
    bytes.extend_from_slice(&empty_dispositions);
    push_u64(&mut bytes, 0x03ff);
    require_length(&bytes, 224, "migration-receipt checksum preimage")?;
    let checksum = hash(b"keep.store-migration-receipt-checksum/v2\0", &[&bytes]);
    bytes.extend_from_slice(&checksum);
    require_length(&bytes, 256, "migration receipt")?;
    Ok(Artifact {
        case_name: "migration-receipt",
        kind: "migration-receipt",
        generation: "1",
        entry_count: "2",
        bound_digest: intent.bound_digest,
        final_checksum: checksum,
        fixture: "migration-receipt.hex",
        bytes,
    })
}
