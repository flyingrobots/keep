//! These laws own typed refusal of misdispatched migration phases.
//! Size: medium (owned filesystem). Oracle: each section executes only its own
//! phases and names a foreign phase as typed evidence, without touching storage.
//! Delete when phase dispatch stops being split into sections.

use std::error::Error;
use std::io;

use super::super::filesystem_migration_test_fixture::open_authority;
use super::super::{FilesystemMigrationRefusal, StoreMigrationPhase};
use super::{MigrationRecords, intent_and_namespace_phase, marker_phase, receipt_phase};

fn section_refusal(result: io::Result<()>) -> Option<(io::ErrorKind, FilesystemMigrationRefusal)> {
    let error = result.err()?;
    let refusal = error
        .get_ref()?
        .downcast_ref::<FilesystemMigrationRefusal>()
        .copied()?;
    Some((error.kind(), refusal))
}

const fn misdispatched(
    observed: StoreMigrationPhase,
) -> Option<(io::ErrorKind, FilesystemMigrationRefusal)> {
    Some((
        io::ErrorKind::InvalidInput,
        FilesystemMigrationRefusal::PhaseSection { observed },
    ))
}

#[test]
fn a_section_refuses_a_foreign_phase_by_naming_it() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("migration-refusal-phase-section")?;
    let intent = authority.observe_intent()?;
    let records = MigrationRecords::for_intent(&intent);
    let receipt_only = StoreMigrationPhase::WriteReceiptStage;
    let intent_only = StoreMigrationPhase::WriteIntentStage;

    assert_eq!(
        section_refusal(intent_and_namespace_phase(&mut authority, receipt_only, &intent)),
        misdispatched(receipt_only)
    );
    assert_eq!(
        section_refusal(marker_phase(&mut authority, intent_only, &records.marker)),
        misdispatched(intent_only)
    );
    assert_eq!(
        section_refusal(receipt_phase(&mut authority, intent_only, &records.receipt)),
        misdispatched(intent_only)
    );
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
