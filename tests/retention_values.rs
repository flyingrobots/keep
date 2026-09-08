//! Public laws for version-2 retention values.

use keep::{
    BlobId, LayoutId, LivenessGeneration, LivenessGenerationError, RetentionAnchor,
    RetentionNamespace, RetentionNamespaceError, RootGeneration, RootGenerationError,
};

const ONE_ZERO_BLOB: &str = concat!(
    "keep:blob:v1:blake3-256:1:",
    "1cfb8fa9e917aba15a1f592095f377ff180755fe1212b0d7d2ec750bd128b606"
);
const ONE_ZERO_LAYOUT: &str = concat!(
    "keep:layout:v1:flat-chunks-v1:blake3-256:220:",
    "887da23f1a7483359a78fc9a7fde80030ec2c4690603803f0ab7d0edb56575b8"
);

#[test]
fn retention_namespaces_preserve_every_admitted_byte_and_bind_length()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        RetentionNamespace::try_from(Vec::new()),
        Err(RetentionNamespaceError::Empty)
    );
    assert_eq!(
        RetentionNamespace::try_from(vec![0_u8; 256]),
        Err(RetentionNamespaceError::TooLong {
            maximum: 255,
            observed: 256,
        })
    );

    let bytes = [0x00, 0x2f, 0xff];
    let namespace = RetentionNamespace::try_from(bytes.as_slice())?;
    assert_eq!(namespace.as_bytes(), bytes.as_slice());
    assert_eq!(
        namespace.digest().as_bytes(),
        &[
            0xdd, 0xde, 0x2a, 0xc6, 0x5c, 0x5b, 0xa3, 0x82, 0x9b, 0xf0, 0xfb, 0xd6, 0xf3, 0x6e,
            0x90, 0x27, 0x2d, 0x69, 0xa0, 0x45, 0x9f, 0xad, 0xe9, 0x22, 0x72, 0xb7, 0x28, 0xa8,
            0x0d, 0x7a, 0xe6, 0xe2,
        ]
    );
    Ok(())
}

#[test]
fn retention_generations_are_positive_checked_and_semantically_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(RootGeneration::new(0), Err(RootGenerationError::Zero));
    assert_eq!(
        LivenessGeneration::new(0),
        Err(LivenessGenerationError::Zero)
    );

    let root = RootGeneration::new(1)?;
    let liveness = LivenessGeneration::new(1)?;
    assert_eq!(root.successor().map(RootGeneration::get), Ok(2));
    assert_eq!(liveness.successor().map(LivenessGeneration::get), Ok(2));
    assert_eq!(
        RootGeneration::new(u64::MAX).and_then(RootGeneration::successor),
        Err(RootGenerationError::Exhausted { current: u64::MAX })
    );
    assert_eq!(
        LivenessGeneration::new(u64::MAX).and_then(LivenessGeneration::successor),
        Err(LivenessGenerationError::Exhausted { current: u64::MAX })
    );
    Ok(())
}

#[test]
fn retention_anchors_preserve_exact_logical_and_layout_coordinates()
-> Result<(), Box<dyn std::error::Error>> {
    let blob: BlobId = ONE_ZERO_BLOB.parse()?;
    let layout: LayoutId = ONE_ZERO_LAYOUT.parse()?;
    let anchor = RetentionAnchor::new(blob, layout);
    assert_eq!(anchor.blob_id(), blob);
    assert_eq!(anchor.layout_id(), layout);
    Ok(())
}
