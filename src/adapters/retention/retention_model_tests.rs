//! Model-based retention laws: every operation sequence agrees with a
//! deterministic namespace-to-anchor-set map, and a refused operation leaves
//! the fenced reader view exactly where it was.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

#[path = "retention_model_refusal.rs"]
mod refusal;
use refusal::Refusal;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, initial_root, new_namespace_preparation,
    open_authority, successor_preparation,
};
use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, FilesystemRetentionPublicationAuthority,
    FilesystemRetentionSnapshot, ReaderAttemptLimit, RetentionPublicationOutcome,
    RetentionPublicationPreparation,
};
use crate::adapters::{
    CatalogRestartByteLimit, CatalogRestartPolicy, SegmentReadPolicy, SegmentRecordLimit,
};
use crate::{
    CanonicalRetentionRoot, LayoutEntryLimit, RetentionAnchor, RetentionNamespaceDigest,
    RetentionPolicy, RetentionRoot, execute_retention_publication,
};

const NAMESPACE_B: &[u8] = b"model-namespace-b";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Namespace {
    A,
    B,
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    /// Publish generation one of the namespace from a fresh view.
    Initial(Namespace),
    /// Publish the exact successor of namespace A from a fresh view.
    Successor,
    /// Publish an empty anchor set in namespace A's next generation.
    Release,
    /// Restore the original anchors in namespace A's next generation.
    Restore,
    /// Replay the last accepted publication byte for byte.
    RetryLast,
    /// Publish generation one of namespace A from a view that predates it.
    StaleInitial,
}

const OPERATIONS: [Operation; 7] = [
    Operation::Initial(Namespace::A),
    Operation::Initial(Namespace::B),
    Operation::Successor,
    Operation::Release,
    Operation::Restore,
    Operation::RetryLast,
    Operation::StaleInitial,
];

/// The byte ingredients of one preparation; rebuilding it is byte-identical.
#[derive(Clone)]
enum Recipe {
    Initial {
        candidate: Vec<u8>,
        manifest: Option<Vec<u8>>,
    },
    Successor {
        current_root: Vec<u8>,
        manifest: Vec<u8>,
        candidate: Vec<u8>,
    },
}

impl Recipe {
    fn publish(
        &self,
        authority: &mut FilesystemRetentionPublicationAuthority,
    ) -> Result<RetentionPublicationOutcome, Box<dyn Error>> {
        let preparation: RetentionPublicationPreparation<'_> = match self {
            Self::Initial {
                candidate,
                manifest: None,
            } => initial_preparation(candidate)?,
            Self::Initial {
                candidate,
                manifest: Some(manifest),
            } => {
                new_namespace_preparation(&AdmittedRetentionManifest::decode(manifest)?, candidate)?
            }
            Self::Successor {
                current_root,
                manifest,
                candidate,
            } => successor_preparation(
                &AdmittedRetentionRoot::decode(current_root)?,
                &AdmittedRetentionManifest::decode(manifest)?,
                candidate,
            )?,
        };
        Ok(execute_retention_publication(authority, &preparation)?.outcome())
    }
}

/// The reference model: namespace digest to (generation, anchors), plus the
/// liveness generation, which counts accepted publications.
#[derive(Default)]
struct Model {
    namespaces: BTreeMap<RetentionNamespaceDigest, (u64, Vec<RetentionAnchor>)>,
    liveness: u64,
}

struct Store {
    authority: FilesystemRetentionPublicationAuthority,
    path: PathBuf,
    template: Vec<u8>,
    last_accepted: Option<Recipe>,
}

fn policy() -> Result<CatalogRestartPolicy, Box<dyn Error>> {
    Ok(CatalogRestartPolicy::new(
        SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
        CatalogRestartByteLimit::new(1_048_576)?,
    ))
}

fn snapshot(store: &Store) -> Result<FilesystemRetentionSnapshot, Box<dyn Error>> {
    Ok(FilesystemRetentionSnapshot::load(
        &store.path,
        policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?)
}

fn digest_of(bytes: &[u8]) -> Result<RetentionNamespaceDigest, Box<dyn Error>> {
    Ok(AdmittedRetentionRoot::decode(bytes)?
        .root()
        .namespace()
        .digest())
}

fn candidate_bytes(store: &Store, namespace: Namespace) -> Result<Vec<u8>, Box<dyn Error>> {
    match namespace {
        Namespace::A => Ok(store.template.clone()),
        Namespace::B => {
            let template = AdmittedRetentionRoot::decode(&store.template)?;
            Ok(initial_root(NAMESPACE_B, &template)?.encoded().to_vec())
        }
    }
}

/// A recipe and the answer the model expects for it.
type Planned = (Recipe, Expected);

/// What the model says the store must answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Expected {
    Published,
    AlreadyCommitted,
    Refused(Refusal),
}

/// Builds the recipe an operation would publish and the model's expected
/// answer. `None` means the operation has no candidate.
fn recipe(
    store: &Store,
    model: &Model,
    operation: Operation,
) -> Result<Option<Planned>, Box<dyn Error>> {
    let fresh_manifest = store
        .authority
        .observe_current()?
        .map(|state| state.manifest_bytes().to_vec());
    Ok(match operation {
        Operation::Initial(namespace) => {
            let candidate = candidate_bytes(store, namespace)?;
            let expected = if model.namespaces.contains_key(&digest_of(&candidate)?) {
                Expected::Refused(Refusal::repeated_initial(
                    &candidate,
                    fresh_manifest.as_deref(),
                )?)
            } else {
                Expected::Published
            };
            Some((
                Recipe::Initial {
                    candidate,
                    manifest: fresh_manifest,
                },
                expected,
            ))
        }
        Operation::StaleInitial => {
            // The stale caller's preparation stages liveness generation one,
            // so it is byte-identical to the accepted publication only while
            // that publication is still the whole history; any later
            // publication supersedes it.
            let candidate = candidate_bytes(store, Namespace::A)?;
            let expected = match (
                model.namespaces.get(&digest_of(&candidate)?),
                model.liveness,
            ) {
                (None, 0) => Expected::Published,
                (Some(_), 1) => Expected::AlreadyCommitted,
                _ => Expected::Refused(Refusal::superseded(fresh_manifest.as_deref())?),
            };
            Some((
                Recipe::Initial {
                    candidate,
                    manifest: None,
                },
                expected,
            ))
        }
        Operation::Successor | Operation::Release | Operation::Restore => {
            return successor_recipe(store, model, operation, fresh_manifest);
        }
        Operation::RetryLast => store
            .last_accepted
            .clone()
            .map(|recipe| (recipe, Expected::AlreadyCommitted)),
    })
}

fn successor_recipe(
    store: &Store,
    model: &Model,
    operation: Operation,
    manifest: Option<Vec<u8>>,
) -> Result<Option<Planned>, Box<dyn Error>> {
    let digest = digest_of(&store.template)?;
    if !model.namespaces.contains_key(&digest) {
        return Ok(None);
    }
    let current_root = snapshot(store)?
        .retained_root(digest)?
        .ok_or("model root absent on disk")?
        .to_vec();
    let current = AdmittedRetentionRoot::decode(&current_root)?;
    let anchors = match operation {
        Operation::Release => Vec::new(),
        Operation::Restore => AdmittedRetentionRoot::decode(&store.template)?
            .root()
            .anchors()
            .to_vec(),
        _ => current.root().anchors().to_vec(),
    };
    let root = RetentionRoot::new(
        current.root().namespace().clone(),
        current.root().generation().successor()?,
        RetentionPolicy::new(current.root().profile(), current.root().limits()),
        Some(current.digest()),
        anchors,
    )?;
    let candidate = CanonicalRetentionRoot::from_root(&root)?.encoded().to_vec();
    Ok(Some((
        Recipe::Successor {
            current_root,
            manifest: manifest.ok_or("successor over no manifest")?,
            candidate,
        },
        Expected::Published,
    )))
}

/// Advance only from the requested operation and prior model, never candidate output.
fn advance_model(
    store: &Store,
    model: &mut Model,
    operation: Operation,
) -> Result<(), Box<dyn Error>> {
    let namespace = match operation {
        Operation::Initial(namespace) => namespace,
        _ => Namespace::A,
    };
    let initial = candidate_bytes(store, namespace)?;
    let initial = AdmittedRetentionRoot::decode(&initial)?;
    let digest = initial.root().namespace().digest();
    let (generation, anchors) = match operation {
        Operation::Initial(_) | Operation::StaleInitial => (1, initial.root().anchors().to_vec()),
        Operation::Successor | Operation::Release | Operation::Restore => {
            let (previous, retained) = model
                .namespaces
                .get(&digest)
                .ok_or("model successor lacks predecessor")?;
            let anchors = match operation {
                Operation::Release => Vec::new(),
                Operation::Restore => initial.root().anchors().to_vec(),
                _ => retained.clone(),
            };
            (
                previous
                    .checked_add(1)
                    .ok_or("model root generation overflow")?,
                anchors,
            )
        }
        Operation::RetryLast => return Err("retry cannot advance model state".into()),
    };
    model.namespaces.insert(digest, (generation, anchors));
    model.liveness = model
        .liveness
        .checked_add(1)
        .ok_or("model liveness overflow")?;
    Ok(())
}

/// Applies one operation to the store and the model.
fn apply(store: &mut Store, model: &mut Model, operation: Operation) -> Result<(), Box<dyn Error>> {
    let Some((recipe, expected)) = recipe(store, model, operation)? else {
        return Ok(());
    };
    match (expected, recipe.publish(&mut store.authority)) {
        (Expected::AlreadyCommitted, Ok(RetentionPublicationOutcome::AlreadyCommitted)) => {}
        (Expected::Refused(refusal), Err(error)) => refusal.verify(error.as_ref())?,
        (Expected::Published, Ok(RetentionPublicationOutcome::Published)) => {
            advance_model(store, model, operation)?;
            store.last_accepted = Some(recipe);
        }
        (expected, result) => {
            return Err(
                format!("{operation:?}: model expects {expected:?}, store {result:?}").into(),
            );
        }
    }
    Ok(())
}

/// Requires the fenced reader view to agree with the model exactly.
fn verify(store: &Store, model: &Model, schedule: [Operation; 3]) -> Result<(), Box<dyn Error>> {
    let snapshot = snapshot(store)?;
    let observed: BTreeMap<_, _> = snapshot
        .manifest()
        .map(|manifest| {
            manifest
                .entries()
                .iter()
                .map(|entry| (entry.namespace(), entry.root_generation().get()))
                .collect()
        })
        .unwrap_or_default();
    let expected: BTreeMap<_, _> = model
        .namespaces
        .iter()
        .map(|(namespace, (generation, _))| (*namespace, *generation))
        .collect();
    assert_eq!(
        observed, expected,
        "manifest disagrees with the model: {schedule:?}"
    );
    let liveness = snapshot
        .retention_head()
        .map_or(0, |head| head.generation().get());
    assert_eq!(
        liveness, model.liveness,
        "liveness generation disagrees with the model: {schedule:?}"
    );
    for (namespace, (generation, anchors)) in &model.namespaces {
        let bytes = snapshot
            .retained_root(*namespace)?
            .ok_or("model namespace has no root on disk")?;
        let root = AdmittedRetentionRoot::decode(&bytes)?;
        assert_eq!(
            root.root().generation().get(),
            *generation,
            "root generation: {schedule:?}"
        );
        assert_eq!(
            root.root().anchors(),
            anchors.as_slice(),
            "anchor set disagrees with the model: {schedule:?}"
        );
    }
    Ok(())
}

/// Runs every three-operation sequence that starts with `first` in a fresh
/// migrated store each, checking the fenced view against the model after
/// every step.
fn run_sequences(first: Operation, label: &str) -> Result<(), Box<dyn Error>> {
    let template = fixture(ROOT_HEX)?;
    let mut sequences = 0_u32;
    for second in OPERATIONS {
        for third in OPERATIONS {
            let name = format!("filesystem-retention-model-{label}-{sequences}");
            let (sandbox, authority) = open_authority(&name)?;
            let mut store = Store {
                authority,
                path: sandbox.path().to_path_buf(),
                template: template.clone(),
                last_accepted: None,
            };
            let mut model = Model::default();
            for operation in [first, second, third] {
                apply(&mut store, &mut model, operation)
                    .map_err(|error| format!("{first:?} {second:?} {third:?}: {error}"))?;
                verify(&store, &model, [first, second, third])
                    .map_err(|error| format!("{first:?} {second:?} {third:?}: {error}"))?;
            }
            sequences = sequences
                .checked_add(1)
                .ok_or("model sequence index overflow")?;
        }
    }
    Ok(())
}

// Size: medium. Oracle: namespace state model plus exact rejected-operation contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_an_initial_publication_of_a_agree_with_the_model()
-> Result<(), Box<dyn Error>> {
    run_sequences(Operation::Initial(Namespace::A), "initial-a")
}

// Size: medium. Oracle: namespace state model plus exact rejected-operation contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_an_initial_publication_of_b_agree_with_the_model()
-> Result<(), Box<dyn Error>> {
    run_sequences(Operation::Initial(Namespace::B), "initial-b")
}

// Size: medium. Oracle: namespace state model plus exact rejected-operation contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_a_successor_agree_with_the_model() -> Result<(), Box<dyn Error>> {
    run_sequences(Operation::Successor, "successor")
}

// Size: medium. Oracle: namespace state model plus exact rejected-operation contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_a_retry_agree_with_the_model() -> Result<(), Box<dyn Error>> {
    run_sequences(Operation::RetryLast, "retry")
}

// Size: medium. Oracle: namespace state model plus exact rejected-operation contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_a_stale_initial_agree_with_the_model() -> Result<(), Box<dyn Error>> {
    run_sequences(Operation::StaleInitial, "stale")
}

// Size: medium. Oracle: operation-derived namespace model and exact refusal contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_release_agree_with_the_model() -> Result<(), Box<dyn Error>> {
    run_sequences(Operation::Release, "release")
}

// Size: medium. Oracle: operation-derived namespace model and exact refusal contract.
// Delete only when stronger scenario exploration subsumes these histories and diagnostics.
#[test]
fn sequences_starting_with_restore_agree_with_the_model() -> Result<(), Box<dyn Error>> {
    run_sequences(Operation::Restore, "restore")
}
