# Content-store port

The content-store port is the backend-neutral contract code writes against
once and runs on any admitted backend. It is two halves, each a Rust trait
in the crate root, and a receipt law enforced by the type system.

## The read half: `ContentReads`

Every admitted view answers the reference store's read laws through one
trait: `contains_blob`, `reconstruct`, `reconstruct_layout`, `read_range`,
and `read_layout_range`. The laws are the reference store's, unchanged:

- every emitted byte is authenticated before it is written;
- an exact layout never substitutes another;
- a range read proves only the chunks the range overlaps and emits exactly
  the requested bytes;
- absence is evidence only against a complete view.

Implemented by the non-durable
[`ReferenceStore`](../reference-store/README.md) and by the fenced
`DurableSnapshot`. Each backend keeps its own receipt and error types as
associated types: the durable receipts carry the view coordinates, the
reference receipts do not, and neither converts into the other.

`contains_blob` is fallible because a durable view answers membership by
admitting retained roots, which can fail; the reference store names
`Infallible`. `Ok(false)` is evidence of absence only against a complete
view.

The durable implementation forwards to the snapshot's inherent reads. It
adds no verification, fence, or allocation, and the snapshot keeps its
collector fence for as long as the caller holds it.

## The write half: `ContentStaging` and `StagedContent`

`ContentStaging::stage(source, limits)` chunks, hashes, and holds an
unknown-length source without making anything visible;
`stage_expected(source, expected, limits)` additionally refuses when the
complete source does not hash to `expected`, naming both identities.
`StagingLimits` carries a `LayoutEntryLimit` and a `StagedByteLimit`. The
byte limit is checked before each read is hashed, chunked, or staged, and
the refusal (`IngestionError::ByteLimitExceeded { limit, accepted,
incoming }`) says how far the source was admitted and what pushed it over.
The refused read has already been copied into the fixed 8 KiB read buffer;
nothing past the limit enters staged work.

A staging borrows its store (`ContentStaging::Staged<'store>`), so the
store cannot change underneath it and `StagedContent::commit(self)` needs
no second reference: it makes the staging visible, or leaves it invisible,
and returns a `CommitReceipt` naming the target and the exact committed
layout.

Only the non-durable `ReferenceStore` implements the write half, through
`ReferenceStagedContent`, and it stays honest about being non-durable:
process death loses everything in it, and no port method claims otherwise.
No durable writer implements it yet; bounded durable ingestion is
tracked by [#82](https://github.com/flyingrobots/keep/issues/82).

## The receipt law

A non-durable receipt can never stand where a durable one is required. The
port does not name a common receipt type; each backend's receipt is its own
type, so the substitution is a compile error. `src/store/mod.rs` pins this
with a `compile_fail` doctest that hands a `ReconstructionReceipt` to a
function taking a `DurableReconstructionReceipt`.

## Evidence

- `src/store/port_laws.rs` holds the generic laws: exact reconstruction
  through both entry points with equal receipts, exact ranges plus a
  past-the-end refusal, and absence reported and refused with nothing
  written. Each backend supplies a typed refusal oracle, so the laws assert
  exact variants rather than any error.
- The reference backend runs them in `src/store/reference_port_tests.rs`;
  the durable backend runs them over the migrated one-zero corpus in
  `src/adapters/retention/durable_port_law_tests.rs` on admitted Linux ext4.
- `tests/content_store_port.rs` runs the reference backend from outside the
  crate through the port alone: round trip, byte-limit refusal before
  anything is visible, the exact entry-limit refusal, and expected-identity
  mismatch.

## Provenance

This port was first written on the closed PR #107 branch
(`feature/roadmap-m4-tasks`, commits `0d300db` and `ee4ef54`) and salvaged
onto main with two changes: `contains_blob` became fallible to match the
durable snapshot main implements, and the generic laws assert typed
refusals instead of bare errors. The branch's `ChunkSink` generalization
of the streaming core is not included because nothing on main consumes it
yet; it travels with the durable ingestion port.
