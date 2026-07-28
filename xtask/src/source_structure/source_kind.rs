//! This module owns repository source-module classification.

pub(super) fn is_source_module(path: &[u8]) -> bool {
    if is_python_module(path) {
        return true;
    }
    let Some(suffix) = source_suffix(path) else {
        return false;
    };
    suffix == b"rs" || suffix == b"sh"
}

pub(super) fn is_python_module(path: &[u8]) -> bool {
    repository_file_name(path).is_some_and(|file_name| {
        file_name.strip_prefix(b".").is_some_and(is_python_suffix)
            || source_suffix(path).is_some_and(is_python_suffix)
    })
}

pub(super) fn is_source_candidate(path: &[u8]) -> bool {
    is_source_module(path) || is_extensionless_file(path)
}

pub(super) fn is_extensionless_file(path: &[u8]) -> bool {
    let Some(file_name) = repository_file_name(path) else {
        return false;
    };
    !file_name.is_empty() && !file_name.contains(&b'.')
}

const fn is_python_suffix(suffix: &[u8]) -> bool {
    suffix.eq_ignore_ascii_case(b"py") || suffix.eq_ignore_ascii_case(b"pyw")
}

fn source_suffix(path: &[u8]) -> Option<&[u8]> {
    let file_name = repository_file_name(path)?;
    let mut components = file_name.rsplitn(2, |byte| *byte == b'.');
    let suffix = components.next()?;
    let stem = components.next()?;
    (!stem.is_empty()).then_some(suffix)
}

fn repository_file_name(path: &[u8]) -> Option<&[u8]> {
    path.rsplit(|byte| *byte == b'/').next()
}
