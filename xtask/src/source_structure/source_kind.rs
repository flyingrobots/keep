//! This module owns repository source-module classification.

const SOURCE_SUFFIXES: [[u8; 2]; 3] = [*b"py", *b"rs", *b"sh"];

pub(super) fn is_source_module(path: &[u8]) -> bool {
    let Some(suffix) = source_suffix(path) else {
        return false;
    };
    SOURCE_SUFFIXES.iter().any(|candidate| {
        suffix == candidate || (*candidate == *b"py" && suffix.eq_ignore_ascii_case(b"py"))
    })
}

pub(super) fn is_python_module(path: &[u8]) -> bool {
    source_suffix(path).is_some_and(|suffix| suffix.eq_ignore_ascii_case(b"py"))
}

fn source_suffix(path: &[u8]) -> Option<&[u8]> {
    let file_name = path.rsplit(|byte| *byte == b'/').next()?;
    let mut components = file_name.rsplitn(2, |byte| *byte == b'.');
    let suffix = components.next()?;
    let stem = components.next()?;
    (!stem.is_empty()).then_some(suffix)
}
