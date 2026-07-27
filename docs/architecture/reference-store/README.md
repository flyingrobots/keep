# Non-Durable Reference CAS

- Status: Implemented public reference adapter
- Related issue: [#13](https://github.com/flyingrobots/keep/issues/13)
- Storage profile: `fastcdc-64k-v1`
- Layout format: `keep.flat-chunks/v1`
- Durability: None

The reference CAS is executable evidence for Keep's streaming
content-addressed storage laws. It is intentionally an in-memory adapter, not
the durable segment backend planned for M3.

## Contract

For a requested `BlobId`, reconstruction returns exactly the authenticated
bytes named by that identity or a typed refusal. Presence in this adapter means
only that at least one committed layout currently names the blob in process
memory. It does not establish retention, restart recovery, crash durability, or
power-loss durability.

`ReferenceStore` uses deterministic ordered maps and sets. When multiple
layouts name one blob, reconstruction chooses the lowest canonical `LayoutId`.
Chunk deduplication is keyed by `ChunkId`; it is a storage fact, not a retention
claim.

## Ingestion

`ReferenceStore::stage` is one blocking streaming flow:

1. read through a fixed 8 KiB input buffer;
2. update the complete `BlobId`;
3. feed the registered FastCDC detector;
4. verify and stage each emitted chunk;
5. enforce the caller's `LayoutEntryLimit` as boundaries arrive;
6. admit the semantic layout; and
7. calculate its canonical `LayoutId`.

The streaming scratch state is bounded by the input buffer, the registered
maximum chunk length, hash and detector state, and the caller's layout entry
cap. Layout admission and `LayoutId` calculation transiently materialize
metadata proportional to the bounded entry count.

The returned `StagedBlob` deliberately owns every new unique chunk value. Those
bytes may grow with blob length up to `ReferenceStoreCapacity`. The API and
type documentation expose that materialization; input beyond the configured
capacity refuses.

## Publication

Staged work is invisible and `#[must_use]`. `StagedBlob::commit` is the only
ordinary transition into visible reference-store state. It revalidates
capacity, chunk conflicts, layout conflicts, committed chunks, and layout
indexes before changing the store.

Commit is atomic only with respect to synchronous exclusive `&mut
ReferenceStore` access. It may allocate in-memory map nodes. It performs no
filesystem I/O, flush, synchronization, journal write, or durable publication.
Process death may erase every pre-commit and post-commit state.

Missing or inconsistent committed state is a refusal. Ordinary publication
never repairs a missing committed chunk, layout, or index. A future durable
backend must define a separate explicit recovery protocol.

## Reconstruction

Whole-blob reconstruction performs two passes over immutable in-memory chunks.
Before output it:

1. verifies every stored chunk against its named `ChunkId`;
2. replays `fastcdc-64k-v1` and compares every boundary with the layout; and
3. verifies the complete byte sequence against the target `BlobId`.

Only after all three checks succeed does it reverify and emit each chunk. Short
writes are completed, interruptions are retried, and broken writer counts are
typed refusals. The committed-layout path allocates no adapter-owned heap
memory; any allocation by the supplied writer belongs to that writer.

Reconstruction does not flush the writer and makes no durability claim about
the output.

## Evidence

- `tests/streaming_cas/ingestion_laws.rs` covers staging, deduplication,
  capacity, short reads, interruptions, and streaming entry-cap refusal.
- `tests/streaming_cas/reconstruction_laws.rs` covers exact authenticated
  output, full-blob mismatch, and missing chunks.
- `tests/streaming_cas/refusal_laws.rs` covers malformed records, frozen false
  profile boundaries, and broken writers.
- `tests/streaming_cas/model_laws.rs` compares all 216 exhaustive three-step
  operation sequences with a boring reference model.
- `tests/streaming_cas_memory.rs` proves the committed-layout reconstruction
  path allocates no adapter-owned heap memory.
- `tests/golden_file_worldline/storage_assertions.rs` executes the Golden File
  Worldline through the public API.

The [rationale](rationale.md) records why this adapter materializes bytes,
requires explicit commit, verifies before output, and refuses to imply
durability.
