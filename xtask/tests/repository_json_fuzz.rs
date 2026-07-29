//! Integration laws for duplicate-refusing repository JSON fuzz admission.

#![cfg(feature = "repository-json-fuzz")]

use xtask::{RepositoryJsonAdmission, admit_repository_json};

#[test]
fn repository_json_fuzz_boundary_reaches_success_and_refusal_classes() {
    let valid = br#"{"outer":[true,null,{"key":"value"}]}"#;
    let malformed = br#"{"outer":["#;
    let nested_duplicate = br#"{"outer":[{"key":1,"key":2}]}"#;

    assert_eq!(
        admit_repository_json(valid),
        RepositoryJsonAdmission::Admitted
    );
    for refused in [malformed.as_slice(), nested_duplicate.as_slice()] {
        assert_eq!(
            admit_repository_json(refused),
            RepositoryJsonAdmission::Refused
        );
    }
}

#[test]
fn repository_json_fuzz_boundary_refuses_resource_limits() {
    let oversized = vec![b' '; 1_048_577];
    let deeply_nested = format!("{}null{}", "[".repeat(256), "]".repeat(256));

    for refused in [oversized.as_slice(), deeply_nested.as_bytes()] {
        assert_eq!(
            admit_repository_json(refused),
            RepositoryJsonAdmission::Refused
        );
    }
}
