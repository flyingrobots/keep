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
[`ReferenceStore`](../reference-store/README.md) and by the pinned
[`DurableSnapshot`](../durable-store/README.md). Each backend keeps its own
receipt and error types as associated types: the durable receipts carry the
view coordinates, the reference receipts do not, and neither converts into
the other.

## The write half: `ContentStaging` and `StagedContent`

`ContentStaging::stage(source, limits)` chunks, hashes, and holds an
unknown-length source without making anything visible;
`stage_expected(source, expected, limits)` additionally refuses when the
complete source does not hash to `expected`, naming both identities.
`StagingLimits` carries a `LayoutEntryLimit` and a `StagedByteLimit`. Both
refuse before the excess is materialized: the byte limit is checked as each
read is accepted, and the refusal
(`IngestionError::ByteLimitExceeded { limit, accepted, incoming }`) says
how far the source was admitted and what pushed it over.

A staging borrows its store (`ContentStaging::Staged<'store>`), so the
store cannot change underneath it and `StagedContent::commit(self)` needs
no second reference: it makes the staging visible, or leaves it invisible,
and returns a `CommitReceipt` naming the target and the exact committed
layout.

The non-durable `ReferenceStore` implements the write half through
`ReferenceStagedContent` and stays honest about being non-durable: process
death loses everything in it, and no port method claims otherwise. The
[`DurableWriter`](../durable-store/ingestion.md) implements it on disk,
with deduplication against the pinned catalog and publication through the
catalog protocol; its receipt is `DurableIngestionReceipt`.

## Moving bytes out and between

The [transfer pipeline](pipeline.md) moves authenticated bytes from any
`ContentReads` view into a `TransferSink` as verified segments under a
window and a cancellation signal, and `copy_layout` moves a blob between
any `TransferSource` and any `ContentStaging` destination without
buffering it.

## The receipt law

A non-durable receipt can never stand where a durable one is required. The
port does not name a common receipt type; each backend's receipt is its own
type, so the substitution is a compile error. `src/store/mod.rs` pins this
with a `compile_fail` doctest that hands a `ReconstructionReceipt` to a
function taking a `DurableReconstructionReceipt`.

## Evidence

- `src/store/port_laws.rs` holds the generic laws (exact reconstruction
  through both entry points with equal receipts, exact ranges plus a past-
  the-end refusal, absence refused with nothing written); the reference
  backend runs them in `src/store/reference_port_tests.rs` and the durable
  backend in `src/adapters/durable/port_tests.rs`. The durable run is
  in-crate because a durable store is built today only through test-only
  unchecked admission.
- `tests/content_store_port.rs` runs the reference backend from outside the
  crate through the port alone: round trip, byte-limit refusal before
  anything is visible, entry-limit refusal, and expected-identity mismatch.
- The durable writer's own laws are in `src/adapters/durable/writer_tests.rs`.
- The Worldline golden run through the port is owed; it is listed under
  T-24.2 in the roadmap.
