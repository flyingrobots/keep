# Catalog Publication and Stage Recovery

This page owns segment ingestion and catalog publication on a committed
`keep.segment-store/v2` store, and recovery of the catalog-protocol
stages that publication leaves. It adds no byte format: the segment, catalog,
and publication-head grammars stay the version-1 grammars, and the retention
records stay those of [Retention records](retention.md). Version-2 stores are
created by [direct initialization](initialization.md).

## Supported envelope

- One publication stages at most one new segment, through the writer's own
  `staging/current.seg`, or none. A candidate that needs more than one new
  segment is outside this envelope. Segment rollover ([#121](https://github.com/flyingrobots/keep/issues/121))
  is not claimed.
- The caller supplies the complete candidate catalog and every segment it
  references. The version-1 catalog, segment, and head bounds apply unchanged.
- Retention evidence is bounded by one manifest of at most 4,096 entries and
  one root at a time. Each closure is bounded by its root's admitted limits.
- The store must be a committed version-2 root. A freshly initialized store
  has no `HEAD`; its first publication uses the uninitialized expectation,
  has no predecessor, and is admitted only when `segments`, `catalogs`, and
  both retention pools are empty.

## Authority

`FilesystemVersionTwoAdmission::reopen` admits the root and acquires the writer
lock. `FilesystemVersionTwoCatalogPublisher::open` consumes that admission. It
pins the store root, `staging`, `segments`, `catalogs`, `retention`,
`retention/roots`, and `retention/manifests`. Version-1 writer authority
cannot open it.

The publisher implements only `RetainedCatalogPublicationStorage`, so it
publishes only through `publish_retained_catalog_generation`. Its stage type,
`FilesystemVersionTwoCatalogStages`, has private fields and no public
constructor. It is reachable only through `catalog_stages`, which requires the
gate's `RetainedClosureAdmission`. The ungated `publish_catalog_generation`
therefore cannot drive version-2 stages.

The stage transitions are the version-1 implementation. One
`FilesystemCatalogStages` value runs them under either writer authority, so the
two formats cannot diverge in their segment, catalog, or head steps.

Publication never takes the reader fence. Readers hold `reader.lock` shared
and select only `HEAD`. A retained or in-flight catalog `head.next` is admitted
and ignored by readers, so a pinned view keeps its generation while a
publication runs beside it. Only GC takes the fence exclusively.

## Forward protocol

Under one writer lock, one call to `publish_retained_catalog_generation`:

1. admits the candidate catalog, its successor relationship, and its
   generated head without I/O;
2. requires `retention`, `retention/roots`, and `retention/manifests` to still
   be the pinned directories, refuses any fixed retention stage
   (`root.next`, `manifest.next`, `head.next` under `retention`), and admits the
   retention namespace and both pools;
3. reads `retention/HEAD` no-follow and bounded to 144 bytes plus one
   trailing-byte probe. An absent head is reported only when both pools are
   empty; otherwise the call refuses with `HeadAbsentWithArtifacts`;
4. reads the selected manifest, bounded by the head's declared length, and
   every root the manifest names, bounded by the root grammar's maximum
   length. The gate decodes and binds each record and verifies each root's
   closure against the candidate;
5. verifies the current catalog state: no `head.next`, no `current.cat`, the
   expected `HEAD`, and a `current.seg` only when this call selected it;
6. for a selected segment: links `current.seg` into `segments` without
   replacement, reverifies the pool entry, synchronizes `segments`, removes
   `current.seg`, and synchronizes `staging`;
7. exclusively creates `staging/current.cat`, writes, flushes, and
   synchronizes the canonical catalog, links it into `catalogs` without
   replacement, reverifies it, synchronizes `catalogs`, removes the stage, and
   synchronizes `staging`;
8. exclusively creates `head.next`, writes, flushes, and synchronizes the
   canonical head, then reopens and verifies its complete transitive view;
9. rereads `retention/HEAD` and requires it to equal the admitted head; and
10. atomically renames `head.next` over `HEAD` and synchronizes the store root.

Steps 1 to 4 complete before the first catalog storage call. A refusal there
leaves no stage, pool entry, or head change. The caller's sealed
`current.seg`, created before the call, stays for explicit recovery.

## Crash states and recovery

Every state below is ordinary recovery evidence. Nothing is repaired
automatically, and recovery never removes or rewrites an immutable pool entry.

| Interrupted at | Durable residue | Recovery |
| --- | --- | --- |
| Before step 6, including any gate refusal | the caller's `current.seg`, complete or incomplete | complete a complete stage into `segments`; discard an incomplete one; republish |
| Step 6, after the link | `current.seg` and an identical pool entry | completion verifies the existing entry and removes the stage; republish |
| After step 6, before `current.cat` exists | no stage; the new segment is pooled | republish with no segment selection |
| Step 7, `current.cat` incomplete | an incomplete `current.cat` | discard; republish |
| Step 7, `current.cat` complete | a complete `current.cat`, possibly also pooled | completion links or verifies it and removes the stage; republish |
| Step 8, `head.next` incomplete | an incomplete `head.next` | discard; republish |
| Step 8 or 9, `head.next` complete | a complete `head.next` naming a pooled catalog | gated next-head finalization |
| Step 10, after the rename | `HEAD` names the candidate | none; a byte-identical retry returns already published |

A retained retention stage refuses publication at step 2. Retention recovery
must resolve it first.

### Recovery admission

`FilesystemVersionTwoRecoveryAdmission::reopen` applies the writer admission
with one difference: the root may also hold `head.next`. It requires the
platform profile, the writer lock, the exact version-2 namespace and protocol
directories, and the canonical `FORMAT` marker. The recovery tools take it through their
`open_version_two` constructors:

- `FilesystemRecoveryStageDiscarder`;
- `FilesystemRecoveryStageCompleter`;
- `FilesystemRecoverySegmentResumer`; and
- `FilesystemRecoveryNextHeadFinalizer`.

The read-only `FilesystemRecoveryInventoryReader::open_version_two` applies
the same namespace and marker laws without the writer lock.

`classify_version_two_recovery_names` classifies `reader.lock`, `FORMAT`,
`retention`, `gc`, and `recovery` as known roles and requires each of them. It
does not require `HEAD`: the first publication of an initialized store can be
interrupted before `HEAD` exists. `FORMAT.next` is an unknown entry.
Version-2 admission owns the contents of the three protocol
directories, and retention recovery owns `retention`.

Stage discard, completion, and segment resume execute the version-1
executors unchanged. None of them changes `HEAD`.

### Gated next-head finalization

A version-2 finalizer retains the admitted retention directories. Before every
candidate verification that can precede `HEAD` replacement, including the one
immediately before the rename, it runs the forward gate's retained-closure
admission against the staged candidate catalog. A candidate that would lose
retained content refuses with
`RecoveryNextHeadFinalizationStorageError::RetainedClosure`. The error carries
the gate's exact refusal, and `head.next`, the pools, and `HEAD` stay
unchanged. A retained retention stage, or a retention head absent while the
pools hold artifacts, refuses in the same way.

## Cross-version entry

- Version-1 publisher admission, version-1 inventory, and every version-1
  recovery tool keep refusing a version-2 root through
  `admit_recoverable` or the exact published namespace.
- Version-2 writer admission, version-2 recovery admission, and the version-2
  inventory refuse a version-1 root because its marker, records, and reader
  fence are absent.
- `classify_recovery_names` refuses every version-2 root entry as unexpected.
  `classify_version_two_recovery_names` refuses a version-1 inventory as
  missing a required entry.
- A segment stage sealed under one publisher's authority, version-1 or
  version-2, refuses selection by any other publisher.

## Process-death matrix

`cargo xtask durability-crash-matrix --sequence version-two-catalog` runs the
production version-2 writer in a child process. The child blocks at the
selected boundary through an observer on the writer's own stage type. Restart
then requires the exact stage residue, recovers it with the public version-2
recovery tools, retains the ingested blob, and reads both blobs through
`DurableStore`.

<!-- markdownlint-disable MD013 -->

| ID | Boundary | Positions | Expected residue and recovery | Status |
| --- | --- | --- | --- | --- |
| `KEEP-CRASH-074` | segment link | before, during, after | sealed `current.seg`; completion, then republication | Implemented |
| `KEEP-CRASH-075` | catalog-stage creation | before, during, after | none and republication before; empty `current.cat`, discard, and republication otherwise | Implemented |
| `KEEP-CRASH-076` | `head.next` synchronization | before, during, after | complete `head.next`; gated finalization | Implemented |
| `KEEP-CRASH-077` | `HEAD` replacement | before, during, after | complete `head.next` and gated finalization before; generation two published otherwise | Implemented |
| `KEEP-CRASH-078` | version-2 recovery stage removal | before, during, after | the version-1 discard executor's law under version-2 admission | Planned |
| `KEEP-CRASH-079` | version-2 recovery stage completion link | before, during, after | the version-1 completion executor's law under version-2 admission | Planned |
| `KEEP-CRASH-080` | version-2 gated finalization replacement | before, during, after | finalized or retryable `head.next`; the gate reruns on retry | Planned |

<!-- markdownlint-enable MD013 -->

The implemented rows are 12 cases. CI runs them with the complete matrix in
debug and release builds. They establish recovery after process death; they do
not simulate power loss. In-process laws add an injected storage-port failure
after segment link, catalog-stage creation, and `head.next` synchronization
(`filesystem_version_two_catalog_recovery_tests`), plus hand-staged complete
`head.next` candidates that preserve or drop retained content
(`tests/version_two_catalog_writer.rs`). Other boundaries rely on the shared
version-1 stage implementation and its `KEEP-CRASH-001` to `030` evidence.
