# Durable Ingestion

`DurableWriter` is the write half of the
[durable store](README.md): one bounded pass from an unknown-length source
into a version-two store, with deduplication against the pinned catalog and
publication through the version-one catalog protocol. It implements the
[content-store port](../content-store/README.md)'s `ContentStaging`, so
code written against the port runs on the reference store in tests and on
disk in production.

## Contract

`DurableWriter::open(admission, root, policy)` takes writer authority over
one admitted version-two root and holds it, through its catalog publisher,
for the writer's lifetime. Readers are not excluded: a pinned
`DurableSnapshot` keeps reading its generation while the writer publishes
the next.

`stage(source, limits)` reads the source exactly once through the reference
store's streaming core (`FastCDC` boundaries, `BlobHasher`, per-chunk
identity verification, the entry and byte limits) and hands each chunk to
the durable sink:

- when the pinned catalog holds a record under the chunk's identity, its
  payload is compared byte for byte with the chunk; a difference refuses
  with `ChunkRepresentation`, and agreement counts the chunk as reused;
- otherwise the chunk is appended to `staging/current.seg` as a chunk
  record the moment it is produced. The stage is created on the first new
  chunk, so a source the catalog already holds creates nothing;
- after the last byte the canonical layout record is derived and appended,
  unless the catalog already holds it byte-identically
  (`LayoutRepresentation` otherwise); the stage is sealed and closed.

The blob is never materialized. Beyond the streaming core's fixed scratch,
staging holds the layout spans bounded by the entry limit and the set of
chunk identities written so far.

`stage_expected(source, expected, limits)` additionally refuses when the
complete source does not hash to `expected`, naming both identities.

`DurableStagedBlob::commit(self)` re-admits the current catalog, refuses
with `CatalogMoved` if it is not the generation the staging was verified
against, reads the sealed stage back, admits it as a segment, binds it to
the closed stage's record count, length, and digest, encodes the successor
catalog naming every current segment and the new one, and runs
`publish_catalog_generation`: the same twenty-six version-one crash
boundaries compaction and the fixture publications cross. When the catalog
already held everything, commit publishes nothing and the receipt says so
(`segment() == None`, the current generation).

`DurableIngestionReceipt` binds the storage profile, the blob and layout
identities, the published segment digest, the catalog generation and
digest, and `IngestionAccounting`: logical bytes, physical new bytes,
physical reused bytes, chunks new, chunks reused. Physical new plus
physical reused equals logical.

Content is readable through `DurableSnapshot::reconstruct_layout` by its
committed layout at once. `contains_blob` and the by-identity reads answer
once a retention root anchors the blob; the writer publishes content, and
retention is a separate, explicit act.

## Refusals and residue

Every refusal is typed at its boundary (`DurableIngestionError`). A refusal
before the first new chunk leaves nothing. A refusal after it leaves a
`staging/current.seg`, truncated when the source or a limit failed
mid-stream and complete when the identity mismatched after sealing; a
later `stage` refuses with `StageRetained` until the store is recovered.
`recover_durable_ingestion(root, policy)` runs the version-one recovery
protocol over the residue with ingestion's evidence for a complete stage:
it is discarded when the current catalog names none of its records, which
is exactly an ingestion stage that never reached commit. A complete stage
the catalog partly names is neither ingestion nor compaction residue and
refuses. `staging/current.cat` and `head.next` resolve as after an
interrupted compaction, because commit runs the same protocol.

The segment ceilings (1,048,576 records, 1 GiB) refuse with the stage's own
typed `RecordCountLimit` and `SegmentLengthLimit` inside `Stage`; a source
that would cross them is refused, not rolled over into a second segment.

## Evidence

`src/adapters/durable/writer_tests.rs`: one pass commits a blob readable by
layout, then by anchor beside the fixture store; nearby content reuses
every unchanged chunk and an exact re-ingest publishes nothing; byte-limit,
entry-limit, and expected-identity refusals leave nothing visible; an
interrupted source leaves a stage that recovery discards, and staging
refuses until it does. The runs are in-crate because a version-two store
is built today only through test-only unchecked admission.

## Still owed

- Segment rollover at the ceilings mid-blob: refused today, not rolled.
- Commit re-reads the sealed stage and re-admits every current segment
  from the loaded catalog, so its peak memory is segment-proportional;
  streaming segment admission is owed.
- The crash matrix driven by ingestion rather than by fixtures, the soak to
  the catalog entry ceiling, the multi-GiB stress, and the throughput,
  memory, allocation, sync-count, and dedup-ratio benchmarks.
- The Worldline golden run through the port, and the `keep put` and MCP
  `keep.ingest` adapters (F-42).
