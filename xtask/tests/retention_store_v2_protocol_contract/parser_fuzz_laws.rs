//! Fuzz-evidence laws for durable retention parser boundaries.

use std::error::Error;
use std::path::Path;

const FUZZ_MANIFEST: &str = include_str!("../../../fuzz/Cargo.toml");
const FUZZ_GUIDE: &str = include_str!("../../../fuzz/README.md");
const REQUIREMENTS: &str = include_str!("../../../docs/formats/segment-store-v2/requirements.md");

#[test]
fn retention_decoders_have_registered_seeded_fuzz_evidence() -> Result<(), Box<dyn Error>> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest must have a repository parent")?;

    assert!(
        repository_root
            .join("fuzz/fuzz_targets/retention_format.rs")
            .is_file()
    );
    assert!(
        repository_root
            .join("xtask/src/fuzz_seed_corpus/retention_seeds.rs")
            .is_file()
    );
    assert!(FUZZ_MANIFEST.contains("name = \"retention_format\""));
    assert!(FUZZ_MANIFEST.contains("path = \"fuzz_targets/retention_format.rs\""));
    assert!(FUZZ_GUIDE.contains("The `retention_format` seeds"));
    assert!(REQUIREMENTS.contains("`retention_format`"));
    Ok(())
}
