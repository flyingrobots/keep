//! This module owns the nine literal Rust source basename prohibitions.

use std::path::Path;

use super::SourceStructureError;

const FORBIDDEN: [&str; 9] = [
    "utils.rs",
    "helpers.rs",
    "common.rs",
    "misc.rs",
    "shared.rs",
    "manager.rs",
    "service.rs",
    "types.rs",
    "models.rs",
];

pub(super) fn admit(path: &Path) -> Result<(), SourceStructureError> {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| FORBIDDEN.contains(&name))
    {
        Err(SourceStructureError::ForbiddenFilename(path.to_owned()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SourceStructureError;
    use std::path::Path;

    #[test]
    fn every_prohibited_basename_refuses_in_every_parent() {
        for name in [
            "utils.rs",
            "helpers.rs",
            "common.rs",
            "misc.rs",
            "shared.rs",
            "manager.rs",
            "service.rs",
            "types.rs",
            "models.rs",
        ] {
            for parent in ["", "src", "tests/nested"] {
                let path = Path::new(parent).join(name);
                assert!(matches!(super::admit(&path),
                    Err(SourceStructureError::ForbiddenFilename(observed)) if observed == path));
            }
        }
    }

    #[test]
    fn semantic_prefixes_and_directories_do_not_inherit_a_basename_ban() {
        for path in [
            "segment_header.rs",
            "storage_models.rs",
            "helpers.rs/record.rs",
            "models.rs.txt",
            "Utils.rs",
        ] {
            assert!(super::admit(Path::new(path)).is_ok());
        }
    }
}
