//! Written-contract evidence for the version-2 retention store.

#![cfg(feature = "repository-tasks")]

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
fn migration_and_recovery_define_every_authority_boundary() -> Result<(), Box<dyn std::error::Error>>
{
    let recovery = normalized(&read(&format!("{FORMAT_ROOT}/recovery.md"))?);

    for required in [
        "one-way explicit migration",
        "`migration.intent`",
        "`migration.intent.next`",
        "`migration.receipt`",
        "`migration.receipt.next`",
        "`FORMAT.next`",
        "`migration.intent` is exactly 256 bytes",
        "`migration.receipt` is exactly 256 bytes",
        "catalog generation, length, and digest",
        "`definition.tsv`",
        "migration inventory entry is exactly 56 bytes",
        "2,097,152",
        "keep.store-migration-intent/v2\\0",
        "keep.store-format-marker/v2\\0",
        "deterministically derived store identifier",
        "absence of `retention/HEAD` is the canonical empty retention state",
        "pre-effect incomplete stage",
        "keep.initial-retention-state/v2\\0",
        "keep.initial-gc-state/v2\\0",
        "keep.empty-disposition-set/v2\\0",
        "root.next` is durable before a new namespace directory",
        "`KEEP-CRASH-036`",
        "`KEEP-CRASH-073`",
        "partial migration",
        "Version-1 admission refuses",
        "`reader.lock`",
        "`GcRetirementIntent`",
        "`GcRetirementReceipt`",
        "`RecoveryDispositionReceipt`",
        "unknown entry",
        "unrecoverable ambiguity",
        "idempotent",
        "process death",
    ] {
        assert!(
            recovery.contains(required),
            "segment-store v2 recovery contract omits `{required}`"
        );
    }
    Ok(())
}

#[test]
fn migration_never_writes_canonical_fixed_names_in_place() -> Result<(), Box<dyn std::error::Error>>
{
    let migration = normalized(&read(&format!("{FORMAT_ROOT}/migration-crash.md"))?);

    for required in [
        "never writes canonical fixed names in place",
        "`migration.intent.next`",
        "`FORMAT.next`",
        "`migration.receipt.next`",
        "linked without replacement",
        "pre-effect incomplete stage",
        "`KEEP-CRASH-053`",
        "`KEEP-CRASH-073`",
        "`0x00000000000003ff`",
        "before, during, and after process-death evidence",
    ] {
        assert!(
            migration.contains(required),
            "segment-store v2 migration crash protocol omits `{required}`"
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
