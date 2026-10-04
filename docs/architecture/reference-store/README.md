# Non-Durable Reference CAS

- Status: Implemented public reference adapter
- Related issues:
  [#11](https://github.com/flyingrobots/keep/issues/11) and
  [#13](https://github.com/flyingrobots/keep/issues/13)
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

Exact range reads use the same deterministic layout choice. A successful range
receipt proves only the exact claim described under
[Exact byte-range reads](#exact-byte-range-reads).

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

The returned `StagedBlob` deliberately owns every chunk value absent from the
store used during staging. Those bytes may grow with blob length up to
`ReferenceStoreCapacity`. The API and type documentation expose that
materialization; input beyond the configured capacity refuses.

## Staging memory contract

`ReferenceStore::STAGING_SCRATCH_LIMIT_BYTES` names the fixed read/chunk
buffers and stream state. It excludes staged payloads, map/layout metadata,
caller input, and allocator overhead; it is not a process-RSS bound.

Committed payload bytes plus pending unique payload bytes cannot exceed
`ReferenceStoreCapacity`. Capacity is checked before copying each new chunk;
an ordinary capacity refusal reports an attempted total at most one maximum
chunk beyond capacity. Arithmetic overflow is a separate refusal path.
Metadata remains bounded by `LayoutEntryLimit`, rather than by logical bytes.

The allocation laws measure incremental live heap during `stage`, excluding
caller input and previously committed data. They cover over-capacity refusal
with no retained heap, source failure after a staged chunk with preserved
committed content, already-committed deduplication, and a synthetic stream
sixteen times capacity whose repeated content stages one unique chunk. A
synthetic 4 GiB source refuses on its first oversized chunk after consuming
256 KiB, with bounded heap and no caller-side source allocation. The
fixture's 1 KiB allowance per possible entry accounts for map/layout metadata
in these measurements; it is an empirical test allowance, not a format limit
or a universal allocator theorem.

Fully deduplicated staging has zero pending payload bytes, but still allocates
the bounded chunk buffer and layout metadata. This adapter materializes every
new unique chunk until commit; it does not accept arbitrary unique content at
constant memory. The [rationale](rationale.md#why-staging-materializes-up-to-capacity)
records why spilling or publishing prefixes requires a different protocol.

## Publication

Staged work is invisible and `#[must_use]`. `StagedBlob::commit` is the only
ordinary transition into visible reference-store state. It revalidates
capacity, required chunk availability, chunk conflicts, layout conflicts,
committed chunks, and layout indexes before changing the store. A staged value
may be committed to another store only when its owned chunks plus that
destination's existing chunks completely satisfy the layout.

Commit is atomic only with respect to synchronous exclusive `&mut
ReferenceStore` access. It may allocate in-memory map nodes. It performs no
filesystem I/O, flush, synchronization, journal write, or durable publication.
Process death may erase every pre-commit and post-commit state.

Missing or inconsistent committed state is a refusal. Ordinary publication
never repairs a missing committed chunk, layout, or index. A future durable
backend must define a separate explicit recovery protocol.

## Reconstruction

Whole-blob reconstruction hashes each immutable in-memory chunk exactly once,
then emits the verified chunks by identity. Before output it:

1. verifies every stored chunk against its named `ChunkId`;
2. replays `fastcdc-64k-v1` and compares every boundary with the layout; and
3. verifies the complete byte sequence against the target `BlobId`.

Only after all three checks succeed does it emit each verified chunk. Short
writes are completed, interruptions are retried, and broken writer counts are
typed refusals. The committed-layout path allocates no adapter-owned heap
memory; any allocation by the supplied writer belongs to that writer.

Reconstruction does not flush the writer and makes no durability claim about
the output.

## Exact byte-range reads

`ByteRange` is a validated half-open `[offset, end)` coordinate whose checked
end cannot wrap. It does not know a target length, so any non-wrapping range,
including an empty range at `u64::MAX`, may be constructed. Only
`AdmittedLayout::plan_range` proves that a range is within the target length:
planning permits an empty range at or before the target end and requires a
nonempty range to end at or before the target length.

Planning traverses admitted layout metadata without allocation and selects the
minimal ordered entry interval that overlaps the request. Committed-layout
reads load no prefix or suffix chunk outside that interval and allocate no
adapter-owned heap memory. Caller-supplied admitted layouts and encoded layout
records may allocate bounded layout-record or decoded-entry metadata to
calculate a canonical identity. That identity must name a committed layout;
planning, receipt coordinates, and chunk lookup use only the committed layout.
None of the range APIs materializes the complete blob.

Before any output, a range read authenticates every selected complete chunk
against its `ChunkId`. During the output pass it fetches each verified chunk,
slices only the overlap, completes short writes, retries interruptions, and
uses checked output accounting. Invalid layouts, out-of-bounds coordinates,
missing or mismatched selected chunks, broken writers, and output failures are
typed refusals.

The resulting `RangeReadReceipt` proves that the requested bytes came from
authenticated complete chunks under the admitted layout. It does not prove the
complete `BlobId`, any unrequested chunk, or any storage-profile boundary.
Applications that require those stronger claims must use whole-blob
reconstruction.

Range reads are synchronous and do not flush the writer. The in-memory chunks
remain non-durable before, during, and after the operation.

## Evidence

- `tests/streaming_cas/ingestion_laws.rs` covers staging, deduplication,
  capacity (exactly at, and one byte over), short reads, interruptions, and
  streaming entry-cap refusal.
- `tests/streaming_cas/reconstruction_laws.rs` covers exact authenticated
  output, full-blob mismatch, and missing chunks.
- `tests/streaming_cas/refusal_laws.rs` covers malformed records, frozen false
  profile boundaries, and broken writers.
- `tests/streaming_cas/model_laws.rs` compares all 216 exhaustive three-step
  operation sequences with a boring reference model.
- `tests/range_plan.rs` covers checked coordinates and minimal ordered overlap.
- `tests/range_read.rs` covers public range boundaries, short writes, and
  malformed-layout refusal.
- `tests/range_read_properties.rs` compares exhaustive short and generated
  multi-chunk ranges with direct reference slicing.
- `src/reference/range_read_tests.rs` proves prefix and suffix chunks are not
  loaded and selected-chunk corruption refuses before output.
- `tests/streaming_cas_memory.rs` proves committed-layout reconstruction and
  range reads allocate no adapter-owned heap memory, that staging a source
  five times the capacity refuses under the memory ceiling and retains
  nothing, and that fully deduplicated staging stays at the scratch floor.
- `tests/golden_file_worldline/storage_assertions.rs` executes the Golden File
  Worldline through the public API.

The [rationale](rationale.md) records why this adapter materializes bytes,
requires explicit commit, verifies before output, and refuses to imply
durability.
