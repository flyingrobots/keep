//! Source-layout laws for the adapters tree: the module root stays a scannable
//! manifest, files rustfmt cannot rewrap stay within the standard width, and
//! record readers take their fixed lengths from the decoders that define them,
//! and no module under `src/` spawns a process.

const ADAPTERS_ROOT: &str = include_str!("../src/adapters/mod.rs");
const RETENTION_REFUSAL: &str =
    include_str!("../src/adapters/retention/filesystem_retention_refusal.rs");

/// `docs/Rust Standards.md` reviews any file above 300 lines and refuses any
/// above 500; the adapters root sat at 498 before its re-export surface moved.
#[test]
fn adapters_root_stays_under_the_review_threshold() {
    let lines = ADAPTERS_ROOT.lines().count();
    assert!(
        lines < 300,
        "src/adapters/mod.rs has {lines} lines; the review threshold is 300"
    );
}

/// The root declares modules and re-exports one surface; item definitions
/// belong in their own files.
#[test]
fn adapters_root_declares_modules_and_reexports_only() {
    for line in ADAPTERS_ROOT.lines() {
        let trimmed = line.trim_start();
        let allowed = trimmed.is_empty()
            || trimmed.starts_with("//!")
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("pub mod ")
            || trimmed.starts_with("pub(super) mod ")
            || trimmed.starts_with("pub use ")
            || trimmed.starts_with("use ")
            || trimmed.starts_with("pub(super) use ")
            || trimmed == "};"
            || trimmed.ends_with(',')
            || trimmed.ends_with("::{");
        assert!(allowed, "unexpected item in src/adapters/mod.rs: {line}");
    }
}

/// rustfmt cannot break a string literal, so an overlong `Display` arm hides a
/// 200-column line behind a clean `cargo fmt --check`. The refusal catalogue
/// stays readable at the standard's 100-column width.
#[test]
fn retention_refusal_lines_stay_within_one_hundred_columns() {
    for (index, line) in RETENTION_REFUSAL.lines().enumerate() {
        let width = line.chars().count();
        assert!(
            width <= 100,
            "filesystem_retention_refusal.rs:{} is {width} columns wide",
            index + 1
        );
    }
}

const RECORD_READERS: [(&str, &str); 4] = [
    (
        "src/adapters/retention/filesystem_retention_current.rs",
        include_str!("../src/adapters/retention/filesystem_retention_current.rs"),
    ),
    (
        "src/adapters/retention/filesystem_retention_catalog.rs",
        include_str!("../src/adapters/retention/filesystem_retention_catalog.rs"),
    ),
    (
        "src/adapters/filesystem_version_two_records.rs",
        include_str!("../src/adapters/filesystem_version_two_records.rs"),
    ),
    (
        "src/adapters/store_migration/filesystem_migration_namespace.rs",
        include_str!("../src/adapters/store_migration/filesystem_migration_namespace.rs"),
    ),
];

/// Fixed record lengths belong to the decoders that define the formats. A
/// reader that restates `144`, `128`, `96`, or `256` can drift from them, so
/// only comments may carry those numbers in the record-reading modules.
#[test]
fn record_readers_take_lengths_from_the_decoders() {
    for (path, source) in RECORD_READERS {
        for (index, line) in source.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            let restated = code
                .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .any(|token| matches!(token, "144" | "128" | "96" | "256"));
            assert!(
                !restated,
                "{path}:{} restates a record length instead of naming its decoder: {line}",
                index + 1
            );
        }
    }
}

/// No source module spawns a process, test scaffolding included. A spawned
/// child briefly holds copies of every open descriptor, so a `mkfifo(1)`
/// fallback in one law kept another law's `flock` alive across its
/// drop-and-reopen and surfaced as an intermittent `WriterLock { Busy }`.
/// Laws that need a device node use the kernel API and are gated to Linux.
#[test]
fn no_source_module_spawns_a_process() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![root];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = std::fs::read_to_string(&path)?;
                assert!(
                    !source.contains("process::Command") && !source.contains("Command::new("),
                    "{} spawns a process",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

const EXACT_RECORD_CONSUMERS: [(&str, &str); 3] = [
    (
        "src/adapters/retention/filesystem_retention_current.rs",
        include_str!("../src/adapters/retention/filesystem_retention_current.rs"),
    ),
    (
        "src/adapters/retention/filesystem_retention_stage.rs",
        include_str!("../src/adapters/retention/filesystem_retention_stage.rs"),
    ),
    (
        "src/adapters/store_migration/filesystem_migration_fixed_artifact.rs",
        include_str!("../src/adapters/store_migration/filesystem_migration_fixed_artifact.rs"),
    ),
];

/// Modules ported onto `filesystem_exact_record` no longer open, read, or
/// identity-check records themselves; one implementation of the no-follow,
/// non-blocking, exact-length read serves every stage and record reader.
#[test]
fn exact_record_consumers_do_not_open_records_themselves() {
    for (path, source) in EXACT_RECORD_CONSUMERS {
        for forbidden in [
            "nonblock(true)",
            "fn verify_name(",
            "fn require_metadata(",
            "fn require_absent(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} reimplements the exact-record primitive `{forbidden}`"
            );
        }
    }
}
