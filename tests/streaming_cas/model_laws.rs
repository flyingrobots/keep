//! Exhaustive short operation sequences against a boring reference model.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::Cursor;

use keep::{
    BlobId, IngestionError, LayoutEntryLimit, ReconstructionError, ReferenceStore,
    ReferenceStoreCapacity,
};

const ALPHA: &[u8] = b"alpha";
const BRAVO: &[u8] = b"bravo";
const EMPTY: &[u8] = b"";
const CASES: [&[u8]; 3] = [ALPHA, BRAVO, EMPTY];
const OPERATIONS: [Operation; 6] = [
    Operation::Admit(ALPHA),
    Operation::Admit(BRAVO),
    Operation::Admit(EMPTY),
    Operation::Read(ALPHA),
    Operation::DropStaged(BRAVO),
    Operation::ClaimMismatch {
        expected: ALPHA,
        observed: BRAVO,
    },
];

#[test]
fn exhaustive_short_sequences_agree_with_the_boring_model() -> Result<(), Box<dyn Error>> {
    for first in OPERATIONS {
        for second in OPERATIONS {
            for third in OPERATIONS {
                assert_sequence([first, second, third])?;
            }
        }
    }
    Ok(())
}

fn assert_sequence(sequence: [Operation; 3]) -> Result<(), Box<dyn Error>> {
    let mut model = BoringModel::new();
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_024));
    for operation in sequence {
        apply(operation, &mut model, &mut store)?;
        assert_equivalent(&model, &store)?;
    }
    Ok(())
}

fn apply(
    operation: Operation,
    model: &mut BoringModel,
    store: &mut ReferenceStore,
) -> Result<(), Box<dyn Error>> {
    match operation {
        Operation::Admit(bytes) => {
            let expected = model.admit(bytes)?;
            let mut source = Cursor::new(bytes);
            let published = store
                .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
                .commit(store)?;
            assert_eq!(published.target(), expected);
        }
        Operation::Read(bytes) => assert_read(model, store, bytes)?,
        Operation::DropStaged(bytes) => {
            let mut source = Cursor::new(bytes);
            let staged = store.stage(&mut source, LayoutEntryLimit::MAXIMUM)?;
            assert_eq!(staged.target(), BlobId::hash_bytes(bytes)?);
        }
        Operation::ClaimMismatch { expected, observed } => {
            let expected = BlobId::hash_bytes(expected)?;
            let observed_id = BlobId::hash_bytes(observed)?;
            let mut source = Cursor::new(observed);
            assert!(matches!(
                store.stage_expected(&mut source, expected, LayoutEntryLimit::MAXIMUM),
                Err(IngestionError::BlobIdentityMismatch {
                    expected: error_expected,
                    observed: error_observed
                }) if error_expected == expected && error_observed == observed_id
            ));
        }
    }
    Ok(())
}

fn assert_equivalent(model: &BoringModel, store: &ReferenceStore) -> Result<(), Box<dyn Error>> {
    for bytes in CASES {
        assert_read(model, store, bytes)?;
    }
    Ok(())
}

fn assert_read(
    model: &BoringModel,
    store: &ReferenceStore,
    bytes: &[u8],
) -> Result<(), Box<dyn Error>> {
    let identity = BlobId::hash_bytes(bytes)?;
    let mut observed = Vec::new();
    if let Some(expected) = model.read(identity) {
        let receipt = store.reconstruct(identity, &mut observed)?;
        assert_eq!(receipt.target(), identity);
        assert_eq!(observed, expected);
    } else {
        assert!(matches!(
            store.reconstruct(identity, &mut observed),
            Err(ReconstructionError::BlobMissing { requested })
                if requested == identity
        ));
        assert!(observed.is_empty());
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Operation {
    Admit(&'static [u8]),
    Read(&'static [u8]),
    DropStaged(&'static [u8]),
    ClaimMismatch {
        expected: &'static [u8],
        observed: &'static [u8],
    },
}

struct BoringModel {
    blobs: BTreeMap<BlobId, Vec<u8>>,
}

impl BoringModel {
    const fn new() -> Self {
        Self {
            blobs: BTreeMap::new(),
        }
    }

    fn admit(&mut self, bytes: &[u8]) -> Result<BlobId, keep::BlobHashError> {
        let identity = BlobId::hash_bytes(bytes)?;
        self.blobs.entry(identity).or_insert_with(|| bytes.to_vec());
        Ok(identity)
    }

    fn read(&self, identity: BlobId) -> Option<&[u8]> {
        self.blobs.get(&identity).map(Vec::as_slice)
    }
}
