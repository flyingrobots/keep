# Catalog Publication and Stage Recovery

This page owns segment ingestion and catalog publication on a committed store,
and recovery of the catalog-protocol stages that publication leaves. It adds
no byte format: the segment, catalog, and publication-head grammars are those
of [Segment bytes](segment.md) and [Catalog and publication-head
bytes](catalog.md), and the retention records stay those of [Retention
records](retention.md). Stores are created by [direct
initialization](initialization.md).

## Supported envelope

- One publication stages at most one new segment, through the writer's own
  `staging/current.seg`, or none. A candidate that needs more than one new
  segment is outside this envelope. Segment rollover ([#121](https://github.com/flyingrobots/keep/issues/121))
  is not claimed.
- The caller supplies the complete candidate catalog and every segment it
  references. The catalog, segment, and head bounds apply unchanged.
- Retention evidence is bounded by one manifest of at most 4,096 entries and
  one root at a time. Each closure is bounded by its root's admitted limits.
- The store must be a committed root. A freshly initialized store
  has no `HEAD`; its first publication uses the uninitialized expectation,
  has no predecessor, and is admitted only when `segments`, `catalogs`, and
  both retention pools are empty.

## Authority

`FilesystemStoreAdmission::reopen` admits the root and acquires the writer
lock. `FilesystemCatalogPublisher::open` consumes that admission. It
pins the store root, `staging`, `segments`, `catalogs`, `retention`,
`retention/roots`, and `retention/manifests`. A bare writer lock cannot open
it.

The publisher implements only `RetainedCatalogPublicationStorage`, so it
publishes only through `publish_retained_catalog_generation`. Its stage type,
`FilesystemRetainedCatalogStages`, has private fields and no public
constructor. It is reachable only through `catalog_stages`, which requires the
gate's `RetainedClosureAdmission`. The ungated `publish_catalog_generation`
therefore cannot drive the gated stages.

The stage transitions are the one `FilesystemCatalogStages` implementation
that [Publication and reader visibility](publication.md) specifies; the gated
stages forward to it unchanged.

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

`FilesystemRecoveryAdmission::reopen` applies the writer admission
with one difference: the root may also hold `head.next`. It requires the
platform profile, the writer lock, the exact store namespace and protocol
directories, and the canonical `FORMAT` marker. The recovery tools take it through their
`open` constructors:

- `FilesystemRecoveryStageDiscarder`;
- `FilesystemRecoveryStageCompleter`;
- `FilesystemRecoverySegmentResumer`; and
- `FilesystemRecoveryNextHeadFinalizer`.

The read-only `FilesystemRecoveryInventoryReader::open` applies
the same namespace and marker laws without the writer lock.

`classify_recovery_names` classifies `reader.lock`, `FORMAT`,
`retention`, `gc`, and `recovery` as known roles and requires each of them. It
does not require `HEAD`: the first publication of an initialized store can be
interrupted before `HEAD` exists. `FORMAT.next` is an unknown entry.
Store admission owns the contents of the three protocol directories, and
retention recovery owns `retention`.

Stage discard, completion, and segment resume execute the executors of
[Catalog-protocol stage recovery](stage-recovery.md) unchanged. None of them changes `HEAD`.

### Gated next-head finalization

A finalizer retains the admitted retention directories. Before every
candidate verification that can precede `HEAD` replacement, including the one
immediately before the rename, it runs the forward gate's retained-closure
admission against the staged candidate catalog. A candidate that would lose
retained content refuses with
`RecoveryNextHeadFinalizationStorageError::RetainedClosure`. The error carries
the gate's exact refusal, and `head.next`, the pools, and `HEAD` stay
unchanged. A retained retention stage, or a retention head absent while the
pools hold artifacts, refuses in the same way.

## Roots without the marker

- Writer admission, recovery admission, the recovery inventory, catalog
  restart, and the retention and durable readers refuse a root without
  `FORMAT` as `MissingFormatMarker`, even when every other entry and record of
  a published, retained store is intact.
- `classify_recovery_names` requires every root entry of an
  initialized store and refuses a missing one as a missing required entry.
- A segment stage sealed under one publisher's authority refuses selection by
  any other publisher.

Version-1 admission, the version-1 publisher, and the version-1 recovery tools
are deleted (#198), so no cross-version entry remains.

## Process-death matrix

`cargo xtask durability-crash-matrix --sequence retained-catalog` runs the
production writer in a child process. The child blocks at the
selected boundary through an observer on the writer's own stage type. Restart
then requires the exact stage residue, recovers it with the public
recovery tools, retains the ingested blob, and reads both blobs through
`DurableStore`.

<!-- markdownlint-disable MD013 -->

| ID | Boundary | Positions | Expected residue and recovery | Status |
| --- | --- | --- | --- | --- |
| `KEEP-CRASH-074` | segment link | before, during, after | sealed `current.seg`; completion, then republication | Implemented |
| `KEEP-CRASH-075` | catalog-stage creation | before, during, after | none and republication before; empty `current.cat`, discard, and republication otherwise | Implemented |
| `KEEP-CRASH-076` | `head.next` synchronization | before, during, after | complete `head.next`; gated finalization | Implemented |
| `KEEP-CRASH-077` | `HEAD` replacement | before, during, after | complete `head.next` and gated finalization before; generation two published otherwise | Implemented |
| `KEEP-CRASH-078` | recovery stage removal under recovery admission | before, during, after | covered by `KEEP-CRASH-027`–`030`, which now run under recovery admission | Retired (#198) |
| `KEEP-CRASH-079` | recovery stage completion link | before, during, after | the completion executor's law under recovery admission | Planned |
| `KEEP-CRASH-080` | gated finalization replacement | before, during, after | finalized or retryable `head.next`; the gate reruns on retry | Planned |

<!-- markdownlint-enable MD013 -->

The implemented rows are 12 cases. CI runs them with the complete matrix in
debug and release builds. They establish recovery after process death; they do
not simulate power loss. In-process laws add an injected storage-port failure
after segment link, catalog-stage creation, and `head.next` synchronization
(`filesystem_catalog_recovery_tests`), plus hand-staged complete
`head.next` candidates that preserve or drop retained content
(`tests/catalog_writer.rs`). Every other boundary of the shared stages is
covered by `KEEP-CRASH-001` to `030`, which run on stores created by direct
initialization.
