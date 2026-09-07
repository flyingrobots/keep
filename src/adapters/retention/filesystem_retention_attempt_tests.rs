//! Filesystem retention publication attempt-state laws.

use std::error::Error;
use std::fs;
use std::io;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, initial_root, open_authority, refusal,
    retention_witness, root_pool_path,
};
use super::{
    AdmittedRetentionRoot, RetentionCurrentStateRefusal, RetentionNamespaceAdmission,
    RetentionPublicationStorage, RetentionTransitionDisposition,
};

#[test]
fn refused_verification_admits_no_later_phase() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-attempt-refused")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    fs::write(
        sandbox.path().join("retention").join("head.next"),
        b"retained",
    )?;
    let before = retention_witness(sandbox.path())?;

    let error = authority
        .verify_current(&preparation)
        .err()
        .ok_or("retained head stage was admitted")?;
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::RetainedStage)
    ));
    let error = authority
        .write_root_stage(preparation.candidate())
        .err()
        .ok_or("root stage was written without an admitted attempt")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(retention_witness(sandbox.path())?, before);
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn stale_stage_handle_does_not_survive_a_refused_verification() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-attempt-stale")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    assert_eq!(
        authority.verify_current(&preparation)?,
        RetentionTransitionDisposition::Publish
    );
    authority.write_root_stage(preparation.candidate())?;

    let error = authority
        .verify_current(&preparation)
        .err()
        .ok_or("retained root stage was admitted")?;
    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::RetainedStage)
    ));
    let error = authority
        .synchronize_root_stage()
        .err()
        .ok_or("stale root stage handle was reused after a refused verification")?;

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(
        sandbox.path().join("retention").join("root.next").is_file(),
        "retained stage evidence must remain for recovery"
    );
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn namespace_admission_refuses_a_directory_the_expectation_excludes() -> Result<(), Box<dyn Error>>
{
    let (sandbox, mut authority) = open_authority("filesystem-retention-attempt-namespace")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    assert_eq!(
        authority.verify_current(&preparation)?,
        RetentionTransitionDisposition::Publish
    );
    authority.write_root_stage(preparation.candidate())?;
    let namespace = root_pool_path(sandbox.path(), preparation.candidate())
        .parent()
        .ok_or("root pool path has no namespace parent")?
        .to_path_buf();
    fs::create_dir(&namespace)?;

    let error = authority
        .admit_root_namespace(preparation.candidate())
        .err()
        .ok_or("namespace directory the expectation excludes was admitted")?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::NamespaceExpectationViolated)
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn namespace_phases_refuse_a_root_outside_the_admitted_namespace() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("filesystem-retention-attempt-other-root")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    assert_eq!(
        authority.verify_current(&preparation)?,
        RetentionTransitionDisposition::Publish
    );
    authority.write_root_stage(preparation.candidate())?;
    authority.synchronize_root_stage()?;
    assert_eq!(
        authority.admit_root_namespace(preparation.candidate())?,
        RetentionNamespaceAdmission::Created
    );
    let template = AdmittedRetentionRoot::decode(&root_bytes)?;
    let other = initial_root(b"a-namespace-the-attempt-did-not-admit", &template)?;
    let other = AdmittedRetentionRoot::decode(other.encoded())?;

    let error = authority
        .synchronize_root_namespace(&other)
        .err()
        .ok_or("a root outside the admitted namespace was synchronized")?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::AttemptNamespaceDisagreed)
    ));
    drop(authority);
    sandbox.remove()?;
    Ok(())
}
