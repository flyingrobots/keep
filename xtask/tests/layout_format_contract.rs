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

#[test]
fn mutation_outcomes_follow_first_failure_precedence() {
    assert_eq!(
        expected_mutation_outcome("inserted-duplicate-flags-field"),
        Some("layout.wrong-header-length")
    );
    assert_eq!(
        expected_mutation_outcome("entry-order-swap"),
        Some("layout.gap")
    );
}

fn expected_mutation_outcome(case_name: &str) -> Option<&'static str> {
    MUTATIONS.lines().skip(2).find_map(|line| {
        let mut fields = line.split('\t');
        if fields.next()? != case_name {
            return None;
        }
        fields.nth(7)
    })
}
