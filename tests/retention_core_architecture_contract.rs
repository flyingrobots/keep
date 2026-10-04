//! The retention core admits no caller identity, path, clock, or application policy.

use std::error::Error;
use std::fs;
use std::path::Path;

/// Tokens that would let identity, paths, clocks, or environment into a
/// transition decision. The filesystem adapters own paths; the core does not.
const FORBIDDEN: [&str; 8] = [
    "SystemTime",
    "Instant",
    "std::env",
    "std::path",
    "std::fs",
    "getuid",
    "hostname",
    "username",
];

/// Storage-independent retention modules outside `src/retention/`.
const CORE_ADAPTERS: [&str; 8] = [
    "src/adapters/retention/transition_planner.rs",
    "src/adapters/retention/transition_preflight.rs",
    "src/adapters/retention/publication_preparation.rs",
    "src/adapters/retention/publication_execution.rs",
    "src/adapters/retention/publication_storage.rs",
    "src/adapters/retention/recovery_planner.rs",
    "src/adapters/retention/recovery_execution.rs",
    "src/adapters/retention/retention_view_collector.rs",
];

#[test]
fn the_retention_core_admits_no_identity_path_clock_or_policy() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    for entry in fs::read_dir(root.join("src/retention"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
    for adapter in CORE_ADAPTERS {
        let path = root.join(adapter);
        assert!(
            path.is_file(),
            "{adapter} is missing; update the contract list"
        );
        sources.push(path);
    }
    for path in sources {
        let source = fs::read_to_string(&path)?;
        for token in FORBIDDEN {
            assert!(
                !source.contains(token),
                "{} names `{token}`; the retention core decides from evidence alone",
                path.display()
            );
        }
    }
    Ok(())
}
