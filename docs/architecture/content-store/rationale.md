# Content-store port rationale

## Decision

`TransferSource`, `StreamConsumer`, and `TransferSourceError` belong to the
content-store port in `src/store/transfer_source.rs`. Reference and durable
adapters implement the capability; pipeline adapters consume it. Sealing
implementations stay with the adapters, so the port imports neither backend.

The transfer port expresses a real substitution boundary: a consumer pulls
authenticated bytes from either reference memory or a fenced durable view.
Its semantic identities, borrowed reader, and typed refusals contain no
physical location or serializer value. Public crate-root names stay unchanged.

## Alternatives rejected

Keeping the capability in `adapters/pipeline` made reference chunk reads import
an adapter-owned error. That reversed the dependency direction required by
[ADR-0004](../../adr/0004-hexagonal-boundary-architecture.md). Duplicating the
capability or its failures would create two contracts for the same operation.

## Evidence and compatibility

`tests/transfer_port_architecture.rs` rejects adapter imports in the reference
pull reader and source. Existing transfer, copy, corruption, cancellation, and
memory laws exercise both implementations. This ownership correction changes
no content identity, public method, durable format, write order, or recovery
protocol and adds no allocation or I/O.
