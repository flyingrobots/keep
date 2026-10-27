//! Initialization, format-marker, and recovery written-contract laws.

use super::{FORMAT_ROOT, normalized, read};

#[test]
fn initialization_and_recovery_define_every_authority_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    // recovery.md owns the namespace, marker, fence, and portability laws;
    // initialization.md owns the ordered protocol, its states, and its
    // process-death rows. Retention restart has its own page. These are
    // documentation guards, not executable evidence for the storage claims
    // written in the pages.
    let recovery = format!(
        "{}\n{}\n{}",
        normalized(&read(&format!("{FORMAT_ROOT}/recovery.md"))?),
        normalized(&read(&format!("{FORMAT_ROOT}/initialization.md"))?),
        normalized(&read(&format!("{FORMAT_ROOT}/retention-recovery.md"))?)
    );

    for required in [
        "direct initialization",
        "`FORMAT.next`",
        "commit point",
        "`AlreadyInitialized`",
        "`InitializationIncomplete`",
        "`MissingFormatMarker`",
        "`definition.tsv`",
        "keep.store-format-marker/v2\\0",
        "never means \"nothing is retained\"",
        "root.next` is durable before a new namespace directory",
        "`KEEP-CRASH-036`",
        "`KEEP-CRASH-081`",
        "`KEEP-CRASH-091`",
        "Version-1 admission refuses",
        "`reader.lock`",
        "`GcRetirementIntent`",
        "`GcRetirementReceipt`",
        "`RecoveryDispositionReceipt`",
        "unknown entry",
        "unrecoverable ambiguity",
        "idempotent",
        "process death",
        "records no device, inode, mount, or path",
    ] {
        assert!(
            recovery.contains(required),
            "segment-store v2 initialization and recovery pages omit `{required}`"
        );
    }
    Ok(())
}

#[test]
fn initialization_never_writes_the_marker_in_place() -> Result<(), Box<dyn std::error::Error>> {
    let initialization = normalized(&read(&format!("{FORMAT_ROOT}/initialization.md"))?);

    for required in [
        "never writes `FORMAT` in place",
        "`FORMAT.next`",
        "without replacement",
        "strict prefix",
        "`KEEP-CRASH-081`",
        "`KEEP-CRASH-091`",
        "before, during, and after",
    ] {
        assert!(
            initialization.contains(required),
            "segment-store v2 initialization protocol omits `{required}`"
        );
    }
    Ok(())
}
