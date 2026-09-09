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

- [Retention records](retention.md) owns canonical namespace, root-generation,
  manifest, and retention-head rules.
- [Retention publication](retention-publication.md) owns closure admission and
  the generation transition.
- [Closure verification](closure.md) owns deterministic traversal, exact
  resource accounting, authenticated reconstruction, and closure evidence.
- [Closure corruption boundary](closure-corruption.md) owns the admitted-record
  ingress proof and its exact refusal evidence.
- [GC and disposition records](gc.md) owns the canonical planned intent,
  completion, and recovery-disposition byte grammars.
- [Migration and recovery](recovery.md) owns the exact root namespace,
  version marker, reader fence, migration records, GC reservation,
  recovery-disposition reservation, and restart behavior.
- [Migration protocol and recovery](migration-recovery.md) owns the ordered
  one-way migration protocol and partial-migration recovery.
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

The format contract is frozen by ADR-0009 and this specification.

Implemented with executable evidence: public core types for namespaces,
generations, realization profiles, closure policies, anchors, and roots;
canonical root, manifest, and head codecs matching their golden records;
storage-independent transition planning, bounded closure verification against
one pinned catalog, preflight, preparation, and the 17-phase publication port;
fresh writer-locked filesystem migration through all 21 phases, refusing a
version-one store that still holds a retained stage;
`FilesystemVersionTwoAdmission::reopen`, which jointly admits the marker,
intent, and receipt, binds the root's device, mount, and inode identity to the
intent, and pins the retention directories it admitted; and
`FilesystemRetentionPublicationAuthority`, which publishes initial and
successor generations against the observed head, binds this store's catalog
head and the catalog it selects, and refuses superseded candidates, retained
stages, replaced protocol directories, and every namespace or capacity
violation before mutation, each as a typed `RetentionCurrentStateRefusal`.

Retention publication recovery is implemented and proven both in-process for
every crash prefix and by the crash matrix, which kills a real writer before,
during, and after `KEEP-CRASH-036` through `052`.
Readers bind one consistent catalog, retention head, and manifest view under a
shared `ReaderFence` and verify selected roots on demand.
Not implemented: partial-prefix migration recovery and `KEEP-CRASH-053..073`,
model-based transition evidence, and garbage collection. Issue #19 owns the first four and issue #21 the last;
issue #97 owns the restart-stable root identity coordinate. A version-1 store
remains admitted until its owner migrates it, and the
[requirements ledger](requirements.md) is the authority on which requirements
are proven.
