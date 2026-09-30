# Transfer Pipeline

The transfer pipeline moves authenticated bytes out of any
[content-store port](README.md) view in bounded memory, and moves a blob
between two stores without buffering it. It is the adapter the read cores
already implied: they emit each verified chunk as one borrowed slice, and
the pipeline turns that slice into a segment the sink applies exactly once.

## Read-to-write: `transfer_*`

`transfer_layout`, `transfer_blob`, `transfer_range`, and
`transfer_layout_range` take a `ContentReads` view, a `TransferSink`, and
`TransferBounds` (a `TransferWindow` and a `CancellationSignal`). The view
runs its own read core, whose laws are unchanged: every chunk is
authenticated before its first byte is emitted, an exact layout never
substitutes another, a range emits exactly the requested bytes. The
pipeline's writer receives each emitted slice and hands it to the sink as
a `TransferSegment` (index, logical offset, borrowed bytes) without a
copy. Every `window` segments the sink is asked to `acknowledge`; after
the last segment it is asked to `complete`. The receipt carries the view's
own read receipt plus the segments, bytes, and acknowledgements the sink
took.

The signal is consulted before every segment. Cancellation stops the
transfer with `TransferError::Cancelled { segments, bytes }`: what the
sink already applied is stated, and nothing about it is called success. A
sink refusal is `TransferError::Sink`; a view refusal is
`TransferError::Read`. No variant claims the sink is empty.

`WriteSink<W: Write>` is the sink over any writer. It applies segments
exactly once in index and offset order, refusing a repeat or a gap with
`WriteSinkError::OutOfOrder`, and flushes on every acknowledgement. A
caller with a stricter durability standard implements `TransferSink`
itself.

## Copy-to-write: `copy_layout`

`copy_layout(source, layout_id, destination, limits)` copies one
committed layout from a `TransferSource` into a `ContentStaging`
destination. The source streams the layout's chunks through a pull reader
that looks each chunk up by identity in the view's own immutable bytes and
authenticates it as it is first served; a chunk that does not hash to its
identity stops the stream at that chunk with
`TransferSourceError::ChunkIdentityMismatch`. The destination stages the
stream with `stage_expected`, so it recomputes the complete identity and
refuses a mismatch before anything becomes visible, then commits. The
blob is never held whole: the source lends one chunk at a time and the
destination holds its own staging scratch.

`ReferenceStore` and `DurableSnapshot` implement `TransferSource`;
`ReferenceStore` and `DurableWriter` implement `ContentStaging`; every
pairing copies.

## Evidence

- `src/adapters/pipeline/tests.rs`: every verified slice reaches the sink
  in order; ranges transfer exactly; a window of one acknowledges every
  segment; a sink that fails mid-window and a cancellation each yield no
  receipt while the partial prefix is exactly what the sink applied; a
  write sink refuses out-of-order and repeated segments; copy between
  reference stores round-trips; an absent layout and a staging refusal
  commit nothing.
- `src/adapters/durable/transfer_tests.rs`: a durable snapshot copies
  into a reference store and transfers to a sink; a reference store copies
  into a durable writer.
- `src/reference/chunk_reader.rs`: the pull reader serves every chunk
  across small reads and refuses at the boundary of a chunk that does not
  hash to its identity.
- `tests/transfer_pipeline_memory.rs`, against the library as shipped:
  read-to-write allocates nothing beyond the sink for a 1 MiB blob
  (`AllocationInfo::default()`), and copy-to-write allocates fewer total
  and fewer peak bytes than a caller-owned copy loop that reconstructs
  into a buffer and stages the buffer.
- `benches/transfer_pipeline.rs` (`cargo bench --bench transfer_pipeline`)
  times read-to-write and copy-to-write through the pipeline against the
  caller-owned copy loop at 1 MiB and 4 MiB. On the authoring machine
  (2026-09-30, divan, 100 samples) read-to-write medians were 5.96 ms
  against 6.54 ms at 1 MiB and 26.33 ms against 26.66 ms at 4 MiB;
  copy-to-write medians were 7.74 ms against 7.90 ms at 1 MiB and
  32.38 ms against 31.76 ms at 4 MiB. The allocation advantage is
  established by the memory test; the CPU advantage is within the run's
  noise and is not claimed.

## Still owed

- A CPU advantage over the caller-owned copy loop: the benchmark exists
  and its numbers above are within noise, so the acceptance criterion's
  "lower CPU" is not established, only "no worse" and "less allocation".
- A profile mismatch between stores cannot be exercised while one storage
  profile is registered; the destination's own chunking decides its
  layout, so the copy receipt's layout may differ from the source's.
- The Worldline copy through the pipeline is owed with the durable
  Worldline run (T-24.2).
- Multi-threaded pipelines stay out of scope until a bounded-memory proof
  exists.
