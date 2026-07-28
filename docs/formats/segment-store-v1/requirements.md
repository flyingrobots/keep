# Requirements, Compatibility, and Evidence

This page owns the requirement ledger, compatibility boundary, security
nonclaims, and golden evidence for `keep.segment-store/v1`.

## Requirement ledger

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Design evidence | Status |
| --- | --- | --- | --- |
| `KEEP-STORE-001` | Segment, record, seal, catalog, entry, and head have exact versioned canonical grammars | Field tables and golden artifacts | Specified in #14 |
| `KEEP-STORE-002` | Every persistent integer is fixed-width big-endian and checked | Canonical primitives and bounds | Specified in #14 |
| `KEEP-STORE-003` | Checksums and physical digests use named domain-separated preimages | Checksum formulas and fixture oracle | Specified in #14 |
| `KEEP-STORE-004` | Sealed segments and published catalogs are immutable | State model and publication protocol | Specified in #14 |
| `KEEP-STORE-005` | Logical identity is separate from physical location | Record and catalog entry grammars | Specified in #14 |
| `KEEP-STORE-006` | Catalog ordering and duplicate refusal are deterministic | Catalog-entry rules | Specified in #14 |
| `KEEP-STORE-007` | Publication has explicit flush, sync, no-clobber link, unlink, head-replacement, and directory-sync order | `KEEP-CRASH-001`–`026` | Specified in #14 |
| `KEEP-STORE-008` | File presence alone proves no state | Core law and visibility table | Specified in #14 |
| `KEEP-STORE-009` | Writer exclusion uses one persistent kernel-managed lock | Writer-exclusion contract | Specified in #14 |
| `KEEP-STORE-010` | Readers observe one complete immutable generation | Reader-snapshot protocol | Specified in #14 |
| `KEEP-STORE-011` | Opening is observational and recovery is explicit | Recovery protocol | Specified in #14 |
| `KEEP-STORE-012` | Required recovery classes remain distinct | Recovery classification ledger | Specified in #14 |
| `KEEP-STORE-013` | Ambiguous or corrupt durable state is refused | Recovery and refusal order | Specified in #14 |
| `KEEP-STORE-014` | Memory, record, segment, and catalog sizes are bounded | Bounds table | Specified in #14 |
| `KEEP-STORE-015` | Unsupported filesystem semantics fail closed | Platform contract | Specified in #14 |
| `KEEP-STORE-016` | No Echo, Graft, Git, or application policy enters the protocol | ADR-0005 and physical namespace | Specified in #14 |
| `KEEP-STORE-017` | Catalog locations equal verified top-level segment-record spans | Catalog-entry admission | Specified in #14 |
| `KEEP-STORE-018` | Truncated-stage discard has explicit unlink and directory-sync crash boundaries | `KEEP-CRASH-027`–`028` | Specified in #14 |
| `KEEP-STORE-019` | Leftover next-head recovery either finalizes an exact successor or discards explicit evidence durably | `KEEP-CRASH-025`–`026`, `029`–`030` | Specified in #14 |
| `KEEP-STORE-020` | Initialization is writer-locked, idempotent, recoverable from every partial canonical namespace set, and admitted only after root synchronization | `KEEP-CRASH-031`–`035` | Specified in #14 |

<!-- markdownlint-enable MD013 -->

## Compatibility and migration

The byte grammars, magic values, field widths and order, endianness, kinds,
flags, algorithm coordinates, bounds, checksum domains, catalog ordering,
physical-name grammar, generation law, crash-point identifiers, publication
order, and recovery classifications are compatibility commitments.

Changing any of them requires a new store protocol version and an explicit
migration decision. A migration writes and verifies new immutable artifacts,
publishes a new compatible head under its accepted protocol, and never
reinterprets existing bytes in place.

Logical `BlobId`, `ChunkId`, and `LayoutId` remain stable when their exact
logical bytes and canonical plans remain unchanged.

## Security and privacy

Version 1 provides integrity checks, not writer authentication or
confidentiality. Logical identities, lengths, record boundaries, catalog
membership, and physical reuse are visible metadata. An attacker controlling
all bytes can recompute unkeyed checksums.

All path operations are capability-relative and no-follow. Untrusted counts
and lengths are bounded before allocation. Diagnostics retain expected and
observed typed coordinates but do not include plaintext payloads, unbounded
paths, secrets, or terminal control characters.

## Golden evidence

The
[durable segment-store corpus](../../../conformance/segment-store/v1/README.md)
contains:

- an empty sealed segment;
- a one-byte chunk segment bound to an existing independent `ChunkId`;
- catalog generation 1 naming its exact physical record;
- a publication head naming that exact catalog;
- a two-record segment carrying the chunk and its canonical flat layout;
- a two-entry catalog proving chunk-before-layout order and checked offsets;
- a publication head naming that cross-kind catalog;
- exact canonical bytes, checksums, physical digests, lengths, counts, and
  offsets; and
- the complete `KEEP-CRASH-001`–`KEEP-CRASH-035` transition ledger.

The test-only Rust oracle reconstructs every artifact directly from these
tables and formulas. Production implementations must match the frozen corpus
and add parser fuzzing, corruption mutations, crash injection, recovery tests,
and model-based generation tests in issues #15–#17.

The format-local tradeoffs are recorded in the
[colocated rationale](rationale.md).
