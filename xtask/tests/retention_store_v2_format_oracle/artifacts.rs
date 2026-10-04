// This included source owns handwritten construction of every golden artifact.

const PROFILE_DOMAIN: &[u8] = b"keep.retention-realization-profile/v1\0";
const DEFINITION_DOMAIN: &[u8] = b"keep.segment-store-definition/v2\0";
const INVENTORY_DOMAIN: &[u8] = b"keep.store-v1-pool-inventory/v2\0";
const STORE_ID_DOMAIN: &[u8] = b"keep.store-identifier/v2\0";

fn build_corpus() -> Result<Corpus, String> {
    let profile_digest = hash(PROFILE_DOMAIN, &[PROFILE_DEFINITION.as_bytes()]);
    let definition_digest = hash(DEFINITION_DOMAIN, &[FORMAT_DEFINITION.as_bytes()]);
    let inventory = build_inventory()?;
    let migration = migration_source(definition_digest, inventory.digest)?;
    let format = build_format_marker(definition_digest)?;
    let intent = build_migration_intent(&migration)?;
    let receipt = build_migration_receipt(&migration, &format, &intent)?;
    let root = build_retention_root(profile_digest)?;
    let manifest = build_retention_manifest(&root)?;
    let head = build_retention_head(&manifest)?;
    let artifacts = vec![
        format,
        intent,
        receipt,
        Artifact {
            case_name: "one-anchor-root",
            kind: "retention-root",
            generation: "1",
            entry_count: "1",
            bound_digest: root.digest,
            final_checksum: root.checksum,
            fixture: "one-anchor-root.hex",
            bytes: root.bytes,
        },
        Artifact {
            case_name: "one-root-manifest",
            kind: "retention-manifest",
            generation: "1",
            entry_count: "1",
            bound_digest: manifest.digest,
            final_checksum: manifest.checksum,
            fixture: "one-root-manifest.hex",
            bytes: manifest.bytes,
        },
        head,
    ];
    Ok(Corpus {
        profile_digest,
        definition_digest,
        inventory,
        migration,
        artifacts,
    })
}

fn build_inventory() -> Result<Inventory, String> {
    let segment = decode_hex(V1_SEGMENT)?;
    let catalog = decode_hex(V1_CATALOG)?;
    require_length(&segment, 337, "version-1 source segment")?;
    require_length(&catalog, 352, "version-1 source catalog")?;
    let segment_digest = array_32(&segment, 273)?;
    let catalog_digest = array_32(&catalog, 320)?;
    let mut rows = vec![
        inventory_row(1, 0, &segment, segment_digest, "one-zero-segment.hex")?,
        inventory_row(2, 1, &catalog, catalog_digest, "one-zero-catalog.hex")?,
    ];
    rows.sort_by_key(|row| row.bytes);
    let entry_count =
        u32::try_from(rows.len()).map_err(|_| "inventory entry count overflow".to_owned())?;
    let mut count = entry_count.to_be_bytes().to_vec();
    for row in &rows {
        count.extend_from_slice(&row.bytes);
    }
    let digest = hash(INVENTORY_DOMAIN, &[&count]);
    Ok(Inventory { rows, digest })
}

fn inventory_row(
    kind: u8,
    generation: u64,
    artifact: &[u8],
    digest: [u8; 32],
    source_fixture: &'static str,
) -> Result<InventoryRow, String> {
    let byte_length =
        u64::try_from(artifact.len()).map_err(|_| "inventory length overflow".to_owned())?;
    let mut bytes = Vec::with_capacity(56);
    bytes.push(kind);
    bytes.extend_from_slice(&[0; 7]);
    push_u64(&mut bytes, generation);
    push_u64(&mut bytes, byte_length);
    bytes.extend_from_slice(&digest);
    require_length(&bytes, 56, "migration inventory entry")?;
    let kind_name = match kind {
        1 => "segment",
        2 => "catalog",
        _ => return Err("unregistered inventory artifact kind".to_owned()),
    };
    Ok(InventoryRow {
        kind: kind_name,
        generation,
        byte_length,
        artifact_digest: digest,
        source_fixture,
        bytes: <[u8; 56]>::try_from(bytes)
            .map_err(|_| "inventory entry conversion failed".to_owned())?,
    })
}

fn migration_source(
    definition_digest: [u8; 32],
    inventory_digest: [u8; 32],
) -> Result<MigrationSource, String> {
    let head = decode_hex(V1_HEAD)?;
    let catalog = decode_hex(V1_CATALOG)?;
    require_length(&head, 128, "version-1 source head")?;
    let catalog_generation = u64_at(&head, 24)?;
    let catalog_length = u64_at(&head, 32)?;
    let catalog_digest = array_32(&head, 40)?;
    let predecessor_digest = array_32(&catalog, 32)?;
    let store_id = hash(
        STORE_ID_DOMAIN,
        &[
            &catalog_generation.to_be_bytes(),
            &catalog_length.to_be_bytes(),
            &catalog_digest,
            &predecessor_digest,
            &inventory_digest,
            &definition_digest,
        ],
    );
    Ok(MigrationSource {
        catalog_generation,
        catalog_length,
        catalog_digest,
        predecessor_digest,
        inventory_digest,
        definition_digest,
        store_id,
        root_device: 1,
        root_mount: 2,
        root_file: 3,
    })
}

include!("artifacts/migration.rs");
include!("artifacts/retention.rs");
