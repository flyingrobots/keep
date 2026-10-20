//! Production I/O adapters retain typed payloads instead of constructing textual failures.

use std::error::Error;
use std::fs;
use std::path::Path;

#[test]
fn production_io_failures_never_start_with_a_text_payload() -> Result<(), Box<dyn Error>> {
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if is_production_source(&path) {
                assert_typed_payloads(&path, &fs::read_to_string(&path)?);
            }
        }
    }
    Ok(())
}

fn is_production_source(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "rs")
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| !name.contains("test"))
}

#[expect(
    clippy::literal_string_with_formatting_args,
    reason = "the guard matches Rust source syntax, not a formatted diagnostic"
)]
fn assert_typed_payloads(path: &Path, source: &str) {
    let production = source
        .split("\n#[cfg(test)]\nmod tests")
        .next()
        .unwrap_or_default();
    let compact: String = production
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    for call in compact.split("io::Error::new(").skip(1) {
        if let Some((_kind, payload)) = call.split_once(',') {
            assert!(
                !is_text(payload),
                "{} constructs an I/O failure from text",
                path.display()
            );
        }
    }
    for payload in compact.split("io::Error::other(").skip(1) {
        assert!(
            !is_text(payload),
            "{} constructs an I/O failure from text",
            path.display()
        );
    }
    assert!(
        !compact.contains("refusal.to_string()"),
        "{} erases a refusal source",
        path.display()
    );
    assert!(
        !compact.contains("format!(\"{refusal:?}\")"),
        "{} erases a refusal source",
        path.display()
    );
}

fn is_text(payload: &str) -> bool {
    payload.starts_with('"') || payload.starts_with("format!(")
}
