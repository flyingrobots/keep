//! Native APFS laws for root coordinates and exact protocol-entry spelling.

use std::{error::Error, fs, io, path::Path, thread};

use cap_fs_ext::MetadataExt;
use cap_std::{ambient_authority, fs::OpenOptions};

use super::{DARWIN_COORDINATE_DOMAIN, DarwinFilesystem};
use crate::adapters::filesystem_directory::FilesystemDirectory;
use crate::adapters::filesystem_entry_spelling::EntrySpellingMismatch;
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::platform_filesystem::PlatformFilesystem;

fn open(path: &Path) -> io::Result<FilesystemDirectory> {
    FilesystemDirectory::open_ambient_dir(path, ambient_authority())
}

fn read_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true);
    options
}

/// Observes this run's volume, so one law states both APFS naming variants.
fn folds_case(root: &Path) -> io::Result<bool> {
    fs::write(root.join("variant-probe"), [])?;
    let folds = root.join("VARIANT-PROBE").try_exists()?;
    fs::remove_file(root.join("variant-probe"))?;
    Ok(folds)
}

fn assert_alias_refused(result: io::Result<cap_std::fs::File>, expected: &str) {
    let source = result.err();
    let mismatch = source
        .as_ref()
        .and_then(io::Error::get_ref)
        .and_then(|inner| inner.downcast_ref::<EntrySpellingMismatch>());
    assert_eq!(
        mismatch,
        Some(&EntrySpellingMismatch {
            expected: expected.into()
        })
    );
}

#[test]
fn apfs_root_coordinates_bind_the_opened_directory() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-root-coordinate")?;
    let path = fs::canonicalize(sandbox.path())?;
    let directory = DarwinFilesystem.open_root(&path)?;
    let identity = DarwinFilesystem.admit_directory(&directory)?;
    let metadata = directory.dir_metadata()?;
    assert_eq!(identity.file(), metadata.ino());
    assert_eq!(identity.device(), DARWIN_COORDINATE_DOMAIN | metadata.dev());
    assert_eq!(identity.mount(), identity.device());
    assert_eq!(identity, DarwinFilesystem.root_identity(&directory)?);
    Ok(())
}

#[test]
fn symbolic_root_components_are_refused() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-symbolic-root")?;
    let root = fs::canonicalize(sandbox.path())?;
    let real = root.join("real");
    let alias = root.join("alias");
    fs::create_dir(&real)?;
    std::os::unix::fs::symlink(&real, &alias)?;
    let result = DarwinFilesystem.open_root(&alias);
    assert!(matches!(result, Err(source)
        if source.raw_os_error() == Some(rustix::io::Errno::NOTDIR.raw_os_error())));
    Ok(())
}

#[test]
fn a_foreign_spelling_never_answers_for_a_protocol_name() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-case-alias")?;
    let folds = folds_case(sandbox.path())?;
    fs::write(sandbox.path().join("head"), b"foreign")?;
    let directory = open(sandbox.path())?;
    let result = directory.open_with("HEAD", &read_options());
    if folds {
        assert_alias_refused(result, "HEAD");
    } else {
        assert!(matches!(result, Err(source) if source.kind() == io::ErrorKind::NotFound));
    }
    Ok(())
}

#[test]
fn a_non_ascii_fold_never_answers_for_a_protocol_name() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-unicode-alias")?;
    // U+017F LATIN SMALL LETTER LONG S folds to ASCII `s` under full case folding.
    fs::write(sandbox.path().join("\u{17f}tage"), b"foreign")?;
    let directory = open(sandbox.path())?;
    let result = directory.open_with("stage", &read_options());
    if sandbox.path().join("stage").try_exists()? {
        assert_alias_refused(result, "stage");
    } else {
        assert!(matches!(result, Err(source) if source.kind() == io::ErrorKind::NotFound));
    }
    Ok(())
}

#[test]
fn mutation_through_a_foreign_spelling_leaves_the_entry_unchanged() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-mutation-alias")?;
    let folds = folds_case(sandbox.path())?;
    fs::write(sandbox.path().join("head"), b"foreign")?;
    let directory = open(sandbox.path())?;
    let removal = directory.remove_file("HEAD");
    assert!(removal.is_err());
    if folds {
        assert_eq!(
            removal.err().map(|source| source.kind()),
            Some(io::ErrorKind::InvalidData)
        );
    }
    assert_eq!(fs::read(sandbox.path().join("head"))?, b"foreign");
    Ok(())
}

#[test]
fn hard_linked_publication_names_stay_admitted_while_another_link_is_open()
-> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-hard-link")?;
    fs::write(sandbox.path().join("stage"), b"published")?;
    fs::hard_link(sandbox.path().join("stage"), sandbox.path().join("HEAD"))?;
    let directory = open(sandbox.path())?;
    let held = directory.open_with("stage", &read_options())?;
    let published = directory.open_with("HEAD", &read_options())?;
    assert_eq!(held.metadata()?.ino(), published.metadata()?.ino());
    directory.open_with("stage", &read_options())?;
    Ok(())
}

#[test]
fn absent_names_remain_available_to_create() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("darwin-absent")?;
    let directory = open(sandbox.path())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    directory.open_with("FORMAT", &options)?;
    assert_eq!(fs::read_dir(sandbox.path())?.count(), 1);
    Ok(())
}

#[test]
fn replacing_an_entry_does_not_refuse_a_concurrent_exact_lookup() -> Result<(), Box<dyn Error>> {
    const REPLACEMENTS: usize = 5_000;
    let sandbox = TestDirectory::create("darwin-replace-race")?;
    let root = sandbox.path().to_owned();
    for index in 0..64 {
        fs::write(root.join(format!("neighbour-{index}")), [])?;
    }
    fs::write(root.join("HEAD"), b"0")?;
    let directory = open(&root)?;
    let publisher = thread::spawn(move || -> io::Result<()> {
        for _ in 0..REPLACEMENTS {
            fs::write(root.join("HEAD.next"), b"1")?;
            fs::rename(root.join("HEAD.next"), root.join("HEAD"))?;
        }
        Ok(())
    });
    while !publisher.is_finished() {
        directory.open_with("HEAD", &read_options())?;
    }
    publisher
        .join()
        .map_err(|_| io::Error::other("publisher panicked"))??;
    Ok(())
}

#[test]
fn filesystems_other_than_apfs_are_refused() -> Result<(), Box<dyn Error>> {
    // devfs is always mounted and never writable through this adapter.
    let directory = DarwinFilesystem.open_root(Path::new("/dev"))?;
    let refusal = DarwinFilesystem.admit_directory(&directory);
    assert!(matches!(refusal, Err(source) if source.kind() == io::ErrorKind::Unsupported));
    Ok(())
}

#[test]
fn read_only_apfs_volumes_are_refused() -> Result<(), Box<dyn Error>> {
    // The sealed system volume is APFS and mounted read-only.
    let directory = DarwinFilesystem.open_root(Path::new("/System"))?;
    let refusal = DarwinFilesystem.admit_directory(&directory);
    assert!(matches!(refusal, Err(source) if source.kind() == io::ErrorKind::Unsupported));
    Ok(())
}
