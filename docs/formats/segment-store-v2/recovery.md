# Namespace, Format Marker, and Recovery

This page owns the exact version-2 root namespace, the `FORMAT` marker, the
reader fence, the GC and recovery-disposition reservation, and the laws that
make a store independent of the directory it lives in.
[Direct initialization](initialization.md) owns how a store comes into
existence; every version-2 store is created by direct initialization.

## Exact filesystem namespace

Version 2 keeps the version-1 files and directories and adds:

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
pages. The version-1 catalog `head.next` stays a version-1 file: writer
admission refuses it as recovery state, while readers and version-2
catalog-stage recovery admit it, as
[catalog publication and stage recovery](catalog-publication.md) specifies.
`FORMAT.next` exists only while initialization has not committed; every
admission refuses it as `InitializationIncomplete`, and a root without `FORMAT`
refuses as `MissingFormatMarker`. Immutable-pool coordinates are
data-dependent but canonically named. Every other root or protocol-directory
entry is an unknown entry and unrecoverable ambiguity. Operations are
capability-relative and never follow links.

Version-1 admission refuses every root that holds `FORMAT`, `FORMAT.next`,
`reader.lock`, `retention`, `gc`, or `recovery`. Version-1 recovery refuses
the same entries before it pins a pool.

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
and refuse a trailing byte. `FilesystemPlatformAdmissionError::FormatMarker`
carries the decoder's typed refusal as its source.

## Reader fence

`reader.lock` is a persistent regular zero-length file. Its contents and
existence alone prove nothing.

A version-2 reader acquires a kernel-managed shared lock on `reader.lock`
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

A copy and its original are two stores. Each takes its own locks, and their
histories diverge independently while unchanged content keeps its identity.
Copies are not replication and are never coordinated.

Admission does not detect an unintended but valid store, or an older valid
state of the intended one. Detecting either needs an expectation the caller
supplies and keeps outside the checked state, such as an exact catalog
generation and digest. Keep does not claim either check.

### Root identity across restart

Earlier revisions bound a migrated store to its root directory's device and
inode and refused a reopen or fenced snapshot whose root disagreed
(`RootIdentityChanged`). That binding lived in `migration.intent`, which no
longer exists, and is retired by
[ADR-0010](https://github.com/flyingrobots/keep/pull/199). Nothing replaces
it: the laws in [Portable stores](#portable-stores) apply instead. The
[retirement ledger](retired-identifiers.md) records the requirements the
binding satisfied.

## Retention publication recovery

[Retention publication recovery](retention-recovery.md) owns fixed-stage classification, synchronization before publication, exact retained evidence, and the `KEEP-CRASH-036` through `052` process-death boundaries.

`GcRetirementIntent`, `GcRetirementReceipt`, and `RecoveryDispositionReceipt` are owned by the [GC specification](gc.md). Until issue #21 implements them, any such artifact is unsupported and refuses.

Initialization recovery is part of the initialization protocol: an interrupted
call is resumed by calling `FilesystemVersionTwoAdmission::initialize` again,
which classifies the root and replays the idempotent remaining phases. Its
process death states are specified in [Direct initialization](initialization.md).
