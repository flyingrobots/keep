// This included source owns assertions over constructed bytes and canonical tables.

#[test]
fn golden_artifacts_match_the_independent_oracle() -> Result<(), String> {
    let corpus = build_corpus()?;
    let expected_manifest = artifacts_table(&corpus.artifacts)?;
    assert_eq!(
        read_corpus_file("artifacts.tsv").map_err(|error| error.to_string())?,
        expected_manifest
    );
    for artifact in &corpus.artifacts {
        let fixture =
            read_corpus_file(artifact.fixture).map_err(|error| error.to_string())?;
        assert_eq!(
            fixture,
            encode_hex(&artifact.bytes)?,
            "golden fixture drifted: {}",
            artifact.fixture
        );
    }
    Ok(())
}

#[test]
fn definition_and_profile_digests_match_the_oracle() -> Result<(), String> {
    let corpus = build_corpus()?;
    let profile_hex = encode_digest(&corpus.profile_digest)?;
    let definition_hex = encode_digest(&corpus.definition_digest)?;
    assert!(
        FORMAT_DEFINITION.contains(&format!("retention.profile.digest\t{profile_hex}")),
        "format definition does not bind the exact profile digest"
    );
    for documentation in ["README.md", "ORIGIN.md"] {
        assert!(
            read_corpus_file(documentation)
                .map_err(|error| error.to_string())?
                .contains(&definition_hex),
            "{documentation} does not name the exact format-definition digest"
        );
    }
    Ok(())
}

#[test]
fn definition_and_profile_tables_are_exact_and_canonical() -> Result<(), String> {
    assert_eq!(
        PROFILE_DEFINITION,
        "keep.retention-realization-profiles/v1\n\
         identity\tversion\tcanonical_name\twitness_count\tselection\n\
         1\t1\tkeep.retention-single-canonical-witness/v1\t1\t\
         canonical-physical-catalog-coordinate\n"
    );
    assert!(FORMAT_DEFINITION.ends_with('\n'));
    assert!(!FORMAT_DEFINITION.contains('\r'));
    let mut rows = FORMAT_DEFINITION.lines();
    assert_eq!(rows.next(), Some("keep.segment-store.definition/v2"));
    assert_eq!(rows.next(), Some("key\tvalue"));
    let mut previous: Option<&str> = None;
    for row in rows {
        let (key, value) = row
            .split_once('\t')
            .ok_or_else(|| format!("definition row lacks one key/value boundary: {row}"))?;
        assert!(!key.is_empty(), "definition row has an empty key");
        assert!(!value.is_empty(), "definition row has an empty value");
        assert!(
            !value.contains('\t'),
            "definition row has more than one key/value boundary: {row}"
        );
        if let Some(prior) = previous {
            assert!(prior < key, "definition keys are not strictly sorted");
        }
        previous = Some(key);
    }
    Ok(())
}

#[test]
fn retention_anchor_ids_are_derived_from_the_accepted_layout_corpus() -> Result<(), String> {
    let row = LAYOUTS
        .lines()
        .find(|line| line.starts_with("one-zero\t"))
        .ok_or_else(|| "layout corpus lacks the one-zero case".to_owned())?;
    let fields: Vec<&str> = row.split('\t').collect();
    let blob_id = fields
        .get(5)
        .ok_or_else(|| "one-zero layout row lacks BlobId text".to_owned())?;
    assert_eq!(
        *blob_id,
        "keep:blob:v1:blake3-256:1:\
         1cfb8fa9e917aba15a1f592095f377ff180755fe1212b0d7d2ec750bd128b606"
    );
    let blob_digest = blob_id
        .rsplit(':')
        .next()
        .ok_or_else(|| "one-zero BlobId lacks a digest".to_owned())?;
    assert_eq!(
        BLOB_ID.get(..16),
        Some(b"KEEP:BLOB:ID\0\0\0\0".as_slice())
    );
    assert_eq!(BLOB_ID.get(16..18), Some([0_u8, 1].as_slice()));
    assert_eq!(BLOB_ID.get(18), Some(&1));
    assert_eq!(u64_at(&BLOB_ID, 19)?, 1);
    assert_eq!(
        BLOB_ID.get(27..),
        Some(decode_hex(&format!("{blob_digest}\n"))?.as_slice())
    );
    let layout_id_binary = encode_hex(&LAYOUT_ID)?;
    assert_eq!(
        fields.get(11).copied(),
        Some(layout_id_binary.trim_end())
    );
    Ok(())
}

fn artifacts_table(artifacts: &[Artifact]) -> Result<String, String> {
    let mut table = String::from(
        "keep.segment-store-v2.artifacts/v1\n\
         case\tkind\tbyte_length\tgeneration\tentry_count\tbound_digest_hex\t\
         final_checksum_hex\tfixture\n",
    );
    for artifact in artifacts {
        writeln!(
            table,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            artifact.case_name,
            artifact.kind,
            artifact.bytes.len(),
            artifact.generation,
            artifact.entry_count,
            encode_digest(&artifact.bound_digest)?,
            encode_digest(&artifact.final_checksum)?,
            artifact.fixture
        )
        .map_err(|_| "artifact table formatting failed".to_owned())?;
    }
    Ok(table)
}


fn encode_digest(digest: &[u8; 32]) -> Result<String, String> {
    encode_hex(digest).map(|encoded| encoded.trim_end().to_owned())
}
