# Namespace, Format Marker, and Recovery

This page owns the exact store root namespace, the `FORMAT` marker, the
reader fence, the GC and recovery-disposition reservation, and the laws that
make a store independent of the directory it lives in.
[Direct initialization](initialization.md) owns how a store comes into
existence; every store is created by direct initialization.

## Exact filesystem namespace

The store keeps the catalog-protocol files and directories of
[Publication and reader visibility](publication.md) and adds:

```text
reader.lock
FORMAT
retention/HEAD
retention/head.next
retention/root.next
retention/manifest.next
retention/roots/<namespace-digest>/<generation>-<root-digest>.root
retention/manifests/<generation>-<manifest-digest>.manifest
gc/intent
gc/receipt
recovery/disposition.next
recovery/dispositions/<artifact-digest>.receipt
```

Catalog `HEAD` is optional: an initialized store holds none until its first
catalog publication. `retention/HEAD`, every fixed `.next` stage, `gc/intent`,
and `gc/receipt` are optional according to the exact state tables of their
pages. The catalog `head.next` is a catalog-protocol stage: writer admission
refuses it as recovery state, while readers and catalog-stage recovery admit
it, as
[catalog publication and stage recovery](catalog-publication.md) specifies.
`FORMAT.next` exists only while initialization has not committed; every
admission refuses it as `InitializationIncomplete`, and a root without `FORMAT`
refuses as `MissingFormatMarker`. Immutable-pool coordinates are
data-dependent but canonically named. Every other root or protocol-directory
entry is an unknown entry and unrecoverable ambiguity. Operations are
capability-relative and never follow links.

Version-1 admission and recovery, which refused every root holding these
entries, are deleted (#198); every open path now requires `FORMAT`.

## Format marker

`FORMAT` is exactly 96 bytes:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:STORE:V2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `96` |
| 20 | 4 | flags | `0` |
| 24 | 32 | format-definition digest | registered v2 digest |
| 56 | 4 | maximum namespace count | `4,096` |
| 60 | 4 | reserved | zero |
| 64 | 32 | checksum | BLAKE3-256 over bytes `0..64` |

The definition and checksum domains are `keep.segment-store-definition/v2\0` and
`keep.segment-store-marker-checksum/v2\0`. An unsupported, corrupt,
substituted, or same-name/different-digest marker refuses.

The format-definition digest is BLAKE3-256 of its domain followed by the exact
corpus `definition.tsv` bytes. The format-marker digest is BLAKE3-256 of
`keep.store-format-marker/v2\0` followed by all 96 marker bytes. The marker
identifies the format contract, not the software release that wrote the store.

`CanonicalStoreFormatMarker` produces the one registered marker;
`AdmittedStoreFormatMarker` admits its framing, checksum, definition, and
namespace bound. Writer admission, catalog-stage recovery, recovery inventory,
and reader admission each read `FORMAT` no-follow, bounded to its exact length,
and refuse a trailing byte. `FilesystemStoreAdmissionError::FormatMarker`
carries the decoder's typed refusal as its source.

## Reader fence

`reader.lock` is a persistent regular zero-length file. Its contents and
existence alone prove nothing.

A reader acquires a kernel-managed shared lock on `reader.lock`
before opening catalog `HEAD` or `retention/HEAD`. The returned `ReaderFence`
owns that lock for the complete snapshot lifetime. Close, drop, or process
death releases only the kernel lock and never deletes the persistent file.
A reader of a committed store with no catalog `HEAD` refuses as
`FilesystemRetentionSnapshotError::Unpublished` once it holds the fence.

GC acquires the store writer authority and then an exclusive `reader.lock`, in
that fixed order. New readers wait and existing readers drain before GC
revalidation or physical deletion. Catalog and retention publication may
proceed beside readers because they publish immutable successors and delete no
published segment.

## Portable stores

A store is not bound to the directory where it was created. The store records
no device, inode, mount, or path, and admission compares the root against
nothing persisted. A complete store copied while nothing was writing to it,
or taken from a consistent filesystem snapshot, opens on any admitted
filesystem after normal admission, and the opening process acquires fresh
writer or reader authority.

Live authority is unchanged:

- every access is capability-relative and no-follow;
- the writer lock and the reader fence verify their own lock files' identity;
- within one process, a protocol name must still identify the directory the
  same process admitted
  ([recovery directory binding](../../adr/retention-recovery-directory-binding.md)),
  so a lock file or protocol directory replaced during an operation refuses;
  and
- every content, closure, catalog, retention, and recovery validation still
  applies, so a corrupt or incomplete copy refuses exactly as the original
  would.

A copy taken while a writer publishes is outside this guarantee. Such a copy
may hold a mix of the states before and after the publication, a partly copied
file, and the writer's in-flight stages. It need not open at all, and it can
lose the newer state. The law it must still obey is the core law: it refuses
or returns exactly the bytes an identity names.

That law is tested for part of what a torn copy may contain
(`tests/single_format/torn_copy_laws.rs`, `KEEP-STORE-023`):

- every mix of two adjacent quiescent published states, file by file
  (`every_mix_of_two_quiescent_published_states_refuses_or_returns_exact_bytes`);
  and
- one file at half its length over the newer state, for each nonempty file in
  turn
  (`a_half_length_copy_of_any_one_file_over_the_newer_state_refuses_or_returns_exact_bytes`).

A refusal counts only when its typed cause is an absent or malformed artifact;
any other failure fails the law. A copy that holds an in-flight stage, a file
at another length than half, or a tear over the older state is not tested.

A copy and its original are two stores. Each takes its own locks, and their
histories diverge independently while unchanged content keeps its identity.
Copies are not replication and are never coordinated.

Admission alone does not detect an unintended but valid store, or an older valid
state of the intended one. Detecting either needs an expectation the caller
supplies and keeps outside the checked state. Every normal open therefore
requires a `StoreExpectation`, with no default:

- `FilesystemStoreAdmission::reopen`, which compares `HEAD` under the writer
  lock;
- `FilesystemCatalogSnapshot::load`, which compares what the checksummed
  `HEAD` names before it reads the catalog or any segment, with no lock and
  no fence;
- `FilesystemRetentionSnapshot::load` and `DurableSnapshot::open`, which
  compare the collected view under the reader fence; and
- `DurableStore::open`, which only records the choice: it checks nothing until
  a snapshot is pinned, and every snapshot then compares as
  `DurableSnapshot::open` does.

The choice is one of:

- `Catalog` names an exact catalog generation and digest, and any other published
  catalog, or none, refuses with the expected and observed values; and
- `NoExpectation` accepts any valid store at the path, and says so at the call site.

`Catalog` does not detect a rollback that leaves the catalog unchanged, such
as restored retention state alone. Two stores that publish the same catalog
bytes are indistinguishable, and "expected to have published nothing" has no
representation.

These routes take no expectation, so a caller of them does not get this check:

- the recovery tools (`FilesystemRecoveryAdmission::reopen` and the
  constructors that consume it, and `FilesystemRecoveryInventoryReader::open`);
- verification loads (`FilesystemCatalogSnapshot::load_for_verification` and
  `FilesystemRetentionSnapshot::load_for_verification`), which examine
  whatever state the path holds;
- `FilesystemStoreAdmission::initialize`, including when it resumes a
  marker-linked root that already holds published state
  ([States and results](initialization.md#states-and-results)); and
- the repository-task entry points, which exist only under the
  `repository-tasks` feature: the two unchecked initializers and
  `reopen_unchecked_for_repository_tasks` on `FilesystemStoreAdmission`, and
  the unchecked variants of the recovery admission and inventory reader.

### Keeping an expectation

Keep stores nothing for the caller. `ExpectedCatalog` is the caller's claim,
not a verified digest. Within one process, `ExpectedCatalog::new` takes the
generation and digest that a publication receipt or a pinned view reports. To
carry it into a later process, the caller records the generation as its `u64`
and the 32 digest bytes from `ExpectedCatalog::digest_bytes`, and rebuilds the
value with `ExpectedCatalog::from_recorded`. Where that record lives, and how
it is protected, is the caller's responsibility: a record an attacker can
rewrite protects nothing.
`an_expectation_rebuilt_from_recorded_values_admits_and_refuses_as_the_original`
shows that the rebuilt value admits and refuses exactly as the original does.

### Recovering a stale expectation

An expectation names one catalog, so the caller must record the new generation
and digest after every publication. Publication and that record are two steps.
A process that dies between them, or after gated next-head finalization moved
`HEAD`, leaves the store at the successor while the caller still holds the
predecessor. Every normal open then refuses at its expectation boundary with
a `StoreExpectationMismatch`. That refusal is the expected result, not damage.

The mismatch reports the expected catalog and, as `ObservedCatalog`, what the
store publishes now. Keep cannot tell the caller's own successor from a
different store through this interface: both are a valid store that publishes
another catalog. Recording what the mismatch observed, or opening with
`NoExpectation`, is therefore an explicit trust decision that the caller makes
with evidence from outside the store. Keep never makes it, and no open accepts
a successor on its own. An expectation that also accepts the direct successor
of the recorded catalog is not implemented.

The crash matrix pins this for the gated writer. In every `retained-catalog`
case (`KEEP-CRASH-074` to `-077`), restart builds the generation-one catalog a
caller would have recorded before the interrupted publication
(`xtask/src/durability_crash_matrix/restart/recorded_catalog.rs`). In every
case where the crash left `HEAD` at generation one, the load under that
expectation must admit before any recovery runs, beside whatever stage the
publication left. In every case, once `HEAD` names generation two, the same
expectation must refuse and report generation two.

### Root identity across restart

Earlier revisions bound a migrated store to its root directory's device and
inode and refused a reopen or fenced snapshot whose root disagreed
(`RootIdentityChanged`). That binding lived in `migration.intent`, which no
longer exists, and is retired by
[ADR-0010](../../adr/0010-one-storage-format.md). Nothing replaces
it: the laws in [Portable stores](#portable-stores) apply instead. The
[retirement ledger](retired-identifiers.md) records the requirements the
binding satisfied.

## Retention publication recovery

[Retention publication recovery](retention-recovery.md) owns fixed-stage classification, synchronization before publication, exact retained evidence, and the `KEEP-CRASH-036` through `052` process-death boundaries.

`GcRetirementIntent`, `GcRetirementReceipt`, and `RecoveryDispositionReceipt` are owned by the [GC specification](gc.md). Until issue #21 implements them, any such artifact is unsupported and refuses.

Initialization recovery is part of the initialization protocol: an interrupted
call is resumed by calling `FilesystemStoreAdmission::initialize` again,
which classifies the root and replays the idempotent remaining phases. Its
process death states are specified in [Direct initialization](initialization.md).
