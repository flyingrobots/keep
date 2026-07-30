//! Migration and recovery written-contract laws.

use super::{FORMAT_ROOT, normalized, read};

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
