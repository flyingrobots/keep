//! This module owns identity and hasher seed derivation from canonical vectors.

use std::collections::BTreeSet;
use std::path::Path;

use super::filesystem::RepositoryFiles;
use super::{FuzzSeedError, MAX_SEED_BYTES, Seed};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex, framed_lines, tab_fields};

const IDENTITIES: &str = "conformance/golden-file-worldline/v1/identities.tsv";
const SCHEMA: &str = "# keep.golden-file-worldline.identities/v1";
const COLUMNS: &str = "case\tsource_kind\tsource_parameter\trepetitions\tlogical_length\tcanonical_text\tcanonical_binary_hex";
const REQUIRED: [&str; 3] = ["empty", "small-text", "large-ramp"];

pub(super) fn seeds(files: &RepositoryFiles) -> Result<Vec<Seed>, FuzzSeedError> {
    let raw = files.read_bounded(Path::new(IDENTITIES), MAX_SEED_BYTES)?;
    let lines = framed_lines(&raw, MAX_SEED_BYTES).map_err(|source| {
        FuzzSeedError::violation(format!("identities framing moved: {source}"))
    })?;
    let rows = admitted_rows(lines)?;
    let mut seeds = identity_parser_seeds(&rows)?;
    seeds.push(Seed::new("blob_hasher", "empty", Vec::new())?);
    seeds.push(Seed::new("blob_hasher", "byte-ramp", byte_ramp())?);
    Ok(seeds)
}

fn admitted_rows(lines: Vec<String>) -> Result<Vec<[String; 7]>, FuzzSeedError> {
    let mut lines = lines.into_iter();
    if lines.next().as_deref() != Some(SCHEMA) || lines.next().as_deref() != Some(COLUMNS) {
        return Err(FuzzSeedError::violation(
            "identities.tsv schema or columns moved",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for line in lines {
        let fields: [&str; 7] = tab_fields(&line, 7)
            .map_err(|_| FuzzSeedError::violation("identity seed row has invalid field count"))?
            .try_into()
            .map_err(|_| FuzzSeedError::violation("identity seed row has invalid field count"))?;
        if !seen.insert(fields[0].to_owned()) {
            return Err(FuzzSeedError::violation(
                "identity seed row identifier is duplicated",
            ));
        }
        rows.push(fields.map(str::to_owned));
    }
    Ok(rows)
}

fn identity_parser_seeds(rows: &[[String; 7]]) -> Result<Vec<Seed>, FuzzSeedError> {
    let mut seeds = Vec::new();
    for name in REQUIRED {
        let fields = rows
            .iter()
            .find(|fields| fields.first().map(String::as_str) == Some(name))
            .ok_or_else(|| {
                FuzzSeedError::violation(format!("required identity {name:?} is absent"))
            })?;
        let text = fields
            .get(5)
            .ok_or_else(|| FuzzSeedError::violation("identity text field is absent"))?;
        if !text.is_ascii() {
            return Err(FuzzSeedError::violation(
                "canonical identity text is not ASCII",
            ));
        }
        let binary = decode_lower_hex(
            fields
                .get(6)
                .ok_or_else(|| FuzzSeedError::violation("identity binary field is absent"))?,
            MAX_SEED_BYTES,
            EmptyHex::Refuse,
        )
        .map_err(|source| {
            FuzzSeedError::violation(format!("canonical identity binary is invalid: {source}"))
        })?;
        seeds.push(Seed::new("blob_id_text", name, text.as_bytes().to_vec())?);
        seeds.push(Seed::new("blob_id_binary", name, binary)?);
    }
    Ok(seeds)
}

fn byte_ramp() -> Vec<u8> {
    let mut content = Vec::with_capacity(4_096);
    for _ in 0..16 {
        content.extend(u8::MIN..=u8::MAX);
    }
    content
}
