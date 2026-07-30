# Closure Verification

- Status: Normative version-2 protocol; storage-independent verifier
  implemented; publication integration planned in issue
  [#19](https://github.com/flyingrobots/keep/issues/19)
- Format coordinate: `keep.segment-store/v2`
- Requirement: [`KEEP-RETENTION-005`](requirements.md#retention-transitions)
- Decision record:
  [ADR-0009](../../adr/0009-retention-roots-release-and-gc-liveness.md)

This page defines deterministic closure traversal, exact resource accounting,
authenticated reconstruction, and canonical closure evidence. The
[retention record specification](retention.md) owns the limits stored in each
root generation. The [format rationale](rationale.md) explains why evidence
cardinality and reconstruction work use separate counters.

## Verification boundary

Closure verification receives:

- one admitted root generation with a canonical anchor sequence;
- its exact registered retention-realization profile;
- one pinned, completely verified catalog generation; and
- the root's admitted closure limits.

Profile and limit admission completes before traversal. The verifier does not
read paths, enumerate a filesystem, consult a clock, invoke a caller callback,
or replace a missing witness. Version 2 selects the single record bound to each
logical identity by the pinned catalog.

The catalog has already admitted each bound segment record's framing, checksum,
logical identity, and payload. Closure verification consumes those proofs,
decodes layouts again under the closure budget, and authenticates each complete
logical blob.

## Deterministic traversal

Anchors are visited in their canonical `BlobId`, then `LayoutId`, order. For
each anchor:

1. Schedule the anchor's layout at depth `1`.
2. Resolve the exact layout record and charge its resource units.
3. Decode the canonical layout with the named `LayoutId` as an independent
   expectation.
4. Require the layout target to equal the anchor's `BlobId`.
5. Visit layout entries in logical-offset order.
6. Schedule each entry's chunk at depth `2`, resolve its exact record, and
   charge its resource units.
7. Reconstruct entries in layout order, replay the exact registered storage
   profile, and authenticate the complete `BlobId`.

Version-2 flat layouts cannot exceed depth `2`. The stored depth limit may be
larger so a successor layout grammar can be represented without weakening the
format ceiling. Version 2 refuses an unknown mandatory edge instead of
interpreting it as a deeper known node.

The visited set is keyed by `SegmentRecordIdentity`. A node is inserted when
its identity is first scheduled, before catalog lookup. An anchor is not a
closure node because the root format bounds anchors separately. Each unique
`SegmentRecordIdentity` contributes one node even when layouts or chunks are
shared. Missing members still consume their scheduled node and depth budget
before the typed missing-member refusal.

## Exact resource accounting

Every counter starts at zero. Every increase uses checked addition before the
corresponding lookup, decode, record consumption, or reconstruction step. An
arithmetic overflow is a typed refusal, not an implied limit breach.

### Nodes

The node count is the number of unique catalog record identities first
scheduled across the complete root. It includes layout and chunk identities.
It excludes anchors, catalog entries not reached by an anchor, and a repeated
logical occurrence of an already visited identity.

### Depth

Depth is the number of catalog record identities on the active edge path.
The layout is depth `1`; one of its chunks is depth `2`. The verifier checks
the candidate depth before scheduling the identity. The observed depth in
successful evidence is the maximum reached across the complete root, or zero
for an empty anchor set.

### Encoded bytes

Encoded bytes count structured closure metadata decoded by the verifier.
Version 2 charges the canonical layout payload length once for each unique
layout identity, before decoding that payload. Chunk payloads, segment framing,
root bytes, manifest bytes, catalog bytes, and profile-definition bytes do not
contribute to this counter.

### Physical bytes

Physical bytes bound record-backed reconstruction work rather than unique
storage footprint. The verifier charges:

- the complete segment-record length once when each layout is consumed; and
- the complete segment-record length for every chunk occurrence consumed in
  layout order.

A repeated logical occurrence therefore consumes physical bytes again even
though it does not add a node or another canonical closure-member entry. This
rule bounds replay and blob-authentication work for layouts that repeat one
small chunk many times. Shared physical evidence is not a license for
unbounded logical reconstruction.

## Fail-closed order

For one first-scheduled identity, checks occur in this order:

1. admit the candidate depth;
2. checked-add and admit the node count;
3. resolve the identity from the pinned catalog;
4. checked-add and admit the complete segment-record length;
5. for a layout, checked-add and admit its canonical layout payload length;
6. consume the already admitted record proof; and
7. decode or reconstruct its semantic content.

An already visited chunk skips steps 2, 3, 5, and canonical-member insertion,
but each logical occurrence repeats the physical-byte check in step 4 before
its bytes enter profile replay and blob authentication.

The first failed check in deterministic traversal order is returned. Missing,
wrong-kind, unsupported-profile, limit, overflow, layout, anchor-target,
chunk, profile-boundary, and final-blob failures remain distinct typed errors
with expected and observed state where applicable. No failure yields partial
closure evidence.

## Canonical closure digest

Successful verification emits 96-byte closure-member entries, one for each
unique record identity:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 1 | record kind | `1` for chunk; `2` for layout |
| 1 | 3 | reserved | zero |
| 4 | 60 | identity slot | canonical encoding below |
| 64 | 32 | record checksum | exact admitted segment-record checksum |

The chunk identity slot is its four-byte big-endian length, then its 32-byte
digest, then 24 zero bytes. The layout identity slot is the exact 60-byte
canonical binary `LayoutId`. Entries use canonical typed-identity order:
chunks by identity slot, followed by layouts by identity slot. There are no
duplicate entries.

The closure digest is:

```text
BLAKE3-256(
    "keep.retention-closure/v2\0" ||
    profile-identity-u32 ||
    profile-version-u32 ||
    profile-definition-digest ||
    catalog-generation-u64 ||
    catalog-digest ||
    node-count-u64 ||
    maximum-depth-u16 ||
    six-zero-reserved-bytes ||
    encoded-bytes-u64 ||
    physical-bytes-u64 ||
    canonical-closure-member-entries
)
```

All integers are unsigned big-endian. `node-count-u64` is also the entry count.
The digest binds the exact profile, catalog, observed resource use, logical
member set, and record checksums. The transition receipt binds it beside the
root's separate anchor-set digest; neither digest substitutes for the other.

## Executable evidence

- The [one-anchor law](../../../tests/retention_closure.rs) freezes exact
  counters, the member transcript, the digest, and reconstruction.
- The [repeated-chunk
  law](../../../tests/retention_closure/repeated_chunk_law.rs) separates
  reconstruction work from unique-node evidence.
- The [adversarial-catalog
  laws](../../../tests/retention_closure/adversarial_catalog_laws.rs) prove exact
  missing-member and target-mismatch precedence.
- The [limit-precedence
  laws](../../../tests/retention_closure/limit_precedence_laws.rs) prove the
  depth, node, physical-byte, and encoded-byte refusal order.
- The [closure model
  laws](../../../tests/retention_closure/closure_model_laws.rs) compare all
  `3 × 3 × 3 × 5 = 135` one-zero boundary policies with a boring model.

## Evidence and nonclaims

Successful evidence records the closure digest, all four observed counters,
the exact profile coordinate, and the exact catalog generation and digest.
It proves that every anchor reconstructed and authenticated at verification
time under those coordinates.

It does not prove application meaning, future reachability after another
generation commits, unique physical ownership, retained byte count on disk,
secure erasure, or a faster verification path than the accounted traversal.
