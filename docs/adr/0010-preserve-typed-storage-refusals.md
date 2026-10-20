# ADR-0010: Preserve Typed Storage Refusals

- Status: Accepted for implementation on this branch; integration pending.
- Date: 2026-10-01
- Owners: Keep maintainers
- Related issue: [#110](https://github.com/flyingrobots/keep/issues/110)

## Context

T-01.2 requires typed refusals with useful expected and observed evidence and
preserved sources. Later durable adapters weakened that foundation by
converting exact-record errors to strings, replacing failures with static
messages, or discarding a predecessor root's decoder error. An I/O port's
return type does not justify erasing its semantic payload.

## Decision

An adapter-owned semantic refusal implements `Error` and remains the payload
of its I/O wrapper. Closed protocol-state enums describe retention stages,
migration, and GC. Selected-root, reader-fence, filesystem-operation, and
compaction-recovery refusals retain expected and observed coordinates where
the boundary has them. These public refusal types support caller downcasts.

`ExactRecordError::into_io` preserves original operational errors unchanged
and wraps exact-record refusals as typed `InvalidData` payloads. Retention,
migration, GC, and stage cleanup reuse this conversion. They never convert
the refusal to a message or replace a failed absence check with another error.

Predecessor decode failures retain the exact root decoder source. Namespace
read failures retain their original OS source and operational error kind;
a missing committed namespace remains an evidenced `InvalidData` refusal.
Transfer's internal stop signal is typed while the writer retains the
original sink failure for the final transfer error.

An `io::Error` exposes its custom payload through `get_ref`; consumers should
inspect that payload as well as `Error::source` when traversing a causal chain.
Diagnostic strings are presentation, not a stable classification API.

## Alternatives considered

Keeping strings preserves familiar diagnostics but prevents callers from
distinguishing the cause without parsing text. Adding a generic message
wrapper would remain a string refusal with a new name. Replacing I/O ports
with protocol-specific return types would unnecessarily change all storage
ports and external implementations. Boundary enums retain typed evidence
without changing the port signatures.

## Consequences

The public refusal vocabulary gains additive types and current-state
variants. Existing public variant constructors remain available. Corrupt
predecessor bytes now report `PredecessorRootRefused` with the decoder cause;
`PredecessorRootChanged` still describes a decoded selection mismatch.
Callers must match typed errors, not rely on previous diagnostic wording.

Identity, canonical bytes, publication order, synchronization, durability,
and recovery decisions do not change. Error evidence contains coordinates
and bounds, never content bytes, keys, or unbounded physical paths. No new
dependency or performance optimization is introduced.

Runtime regressions cover corrupted chunks, substituted retention and
migration records, malformed GC residue, fence length, selected-root bounds,
valid-but-wrong root selections, and predecessor checksum failure. Existing
fault, corruption, recovery, and public API laws remain authoritative.
`tests/typed_refusal_source_contract.rs` guards explicit textual I/O error
constructors in production source; it is a targeted source contract rather
than a general proof of all possible error flows.
