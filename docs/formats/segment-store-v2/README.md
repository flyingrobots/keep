# Durable Segment Store Version 2

`keep.segment-store/v2` is the successor to `keep.segment-store/v1` and the
one store format Keep creates. It keeps the version-1 segment, catalog, and
publication-head grammars byte for byte and adds the `FORMAT` marker, the
reader fence, explicit retention state, and reserved GC and
recovery-disposition namespaces.

ADR-0009 owns the cross-cutting retention and liveness decision.
[ADR-0010](https://github.com/flyingrobots/keep/pull/199) makes this the only
storage format and replaces migration with direct initialization. These pages
own the durable representation.

The [requirements ledger](requirements.md) records evidence and remaining obligations; the laws below are normative requirements, not a claim that every corresponding runtime surface exists.

## Core laws

Version 2 retains every version-1 physical law and adds these:

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
- readers acquire the version-2 reader fence before opening the catalog head;
  and
- ambiguous, corrupt, missing, excessive, or unsupported evidence refuses
  before mutation.

## Normative pages

The following pages form one protocol:

- [Direct initialization](initialization.md) owns how a store comes into
  existence: the forward protocol, its commit point, every interrupted state
  and its one result, and the initialization process-death boundaries.
- [Format, namespace, and portability](recovery.md) owns the exact root
  namespace, the `FORMAT` marker, the reader fence, the GC and
  recovery-disposition reservation, and the laws that keep a store portable.
- [Retention records](retention.md) owns canonical namespace, root-generation,
  manifest, and retention-head rules.
- [Retention publication](retention-publication.md) owns closure admission and
  the generation transition.
- [Catalog publication and stage recovery](catalog-publication.md) owns the
  version-2 segment and catalog writer, its crash states, and version-2
  catalog-stage recovery, including gated next-head finalization.
- [Retention publication recovery](retention-recovery.md) owns interrupted stages, restart publication, and retention process-death boundaries.
- [Closure verification](closure.md) owns deterministic traversal, exact
  resource accounting, authenticated reconstruction, and closure evidence.
- [Closure corruption boundary](closure-corruption.md) owns the admitted-record
  ingress proof and its exact refusal evidence.
- [GC and disposition records](gc.md) owns the canonical planned intent,
  completion, and recovery-disposition byte grammars.
- [Requirements and evidence](requirements.md) owns stable requirement and
  crash identifiers, evidence status, compatibility, and nonclaims.
- [Retired identifiers](retired-identifiers.md) maps every retired
  requirement and crash identifier to its last evidence.
- [Format rationale](rationale.md) records format-local choices and rejected
  alternatives.

The [version-2 golden corpus](../../../conformance/segment-store/v2/README.md)
freezes independent definition, profile, marker, and retention record bytes.
It is format evidence, not production-writer evidence.

The version-1 [segment](../segment-store-v1/segment.md),
[catalog](../segment-store-v1/catalog.md), and
[publication-head](../segment-store-v1/catalog.md#publication-head)
grammars remain byte-for-byte authoritative. Version 2 does not reinterpret or
re-encode them.

## Status

The format contract is frozen by ADR-0009, ADR-0010, and this specification.

Implemented with executable evidence: public core types for namespaces,
generations, realization profiles, closure policies, anchors, and roots;
canonical root, manifest, head, and marker codecs matching their golden
records; storage-independent transition planning, bounded closure
verification against one pinned catalog, preflight, preparation, and the
17-phase publication port; `FilesystemVersionTwoAdmission::initialize`, which
creates or resumes a store through the initialization protocol and its 41-case
process-death matrix; `FilesystemVersionTwoAdmission::reopen`, which admits
the committed namespace and the marker and pins the retention directories it
admitted; and `FilesystemRetentionPublicationAuthority`, which publishes
initial and successor generations against the observed head, binds this
store's catalog head and the catalog it selects, and refuses superseded
candidates, retained stages, replaced protocol directories, and every
namespace or capacity violation before mutation, each as a typed
`RetentionCurrentStateRefusal`.

Retention publication recovery, fenced retention snapshots and model-based transition evidence are implemented on main.

The [landing ledger](../../testing-evidence/retention-landing.md) records the accepted scope and evidence for merged PR #99.

Incomplete retention stages are preserved pending explicit disposition; automatic disposal remains deferred in [#155](https://github.com/flyingrobots/keep/issues/155).

Execution failures preserve the typed cause and report known effects separately from uncertain effects or durability, as specified by the [retention recovery contract](retention-recovery.md).

Writer authority coordinates cooperating writers in a managed namespace; it does not isolate arbitrary out-of-band filesystem mutation or make pathname unlink conditional on inode identity.

`FilesystemRetentionSnapshot` binds a catalog, retention head and manifest under a shared reader fence and verifies selected roots on demand. `DurableStore` composes that view with authenticated whole-blob and exact-range reads, delivered in [PR #164](https://github.com/flyingrobots/keep/pull/164); the [read contract](../../invariants/authenticated-reconstruction/README.md) records proof and allocation limits.

The candidate-catalog retained-closure gate is implemented over its storage port ([#125](https://github.com/flyingrobots/keep/issues/125)). `FilesystemVersionTwoCatalogPublisher` ingests one segment and publishes a catalog successor through it, including the first catalog of an initialized store, and version-two next-head finalization runs the same admission before replacing `HEAD` ([#198](https://github.com/flyingrobots/keep/issues/198)); see [catalog publication and stage recovery](catalog-publication.md). Verifying a new retention root against the current catalog is a different operation.

[Durable verification reports](../../invariants/verification/README.md) are delivered in [PR #165](https://github.com/flyingrobots/keep/pull/165), with explicit subject/depth evidence and typed non-success outcomes.

Production durable ingestion ([#82](https://github.com/flyingrobots/keep/issues/82)) builds on the version-two segment and catalog writer but is not delivered by it. Durable ingestion, garbage collection and compaction ([#21](https://github.com/flyingrobots/keep/issues/21)) remain absent from main; those prepared portions of unmerged PR #107 are not delivered APIs here.

The `KEEP-CRASH-036..052` initial-publication process-death sequence includes independent recovered-reader checks; its [evidence receipt](../../testing-evidence/retention-crash-reader-oracle.md) records the assertions, calibration, and scope.

Version-1 admission is deleted ([#198](https://github.com/flyingrobots/keep/issues/198)): this is the only format, and every open path refuses a root without its `FORMAT` marker. No conversion from version 1 exists.
