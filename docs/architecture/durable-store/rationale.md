# Durable admission rationale

## Decision

Before materializing a sealed ingestion stage, commit requires its current
regular-file length to equal the length recorded by `ClosedSegment`. The
exact-record boundary reads only that admitted length and refuses trailing
bytes. Segment admission and selection still verify the checksum, content,
record count, digest, and the authority that sealed the stage.

An external writer can grow or replace a stage despite Keep's advisory writer
lock. The stage's current metadata cannot authorize additional allocation.
The trusted bound is the sealed work's length, derived by checked arithmetic
during ingestion. The stage remains recovery evidence after any refusal.

## Alternatives rejected

`read_to_end` on the current file allocates according to adversarial state,
before the segment decoder can refuse it. A bound inferred from current
metadata has the same problem. Streaming segment admission remains a separate
unfinished ingestion requirement; exact-length materialization fixes this
allocation boundary while preserving the current admission protocol.

## Errors and evidence

Typed failures remain available through `Error::source` and an `io::Error`'s
`get_ref`, without converting nested failures to strings. The laws in
`src/adapters/durable/refusal_source_tests.rs` preserve a selected root's
decoder and exact-record refusal. `ingestion_bound_tests.rs` grows a sealed
stage and asserts refusal before segment admission, an unchanged catalog head,
and retained stage evidence. There is no format or identity change, new sync,
or change to crash recovery.
