//! Source-layout laws for the adapters tree: the module root stays a scannable
//! manifest, and files rustfmt cannot rewrap stay within the standard width.

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
