# Keep Store Format

`keep.segment-store/v2` is the one store format Keep creates, admits, writes,
and recovers. It is the successor to `keep.segment-store/v1`: it keeps the
version-1 segment, catalog, and publication-head grammars byte for byte and
adds the `FORMAT` marker, the reader fence, explicit retention state, and
reserved GC and recovery-disposition namespaces. Version 1 is no longer a
store format: its admission, initializer, and recovery entry points were
deleted in [#198](https://github.com/flyingrobots/keep/issues/198), and every
open path refuses a root without the `FORMAT` marker. No conversion from
version 1 exists.

ADR-0005 records the segment, catalog, and publication-head protocol.
ADR-0009 owns the retention and liveness decision.
[ADR-0010](https://github.com/flyingrobots/keep/pull/199) makes this the only
storage format and replaces migration with direct initialization. These pages
are one normative set; every page describes the behaviour of `main`, and the
two requirement ledgers name the test behind each claim.

Segment writing and verified reading are implemented in issue #15. Catalog
generation, writer-locked publication, and immutable restart snapshots are
implemented in issue #16. Explicit stage recovery and the process-death crash
matrix are implemented in issue #17; since #198 they run on stores created
by direct initialization.

## Core law

For a given logical content identity, Keep returns exactly the bytes named by
that identity or refuses.

The catalog protocol adds these physical laws:

- file existence proves no identity, completeness, publication, retention, or
  durability claim;
- only one verified publication head selects one verified immutable catalog;
- only that catalog maps logical identities to immutable physical records;
- a segment is immutable after its complete seal is synchronized;
- a generation is published only after all referenced immutable artifacts are
  synchronized and the new head replacement is directory-synchronized; and
- uncertain, conflicting, malformed, unsupported, or corrupt state is
  represented and refused, never guessed or silently repaired.

Physical locations and digests are not `BlobId`, `ChunkId`, or `LayoutId`.
Rechunking and later compaction may move physical evidence without moving
logical identity.

## Store laws

The store adds these laws to the catalog protocol:

- the format version is explicit and cannot be inferred from path existence;
- a store is created only by direct initialization, a recoverable state
  machine under writer authority with one durable commit point, and it does
  not admit before that point;
- an initialized store with no published catalog is a defined, admitted
  state;
- nothing persisted binds a store to its directory: a quiescent copy is the
  same store, and live authority comes from the opening process;
- retention authority exists only through one verified retention head and its
  complete immutable manifest;
- each manifest binds every admitted namespace to one exact root generation
  and canonical digest;
- root closure is derived from a verified catalog, never from paths, caller
  claims, recent access, or application identity;
- catalog publication preserves every current retained closure before
  replacing the catalog head;
- readers acquire the reader fence before opening the catalog head; and
- ambiguous, corrupt, missing, excessive, or unsupported evidence refuses
  before mutation.

## Normative pages

Record grammars:

- [Segment bytes](segment.md) owns canonical primitives, record framing, and
  immutable segment bytes.
- [Catalog and publication-head bytes](catalog.md) owns catalog generations,
  publication-head framing, and fixed bounds.
- [Retention records](retention.md) owns canonical namespace, root-generation,
  manifest, and retention-head rules.
- [GC and disposition records](gc.md) owns the canonical planned intent,
  completion, and recovery-disposition byte grammars.

Store lifecycle:

- [Direct initialization](initialization.md) owns how a store comes into
  existence: the forward protocol, its commit point, every interrupted state
  and its one result, and the initialization process-death boundaries.
- [Format, namespace, and portability](recovery.md) owns the exact root
  namespace, the `FORMAT` marker, the reader fence, the GC and
  recovery-disposition reservation, and the laws that keep a store portable.
- [Publication and reader visibility](publication.md) owns the physical
  namespace, forward protocol, writer exclusion, and reader snapshots of the
  segment, catalog, and head transitions.
- [Catalog publication and stage recovery](catalog-publication.md) owns the
  gated segment and catalog writer, its crash states, and catalog-stage
  recovery, including gated next-head finalization.
- [Catalog-protocol stage recovery and platform contract](stage-recovery.md)
  owns recovery classification, refusal precedence, the stage recovery tools,
  the catalog-protocol crash matrix, and supported filesystem semantics.
- [Retention publication](retention-publication.md) owns closure admission and
  the generation transition.
- [Retention publication recovery](retention-recovery.md) owns interrupted
  stages, restart publication, and retention process-death boundaries.
- [Closure verification](closure.md) owns deterministic traversal, exact
  resource accounting, authenticated reconstruction, and closure evidence.
- [Closure corruption boundary](closure-corruption.md) owns the admitted-record
  ingress proof and its exact refusal evidence.

Evidence and decisions:

- [Requirements and evidence](requirements.md) owns the store, retention,
  initialization, and portability requirement and crash identifiers, evidence
  status, compatibility, and nonclaims.
- [Catalog-protocol requirements, compatibility, and evidence](catalog-protocol-requirements.md)
  owns the segment, catalog, and stage-recovery requirement ledger, security
  nonclaims, and golden evidence.
- [Retired identifiers](retired-identifiers.md) maps every retired
  requirement and crash identifier to its last evidence.
- [Format rationale](rationale.md) and the
  [catalog-protocol rationale](catalog-protocol-rationale.md) record
  format-local choices and rejected alternatives.

No page is independently optional. An implementation conforms only when it
satisfies the complete linked protocol, the
[catalog-protocol transition ledger](../../../conformance/segment-store/v1/transitions.tsv),
and the
[initialization transition ledger](../../../conformance/segment-store/v2/transitions.tsv).

## Golden corpora

The [segment, catalog, and head corpus](../../../conformance/segment-store/v1/README.md)
and the [store corpus](../../../conformance/segment-store/v2/README.md) freeze
independent record bytes, the format definition, the realization profile, and
the marker. Their directory names carry the record-grammar version that each
artifact's own magic and version fields name, so they are not renamed with
these pages. They are format evidence, not production-writer evidence.

## Status

The format contract is frozen by ADR-0005, ADR-0009, ADR-0010, and these
pages.

Implemented with executable evidence: the segment, catalog, and head codecs
and the shared catalog stages; public core types for namespaces, generations,
realization profiles, closure policies, anchors, and roots; canonical root,
manifest, head, and marker codecs matching their golden records;
storage-independent transition planning, bounded closure verification against
one pinned catalog, preflight, preparation, and the 17-phase publication
port; `FilesystemStoreAdmission::initialize`, which creates or resumes a
store through the initialization protocol and its 41-case process-death
matrix; `FilesystemStoreAdmission::reopen`, which admits the committed
namespace and the marker and pins the retention directories it admitted;
`FilesystemCatalogPublisher`, which ingests one segment and publishes a
catalog through the retained-closure gate
([#125](https://github.com/flyingrobots/keep/issues/125)), including the first
catalog of an initialized store; the stage recovery tools, whose next-head
finalization runs the same admission before replacing `HEAD`; and
`FilesystemRetentionPublicationAuthority`, which publishes and recovers
retention generations and refuses every namespace or capacity violation before
mutation.

`FilesystemRetentionSnapshot` binds a catalog, retention head and manifest
under a shared reader fence and verifies selected roots on demand.
`DurableStore` composes that view with authenticated whole-blob and
exact-range reads ([PR #164](https://github.com/flyingrobots/keep/pull/164));
the [read contract](../../invariants/authenticated-reconstruction/README.md)
records proof and allocation limits.
[Durable verification reports](../../invariants/verification/README.md) are
delivered in [PR #165](https://github.com/flyingrobots/keep/pull/165).

Incomplete retention stages are preserved pending explicit disposition;
automatic disposal remains deferred in
[#155](https://github.com/flyingrobots/keep/issues/155). Writer authority
coordinates cooperating writers in a managed namespace; it does not isolate
arbitrary out-of-band filesystem mutation or make pathname unlink conditional
on inode identity. Production durable ingestion
([#82](https://github.com/flyingrobots/keep/issues/82)), garbage collection,
and compaction ([#21](https://github.com/flyingrobots/keep/issues/21)) remain
absent from main.

The `KEEP-CRASH-036..052` initial-publication process-death sequence includes
independent recovered-reader checks; its
[evidence receipt](../../testing-evidence/retention-crash-reader-oracle.md)
records the assertions, calibration, and scope. The
[landing ledger](../../testing-evidence/retention-landing.md) records the
accepted scope and evidence for merged PR #99.
