//! Public construction laws for canonical version-2 retention roots.

mod support;

use std::io;

use keep::{
    BlobId, CanonicalRetentionRoot, LayoutId, RegisteredRetentionProfile, RetentionAnchor,
    RetentionClosureLimit, RetentionClosureLimitError, RetentionClosureLimits, RetentionNamespace,
    RetentionPolicy, RetentionProfileAdmissionError, RetentionRoot, RetentionRootError,
    RootGeneration,
};

const ONE_ANCHOR_ROOT: &str = include_str!("../conformance/segment-store/v2/one-anchor-root.hex");
const EMPTY_BLOB: &str = concat!(
    "keep:blob:v1:blake3-256:0:",
    "c0074a279c09f9d019dc10e4c821f79f1450cfb8541ab4627132ab9f3c75e33f"
);
const ONE_ZERO_BLOB: &str = concat!(
    "keep:blob:v1:blake3-256:1:",
    "1cfb8fa9e917aba15a1f592095f377ff180755fe1212b0d7d2ec750bd128b606"
);
const ONE_ZERO_LAYOUT: &str = concat!(
    "keep:layout:v1:flat-chunks-v1:blake3-256:220:",
    "887da23f1a7483359a78fc9a7fde80030ec2c4690603803f0ab7d0edb56575b8"
);

#[test]
fn canonical_root_reproduces_the_frozen_one_anchor_record() -> Result<(), Box<dyn std::error::Error>>
{
    let root = one_anchor_root()?;
    let canonical = CanonicalRetentionRoot::from_root(&root)?;
    assert_eq!(canonical.encoded(), fixture_bytes()?);
    assert_eq!(
        canonical.digest().as_bytes(),
        &[
            0xca, 0x4c, 0x11, 0xf2, 0x65, 0xc3, 0xbe, 0xd0, 0x70, 0x73, 0xbd, 0xc3, 0xb6, 0xae,
            0xf0, 0x03, 0xe9, 0x64, 0xac, 0x8c, 0xb3, 0x6f, 0xcf, 0xcc, 0x92, 0xf2, 0x0f, 0xa6,
            0xf0, 0xb6, 0x00, 0x85,
        ]
    );
    Ok(())
}

#[test]
fn closure_limits_refuse_zero_and_excess_before_root_construction() {
    assert_eq!(
        RetentionClosureLimits::new(0, 2, 4_096, 4_096),
        Err(RetentionClosureLimitError::Zero {
            limit: RetentionClosureLimit::Nodes,
        })
    );
    assert_eq!(
        RetentionClosureLimits::new(4, 2, 4_096, 1_073_741_825),
        Err(RetentionClosureLimitError::AboveMaximum {
            limit: RetentionClosureLimit::PhysicalBytes,
            maximum: 1_073_741_824,
            observed: 1_073_741_825,
        })
    );
}

#[test]
fn realization_profile_admits_only_the_exact_registered_definition() {
    let expected = RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1;
    let digest = [
        0xdb, 0x1c, 0x1c, 0x1a, 0x50, 0x61, 0x3e, 0xf1, 0x1f, 0x7c, 0x0e, 0xe0, 0x88, 0x2e, 0x37,
        0xb6, 0xd2, 0x4e, 0x2d, 0xb2, 0xca, 0x57, 0x78, 0x3d, 0x01, 0x19, 0x7b, 0xa5, 0x1b, 0x61,
        0xce, 0x59,
    ];
    assert_eq!(
        RegisteredRetentionProfile::admit(1, 1, digest),
        Ok(expected)
    );
    assert_eq!(
        RegisteredRetentionProfile::admit(2, 1, digest),
        Err(RetentionProfileAdmissionError::UnsupportedCoordinate {
            expected_identity: 1,
            expected_version: 1,
            observed_identity: 2,
            observed_version: 1,
        })
    );
    assert_eq!(
        RegisteredRetentionProfile::admit(1, 1, [0_u8; 32]),
        Err(RetentionProfileAdmissionError::DefinitionDigestMismatch {
            expected: digest,
            observed: [0_u8; 32],
        })
    );
}

#[test]
fn root_predecessors_and_anchor_sets_have_one_canonical_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let initial = one_anchor_root()?;
    let canonical = CanonicalRetentionRoot::from_root(&initial)?;
    let predecessor = canonical.digest();
    let namespace = RetentionNamespace::try_from(vec![0x00, 0x2f, 0xff])?;
    let limits = RetentionClosureLimits::new(4, 2, 4_096, 4_096)?;
    let profile = RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1;
    let policy = RetentionPolicy::new(profile, limits);
    let anchor = one_zero_anchor()?;
    let earlier_anchor = RetentionAnchor::new(EMPTY_BLOB.parse()?, anchor.layout_id());

    assert_eq!(
        RetentionRoot::new(
            namespace.clone(),
            RootGeneration::new(1)?,
            policy,
            Some(predecessor),
            vec![anchor],
        ),
        Err(RetentionRootError::InitialGenerationHasPredecessor {
            observed: predecessor,
        })
    );
    assert_eq!(
        RetentionRoot::new(
            namespace.clone(),
            RootGeneration::new(2)?,
            policy,
            None,
            vec![anchor],
        ),
        Err(RetentionRootError::MissingPredecessor {
            generation: RootGeneration::new(2)?,
        })
    );
    assert_eq!(
        RetentionRoot::new(
            namespace.clone(),
            RootGeneration::new(2)?,
            policy,
            Some(predecessor),
            vec![anchor, anchor],
        ),
        Err(RetentionRootError::DuplicateAnchor { anchor })
    );
    let sorted = RetentionRoot::new(
        namespace,
        RootGeneration::new(2)?,
        policy,
        Some(predecessor),
        vec![anchor, earlier_anchor],
    )?;
    assert_eq!(sorted.anchors(), &[earlier_anchor, anchor]);
    Ok(())
}

fn one_anchor_root() -> Result<RetentionRoot, Box<dyn std::error::Error>> {
    Ok(RetentionRoot::new(
        RetentionNamespace::try_from(vec![0x00, 0x2f, 0xff])?,
        RootGeneration::new(1)?,
        RetentionPolicy::new(
            RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1,
            RetentionClosureLimits::new(4, 2, 4_096, 4_096)?,
        ),
        None,
        vec![one_zero_anchor()?],
    )?)
}

fn one_zero_anchor() -> Result<RetentionAnchor, Box<dyn std::error::Error>> {
    let blob: BlobId = ONE_ZERO_BLOB.parse()?;
    let layout: LayoutId = ONE_ZERO_LAYOUT.parse()?;
    Ok(RetentionAnchor::new(blob, layout))
}

fn fixture_bytes() -> Result<Vec<u8>, io::Error> {
    let encoded = ONE_ANCHOR_ROOT
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("retention root fixture lacks final newline"))?;
    if encoded.contains(['\n', '\r']) {
        return Err(io::Error::other(
            "retention root fixture contains an embedded line ending",
        ));
    }
    support::decode_hex(encoded)
}
