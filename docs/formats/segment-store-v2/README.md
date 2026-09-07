# Durable Segment Store Version 2

`keep.segment-store/v2` is the accepted successor to
`keep.segment-store/v1`. It preserves every admitted version-1 segment,
catalog, and publication-head byte while adding explicit retention state,
reader fences, migration evidence, and reserved GC and recovery-disposition
namespaces.

ADR-0009 owns the cross-cutting retention and liveness decision. These pages
own its durable representation. The one-way migration, version-two reopen, and
forward retention publication are implemented with executable evidence;
recovery of retained retention stages, reader fencing, and collection remain
planned in issue #19, and the [requirements ledger](requirements.md) records
exactly which requirements are proven. A version-1 store remains admitted until
its owner migrates it.

## Core laws

Version 2 retains every version-1 physical law and adds these:

- the format version is explicit and cannot be inferred from path existence;
- a complete version-2 store is entered only by the specified version-1
  migration; direct version-2 initialization is undefined;
- migration is one-way, writer-authorized, durable, and recoverable from every
  documented prefix;
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

- [Retention records and publication](retention.md) owns canonical namespace,
  root-generation, manifest, retention-head, and transition rules.
- [Closure verification](closure.md) owns deterministic traversal, exact
  resource accounting, authenticated reconstruction, and closure evidence.
- [Closure corruption boundary](closure-corruption.md) owns the admitted-record
  ingress proof and its exact refusal evidence.
- [GC and disposition records](gc.md) owns the canonical planned intent,
  completion, and recovery-disposition byte grammars.
- [Migration and recovery](recovery.md) owns the exact root namespace,
  version marker, reader fence, one-way migration, crash states, GC reservation,
  recovery-disposition reservation, and restart behavior.
- [Migration crash points](migration-crash.md) owns fixed-stage publication and
  the exact process-death boundaries for migration.
- [Migration inventory](migration-inventory.md) owns the bounded canonical
  digest over preserved version-1 immutable pools.
- [Requirements and evidence](requirements.md) owns stable requirement and
  crash identifiers, evidence status, compatibility, and nonclaims.
- [Format rationale](rationale.md) records format-local choices and rejected
  alternatives.

The [version-2 golden corpus](../../../conformance/segment-store/v2/README.md)
freezes independent definition, profile, inventory, and record bytes. It is
format evidence, not production-writer evidence.

The version-1 [segment](../segment-store-v1/segment.md),
[catalog](../segment-store-v1/catalog.md), and
[publication-head](../segment-store-v1/catalog.md#publication-head)
grammars remain byte-for-byte authoritative. Version 2 does not reinterpret or
re-encode them.

## Status

The format contract is frozen by ADR-0009 and this specification. Public core
types now admit exact namespace bytes, namespace digests, root and liveness
generations, registered realization profiles, bounded closure policies,
reconstruction anchors, and semantic roots. Canonical root, manifest, and head
codecs match their independent golden records. Storage-independent transition
planning, deterministic bounded closure verification against one pinned
catalog, their combined preflight proof, and the exact 17-phase publication
vocabulary with a blocking storage capability port are available.
Storage-independent preparation derives exact canonical manifest and head
successors from coherent preflight and current-manifest evidence. Ordered
storage-port orchestration revalidates authority and returns a complete receipt.
Fresh writer-locked filesystem migration execution now publishes all canonical
fixed records and the exact empty version-2 namespace without changing
version-1 immutable bytes. Partial-prefix restart recovery, production
filesystem retention publication, immutable reader snapshots, and garbage
collection do not exist yet. Requirements still in progress in issue #19 or
issue #21 are not complete evidence. A store must refuse version-2 state until the relevant
corruption, model-based, crash-injection, recovery, and fuzz evidence exists.
