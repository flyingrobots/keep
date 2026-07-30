//! Written-contract evidence for the version-2 retention store.

#![cfg(feature = "repository-tasks")]

#[path = "retention_store_v2_protocol_contract/closure_contract_laws.rs"]
mod closure_contract_laws;
#[path = "retention_store_v2_protocol_contract/migration_contract_laws.rs"]
mod migration_contract_laws;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const FORMAT_ROOT: &str = "docs/formats/segment-store-v2";
const DOCUMENT_REVIEW_LIMIT_LINES: usize = 300;

fn repository_root() -> Result<PathBuf, io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("xtask manifest directory has no parent"))
}

fn read(relative: &str) -> Result<String, io::Error> {
    fs::read_to_string(repository_root()?.join(relative))
}

fn normalized(document: &str) -> String {
    document.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn version_two_is_one_routed_protocol() -> Result<(), Box<dyn std::error::Error>> {
    let format_index = read("docs/formats/README.md")?;
    let changelog = read("CHANGELOG.md")?;
    let overview = normalized(&read(&format!("{FORMAT_ROOT}/README.md"))?);

    assert!(
        format_index.contains("segment-store-v2/README.md"),
        "format index does not route to segment-store v2"
    );
    assert!(
        changelog.contains("`keep.segment-store/v2`"),
        "changelog does not record the segment-store v2 contract"
    );
    for required in [
        "`keep.segment-store/v2`",
        "successor to `keep.segment-store/v1`",
        "[Retention records and publication](retention.md)",
        "[Closure verification](closure.md)",
        "[Closure corruption boundary](closure-corruption.md)",
        "[GC and disposition records](gc.md)",
        "[Migration and recovery](recovery.md)",
        "[Migration crash points](migration-crash.md)",
        "[Migration inventory](migration-inventory.md)",
        "[Requirements and evidence](requirements.md)",
        "[Format rationale](rationale.md)",
    ] {
        assert!(
            overview.contains(required),
            "segment-store v2 overview omits `{required}`"
        );
    }
    Ok(())
}

#[test]
fn retention_records_have_exact_canonical_grammars() -> Result<(), Box<dyn std::error::Error>> {
    let retention = normalized(&read(&format!("{FORMAT_ROOT}/retention.md"))?);

    for required in [
        "`RetentionNamespace`",
        "1 through 255 bytes",
        "`RootGeneration`",
        "`LivenessGeneration`",
        "big-endian",
        "fixed-width header",
        "sorted, duplicate-free",
        "`retention-profile.tsv`",
        "keep.retention-manifest-entries/v2\\0",
        "entry-count-u32",
        "BLAKE3-256",
        "domain-separated",
        "trailing bytes",
        "unknown mandatory flags",
        "maximum admitted namespace count",
        "before any namespace-generation or manifest bytes are staged",
        "expected and observed generations",
        "already committed",
        "one complete root generation",
        "remain durable until the retention head",
        "double-collects the catalog and retention heads",
        "same coordinates before and after",
    ] {
        assert!(
            retention.contains(required),
            "segment-store v2 retention grammar omits `{required}`"
        );
    }
    Ok(())
}

#[test]
fn gc_records_are_bounded_before_their_implementation() -> Result<(), Box<dyn std::error::Error>> {
    let gc = normalized(&read(&format!("{FORMAT_ROOT}/gc.md"))?);

    for required in [
        "`GcRetirementIntent`",
        "320-byte fixed-width header",
        "72-byte candidate entries",
        "65,536",
        "`GcRetirementReceipt`",
        "exactly 320 bytes",
        "`RecoveryDispositionReceipt`",
        "canonical absent candidate prefix",
        "unrecoverable ambiguity",
        "Planned in #21",
    ] {
        assert!(
            gc.contains(required),
            "segment-store v2 GC grammar omits `{required}`"
        );
    }
    Ok(())
}

#[test]
fn requirement_ledger_names_planned_and_executable_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let requirements = normalized(&read(&format!("{FORMAT_ROOT}/requirements.md"))?);

    for required in [
        "`KEEP-RETENTION-001`",
        "`KEEP-RETENTION-010`",
        "`KEEP-MIGRATION-001`",
        "`KEEP-MIGRATION-008`",
        "`KEEP-GC-001`",
        "Planned in #19",
        "Planned in #21",
        "golden-format",
        "model-based",
        "corruption",
        "crash-injection",
        "fuzz",
    ] {
        assert!(
            requirements.contains(required),
            "segment-store v2 requirement ledger omits `{required}`"
        );
    }
    Ok(())
}

#[test]
fn version_two_pages_stay_within_the_review_threshold() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "README.md",
        "closure-corruption.md",
        "closure.md",
        "gc.md",
        "migration-crash.md",
        "migration-inventory.md",
        "rationale.md",
        "recovery.md",
        "requirements.md",
        "retention.md",
    ] {
        let line_count = read(&format!("{FORMAT_ROOT}/{name}"))?.lines().count();
        assert!(
            line_count <= DOCUMENT_REVIEW_LIMIT_LINES,
            "{name} has {line_count} lines; review threshold is {DOCUMENT_REVIEW_LIMIT_LINES}"
        );
    }
    Ok(())
}
