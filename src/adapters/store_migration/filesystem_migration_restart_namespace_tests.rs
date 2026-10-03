//! Restart refuses unknown names and incorrect filesystem kinds without cleanup.

use super::filesystem_migration_restart_test_fixture::{prefix, refusal};
use super::{
    FilesystemMigrationAuthorityError as Authority, FilesystemMigrationRecoveryRefusal as Refusal,
    FilesystemMigrationResidueKind as Kind, StoreMigrationPhase as Phase,
    StoreMigrationRecoveryError as Recovery,
};
use std::{error::Error, fs, io};

// Size: medium. Oracle: recovery admits only the closed migration namespace.
// Delete only if unknown names become explicitly admitted by a new protocol.
#[test]
fn unknown_root_and_nested_entries_preserve_restart_evidence() -> Result<(), Box<dyn Error>> {
    for parent in [
        "",
        "retention",
        "retention/roots",
        "retention/manifests",
        "gc",
        "recovery",
        "recovery/dispositions",
    ] {
        let store = prefix(
            &format!("restart-unknown-{}", parent.replace('/', "-")),
            Phase::AdmitNamespacePrefix,
        )?;
        fs::write(
            store.path().join(parent).join("unexpected"),
            b"retain this evidence",
        )?;
        let error = refusal(store.path())?;
        if parent.is_empty() {
            require_namespace(error.as_ref());
        } else {
            let Some(Recovery::Adoption { source }) = error.downcast_ref::<Recovery>() else {
                return Err(format!("{parent}: wrong refusal boundary: {error:?}").into());
            };
            assert!(
                matches!(source.get_ref().and_then(|source| source.downcast_ref::<Refusal>()),
                Some(Refusal::NamespacePreflight { source }) if source.kind() == io::ErrorKind::InvalidData),
                "{parent}: {error:?}"
            );
        }
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: fixed record and fence names are regular files, never directories.
// Delete only if the named filesystem kinds change by protocol decision.
#[test]
fn directory_substitution_at_each_fixed_file_refuses_restart() -> Result<(), Box<dyn Error>> {
    for name in [
        "migration.intent",
        "migration.intent.next",
        "FORMAT",
        "FORMAT.next",
        "migration.receipt",
        "migration.receipt.next",
        "reader.lock",
    ] {
        let store = prefix(
            &format!("restart-wrong-file-kind-{name}"),
            Phase::RemoveReceiptStage,
        )?;
        let path = store.path().join(name);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        fs::create_dir(path)?;
        let error = refusal(store.path())?;
        require_namespace(error.as_ref());
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: all protocol directories must remain real directories.
// Delete only if migration directory requirements are removed.
#[test]
fn regular_files_at_protocol_directory_names_refuse_restart() -> Result<(), Box<dyn Error>> {
    for name in [
        "retention",
        "gc",
        "recovery",
        "retention/roots",
        "retention/manifests",
        "recovery/dispositions",
    ] {
        let store = prefix(
            &format!("restart-wrong-dir-kind-{}", name.replace('/', "-")),
            Phase::AdmitNamespacePrefix,
        )?;
        let path = store.path().join(name);
        fs::remove_dir_all(&path)?;
        fs::write(path, b"not a directory")?;
        let error = refusal(store.path())?;
        if name.contains('/') {
            assert!(
                matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Observation { source })
                if matches!(source.get_ref().and_then(|source| source.downcast_ref::<Refusal>()),
                    Some(Refusal::NamespaceKind { observed_kind: Kind::RegularFile }))),
                "{name}: {error:?}"
            );
        } else {
            require_namespace(error.as_ref());
        }
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: a reader fence carries no payload; observation names its exact length.
// Delete only if reader-lock payloads acquire defined protocol semantics.
#[test]
fn a_nonempty_reader_fence_preserves_its_exact_refusal() -> Result<(), Box<dyn Error>> {
    let store = prefix("restart-nonempty-reader-fence", Phase::AdmitReaderFence)?;
    fs::write(store.path().join("reader.lock"), b"x")?;
    let error = refusal(store.path())?;
    assert!(
        matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Observation { source })
        if matches!(source.get_ref().and_then(|source| source.downcast_ref::<Refusal>()),
            Some(Refusal::ReaderFence { observed_kind: Kind::RegularFile, observed_length: 1 }))),
        "{error:?}"
    );
    store.remove()?;
    Ok(())
}

fn require_namespace(error: &(dyn Error + 'static)) {
    assert!(
        matches!(error.downcast_ref::<Authority>(), Some(Authority::Namespace { source })
        if source.kind() == io::ErrorKind::InvalidData),
        "namespace refusal lost its typed boundary: {error:?}"
    );
}

// Size: medium. Oracle: migration fixed names never follow substituted symbolic links.
// Delete only if no-follow admission is explicitly replaced by another protocol.
#[test]
fn symbolic_links_at_fixed_record_names_refuse_without_following_them() -> Result<(), Box<dyn Error>>
{
    for name in [
        "migration.intent",
        "migration.intent.next",
        "FORMAT",
        "FORMAT.next",
        "migration.receipt",
        "migration.receipt.next",
        "reader.lock",
    ] {
        let store = prefix(
            &format!("restart-linked-record-{name}"),
            Phase::RemoveReceiptStage,
        )?;
        let path = store.path().join(name);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        std::os::unix::fs::symlink("HEAD", path)?;
        let error = refusal(store.path())?;
        require_namespace(error.as_ref());
        store.remove()?;
    }
    Ok(())
}

// Size: medium. Oracle: completed migration still admits only the reserved v2 namespace.
// Delete only if that namespace contract is removed or stronger restart laws subsume this law.
#[test]
fn completed_migration_refuses_unknown_reserved_entries_without_effects()
-> Result<(), Box<dyn Error>> {
    for parent in ["gc", "recovery", "recovery/dispositions"] {
        let store = prefix(
            &format!("restart-complete-unknown-{}", parent.replace('/', "-")),
            Phase::RemoveReceiptStage,
        )?;
        fs::write(store.path().join(parent).join("unexpected"), b"preserve")?;
        let error = refusal(store.path())?;
        assert!(
            matches!(error.downcast_ref::<Recovery>(), Some(Recovery::Observation { source })
            if matches!(source.get_ref().and_then(|source| source.downcast_ref::<Refusal>()),
                Some(Refusal::NamespacePreflight { source }) if source.kind() == io::ErrorKind::InvalidData)),
            "completed migration admitted invalid {parent}: {error:?}"
        );
        store.remove()?;
    }
    Ok(())
}
