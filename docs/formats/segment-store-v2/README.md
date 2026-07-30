# Durable Segment Store Version 2

`keep.segment-store/v2` is the accepted successor to
`keep.segment-store/v1`. It preserves every admitted version-1 segment,
catalog, and publication-head byte while adding explicit retention state,
reader fences, migration evidence, and reserved GC and recovery-disposition
namespaces.

ADR-0009 owns the cross-cutting retention and liveness decision. These pages
own its durable representation. Issue #19 must supply the production retention
implementation and executable evidence before any version-2 writer is
available. Until that implementation lands, version 1 remains the only
admitted production store.

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
  root-generation, manifest, retention-head, closure, and transition rules.
- [GC and disposition records](gc.md) owns the canonical planned intent,
  completion, and recovery-disposition byte grammars.
- [Migration and recovery](recovery.md) owns the exact root namespace,
  version marker, reader fence, one-way migration, crash states, GC reservation,
  recovery-disposition reservation, and restart behavior.
- [Migration crash points](migration-crash.md) owns fixed-stage publication and
  the exact process-death boundaries for migration.
- [Requirements and evidence](requirements.md) owns stable requirement and
  crash identifiers, evidence status, compatibility, and nonclaims.
- [Format rationale](rationale.md) records format-local choices and rejected
  alternatives.

The version-1 [segment](../segment-store-v1/segment.md),
[catalog](../segment-store-v1/catalog.md), and
[publication-head](../segment-store-v1/catalog.md#publication-head)
grammars remain byte-for-byte authoritative. Version 2 does not reinterpret or
re-encode them.

## Status

The format contract is frozen by ADR-0009 and this specification. Requirements
marked **Planned in #19** or **Planned in #21** are not implementation evidence.
A store must refuse version-2 state until the relevant parser, corruption,
golden-format, model-based, crash-injection, recovery, and fuzz evidence is
implemented.
