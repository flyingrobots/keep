# Immutable-segment protocol and verified I/O audit

This page owns T-11.1 through T-11.3 from the originally checked roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 527–533. Inspected main is
`f49cff732cf7a6e1b472decba9e4c4130990559e`.

## T-11.1 — Specify bytes, crash states, and recovery together

**Named acceptance and delivery: met on main.** Accepted ADR-0005 and the
segment-store format directory describe framing, canonical checksums,
publication ordering, incomplete-stage states, and recovery decisions as one
protocol. The specification separates logical identity from physical segment
layout. This verdict concerns the specification; it does not prove every
implementation transition conforms to it.

## T-11.2 — Implement codecs, writer, and reader

**Acceptance/definition of done: not met.** The header, record, seal, complete
reader, golden fixtures, corruption refusals, and bounded-memory laws exist.
The segment-format fuzz target exercises five decoding boundaries, but no new
fuzz campaign is claimed by this audit.

KEEP-SEGMENT-005 requires a public sealed receipt to expose no mutable stage
handle. With `repository-tasks` enabled, public `SealedSegment::map_stage`
gives an arbitrary caller closure the owned writable stage and preserves the
original receipt metadata. A temporary public API regression sealed an empty
segment, wrote one additional byte through that closure, and observed 193
bytes while the returned sealed receipt still reported 192. The exact length
assertion failed. The probe was removed after recording the result.

This demonstrates writable capability escape and stale sealed evidence. It
does not establish that the filesystem publisher admits corrupted bytes:
publication has separate verification and authority checks. The production
crash harness uses this mapper to remove its storage decorator, so the fix
must preserve that harness without keeping arbitrary post-seal mutation.

Correction owner: [issue #146](https://github.com/flyingrobots/keep/issues/146).
The correction must land capability restrictions, harness adaptation, and
regression evidence together without changing format or content identity.

## T-11.3 — Deterministic fault injection at every write phase

**Behavioral evidence: met; named evidence path needs correction.** The
scripted writer suite covers header, record-header, payload, checksum, seal,
short/interrupted/zero/overreported writes, and both flush/synchronization
boundaries. Failures preserve the exact phase, source, and accepted prefix;
failed durability produces no sealed receipt.

The roadmap's `tests/segment_filesystem_stage.rs` does not exist. The actual
four filesystem laws live in
`src/adapters/filesystem_segment_stage_tests.rs`, covering exclusive creation,
one owner, exact sealed bytes, and retained unsealed prefixes. This is an
outdated evidence reference, not evidence that those laws are missing.
At this inspected snapshot, documentation correction was assigned to #69.
PR #136 subsequently corrected the living v1 ledger and closed #69, but did
not add the originally named integration target. The literal artifact
requirement remains undelivered and is retained by the #131 audit until an
executable correction owns it. Existing unit laws do not substitute for the
named public API integration artifact.

## Executed evidence and limits

Copy-isolated Linux Docker with pinned Rust 1.96.0 and its source-specific
build directory ran nine public segment integration targets: 67 laws passed
in debug and 67 in release. Four actual filesystem unit laws passed in each
build mode. The failing mapper probe ran with `repository-tasks` enabled and
was restored afterward. These runs establish observed behavior under the
specified checks, not absence of all crash or corruption failures.

Thirty-three remaining checked tasks now have recorded verdicts. Thirty-one
other checked tasks and nineteen reopened entries still need full accounting.
No original checkbox changed, and no correction is declared integrated merely
because a PR exists.
