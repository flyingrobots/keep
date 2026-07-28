//! This module owns the pure-Rust source-admission regression law.

use crate::git_inventory::GitPath;

#[test]
fn python_source_is_refused_by_the_pure_rust_boundary() {
    for path in ["scripts/check.py", "scripts/check.PY"] {
        assert!(matches!(
            super::admit_source_path(&GitPath::new(path.as_bytes().to_vec())),
            Err(super::SourceStructureError::PythonSource(ref observed))
                if observed == path
        ));
    }
}
