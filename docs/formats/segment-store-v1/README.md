# Durable Segment Store Version 1

`keep.segment-store/v1` is the accepted physical protocol for Keep's first
durable segment store. It specifies bytes, publication, crash states, reader
visibility, and recovery as one contract.

ADR-0005 records the cross-cutting decision. These pages are a protocol
commitment. Segment writing and verified reading are implemented in issue #15.
Catalog generation, writer-locked publication, and immutable restart snapshots
are implemented in issue #16. Store initialization and complete executable
crash and recovery evidence remain owned by issue #17.

## Core law

For a given logical content identity, Keep returns exactly the bytes named by
that identity or refuses.

Durable version 1 adds these physical laws:

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
Rechunking, migration, and later compaction may move physical evidence without
moving logical identity.

## Normative pages

The following pages form one versioned protocol:

- [Segment bytes](segment.md) owns canonical primitives, record framing, and
  immutable segment bytes.
- [Catalog and publication-head bytes](catalog.md) owns catalog generations,
  publication-head framing, and fixed bounds.
- [Publication and reader visibility](publication.md) owns the physical
  namespace, forward protocol, writer exclusion, and reader snapshots.
- [Recovery and platform contract](recovery.md) owns recovery classification,
  refusal precedence, and supported filesystem semantics.
- [Requirements, compatibility, and evidence](requirements.md) owns the
  requirement ledger, compatibility boundary, security nonclaims, and golden
  evidence.
- The [colocated rationale](rationale.md) records format-local tradeoffs.

No page is independently optional. A version-1 implementation conforms only
when it satisfies the complete linked protocol and the
[durable transition ledger](../../../conformance/segment-store/v1/transitions.tsv).
