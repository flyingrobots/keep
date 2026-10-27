//! Canonical immutable-pool recovery-name grammar laws.

use std::error::Error;

use keep::{
    RecoveryNameClassificationError, RecoveryNamespace, RecoveryPoolNameError,
    classify_recovery_names,
};

use super::{initialized_root, inventory, name};

#[test]
fn segment_name_grammar_returns_exact_refusals() -> Result<(), Box<dyn Error>> {
    let wrong_suffix = format!("{}.cat", "00".repeat(32));
    let uppercase = format!("{}.seg", "AA".repeat(32));
    let invalid = format!("{}g.seg", "0".repeat(63));

    assert_eq!(
        pool_error(RecoveryNamespace::Segments, b"short")?,
        RecoveryPoolNameError::WrongLength {
            expected: 68,
            observed: 5,
        }
    );
    assert_eq!(
        pool_error(RecoveryNamespace::Segments, wrong_suffix.as_bytes())?,
        RecoveryPoolNameError::WrongSuffix
    );
    assert_eq!(
        pool_error(RecoveryNamespace::Segments, uppercase.as_bytes())?,
        RecoveryPoolNameError::UppercaseDigest
    );
    assert_eq!(
        pool_error(RecoveryNamespace::Segments, invalid.as_bytes())?,
        RecoveryPoolNameError::InvalidDigestAlphabet
    );
    Ok(())
}

#[test]
fn catalog_name_grammar_returns_exact_refusals() -> Result<(), Box<dyn Error>> {
    let digest = "00".repeat(32);
    let cases = [
        (
            format!("{:016x}_{}.cat", 1_u64, digest),
            RecoveryPoolNameError::WrongSeparator,
        ),
        (
            format!("000000000000000A-{digest}.cat"),
            RecoveryPoolNameError::UppercaseGeneration,
        ),
        (
            format!("000000000000000G-{digest}.cat"),
            RecoveryPoolNameError::InvalidGenerationAlphabet,
        ),
        (
            format!("0000000000000000-{digest}.cat"),
            RecoveryPoolNameError::ZeroGeneration,
        ),
        (
            format!("0000000000000001-{}.cat", "AA".repeat(32)),
            RecoveryPoolNameError::UppercaseDigest,
        ),
        (
            format!("0000000000000001-{}g.cat", "0".repeat(63)),
            RecoveryPoolNameError::InvalidDigestAlphabet,
        ),
    ];

    for (candidate, expected) in cases {
        assert_eq!(
            pool_error(RecoveryNamespace::Catalogs, candidate.as_bytes())?,
            expected
        );
    }
    Ok(())
}

fn pool_error(
    namespace: RecoveryNamespace,
    candidate: &[u8],
) -> Result<RecoveryPoolNameError, Box<dyn Error>> {
    let entries = match namespace {
        RecoveryNamespace::Segments => [
            initialized_root()?,
            Vec::new(),
            vec![name(candidate)?],
            Vec::new(),
        ],
        RecoveryNamespace::Catalogs => [
            initialized_root()?,
            Vec::new(),
            Vec::new(),
            vec![name(candidate)?],
        ],
        RecoveryNamespace::Root | RecoveryNamespace::Staging => {
            return Err("pool grammar test requires an immutable namespace".into());
        }
    };
    let Err(error) = classify_recovery_names(inventory(entries)?) else {
        return Err("name classification admitted a noncanonical pool name".into());
    };
    match error {
        RecoveryNameClassificationError::PoolName { source, .. } => Ok(source),
        unexpected => Err(format!("unexpected name-classification error: {unexpected}").into()),
    }
}
