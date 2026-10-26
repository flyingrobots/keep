# Content-store port rationale

## Decision

`TransferSource`, `StreamConsumer`, and `TransferSourceError` belong to the
content-store port in `src/store/transfer_source.rs`. The reference store and
the durable snapshot implement the capability; the pipeline in
`src/pipeline` consumes it. Sealing implementations stay with each backend,
so the port imports neither backend.

The pull reader that authenticates each chunk as it is served,
`ChunkReader`, lives in the shared read core (`src/authenticated_read`) next
to `verified_chunk`, because both backends serve chunks through the same
`ChunkSource` lookup and must authenticate them the same way.

The pipeline itself is not an adapter: it owns no technology, imports only
ports (`ContentReads`, `ContentStaging`, `TransferSource`), and coordinates a
caller-supplied `TransferSink`. It therefore lives at `src/pipeline` rather
than under `src/adapters`.

The transfer port expresses a real substitution boundary: a consumer pulls
authenticated bytes from either reference memory or a fenced durable view.
Its semantic identities, borrowed reader, and typed refusals contain no
physical location or serializer value.

## Alternatives rejected

- Keeping the capability in `adapters/pipeline`, as the closed #107 branch
  first did, made reference chunk reads import an adapter-owned error and
  reversed the dependency direction required by
  [ADR-0004](../../adr/0004-hexagonal-boundary-architecture.md). The branch
  corrected this in `652059e` (#127); this salvage starts from the corrected
  shape.
- Keeping `ChunkReader` in `src/reference`, as the branch did, would make the
  durable adapter import the reference adapter for a shared read core.
- Duplicating the capability or its failures per backend would create two
  contracts for the same operation.
- The branch's `tests/transfer_port_architecture.rs` asserted the absence of
  the string `crate::adapters` in two source files. It is not carried over: a
  source-text check cannot observe the dependency direction it names (an
  aliased or re-exported import passes it), which is the evidence gap #168
  records for a similar assertion. Rust module privacy does not enforce the
  direction either, because crate-private adapter items remain nameable
  anywhere in the crate. Inward dependency of `src/store`,
  `src/authenticated_read`, `src/reference`, and `src/pipeline` is therefore
  a review obligation until the repository gains a real dependency-direction
  check.

## Evidence and compatibility

Transfer, copy, corruption, cancellation, and memory laws exercise both
sources. This is an additive API: it changes no content identity, durable
format, write order, or recovery protocol.
