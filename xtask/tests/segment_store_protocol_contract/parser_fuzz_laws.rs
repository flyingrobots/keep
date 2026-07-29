//! Fuzz-evidence laws for durable catalog parser boundaries.

use std::error::Error;
use std::path::Path;

const FUZZ_MANIFEST: &str = include_str!("../../../fuzz/Cargo.toml");
const FUZZ_GUIDE: &str = include_str!("../../../fuzz/README.md");
const REQUIREMENTS: &str = include_str!("../../../docs/formats/segment-store-v1/requirements.md");

#[test]
fn catalog_and_head_decoders_have_registered_seeded_fuzz_evidence() -> Result<(), Box<dyn Error>> {
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("xtask manifest must have a repository parent")?;

    assert!(
        repository_root
            .join("fuzz/fuzz_targets/catalog_format.rs")
            .is_file()
    );
    assert!(FUZZ_MANIFEST.contains("name = \"catalog_format\""));
    assert!(FUZZ_MANIFEST.contains("path = \"fuzz_targets/catalog_format.rs\""));
    assert!(FUZZ_GUIDE.contains("The `catalog_format` seeds"));
    assert!(REQUIREMENTS.contains("`KEEP-CATALOG-011`"));
    Ok(())
}
