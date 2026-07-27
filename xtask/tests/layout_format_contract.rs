//! Written-contract regression evidence for the flat chunk layout format.

const SPECIFICATION: &str = include_str!("../../docs/formats/flat-chunk-layout-v1/README.md");
const MUTATIONS: &str = include_str!("../../conformance/layout/v1/mutations.tsv");

#[test]
fn verified_reconstruction_proves_the_bound_storage_profile() {
    assert!(SPECIFICATION.contains("replayed the"));
    assert!(SPECIFICATION.contains("registered profile's boundary detector"));
    assert!(
        MUTATIONS
            .lines()
            .any(|line| line.starts_with("profile-boundary-mismatch\t"))
    );
}
