//! Repository dependency-policy configuration regression evidence.

const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");
const POLICY: &str = include_str!("../../deny.toml");

#[test]
fn fuzz_dependency_gate_uses_the_reviewed_repository_policy() {
    assert!(
        CI_WORKFLOW
            .contains("cargo deny --manifest-path fuzz/Cargo.toml check --config ../deny.toml")
    );
}

#[test]
fn fuzz_only_license_exceptions_are_package_and_version_scoped() {
    for exception in [
        r#"crate = "libfuzzer-sys@0.4.13""#,
        r#"crate = "memchr@2.8.3""#,
        r#"crate = "winx@0.36.4""#,
        r#"crate = "zmij@1.0.23""#,
    ] {
        assert!(POLICY.contains(exception), "missing {exception}");
    }
}
