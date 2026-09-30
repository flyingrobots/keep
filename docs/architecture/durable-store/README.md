# Durable Store Reads

`DurableStore` is the durable twin of the
[non-durable reference store](../reference-store/README.md)'s read surface:
authenticated `reconstruct`, `reconstruct_layout`, `read_range`, and
`read_layout_range` over one fenced version-two snapshot, with the same laws
and the same reconstruction and range cores, plus a receipt that names the
view. It is a read surface only: durable ingestion is
[F-24](../../../ROADMAP.md#f-24-bounded-production-ingestion-through-the-durable-store),
and content reaches a store through catalog publication and retention
anchoring.

## Contract

`DurableStore::open(root, policy, limit)` names a migrated
`keep.segment-store/v2` root, touching nothing. `snapshot()` pins one view:
it admits the root as version two, acquires the shared `reader.lock`
fence, double-collects one consistent catalog head, retention head, and
manifest under it, and indexes every retained root's anchors. The
`DurableSnapshot` owns the fence for its lifetime; every read borrows it, so
a view cannot be dropped mid-read, and no collector can retire a segment it
may read (`FilesystemGcAuthority` refuses `ReadersActive` rather than
waiting). Publication proceeds beside snapshots because it only adds
immutable successors: a pinned snapshot keeps reading its generation while
a compaction publishes the next.

Resolution is exact and evidence-bound:

- a blob resolves through the retained anchors, canonically first layout
  first; `contains_blob` is anchor membership, and a blob the catalog can
  serve but no root anchors is `BlobMissing`;
- an exact layout resolves through the pinned catalog's layout record,
  decoded under the reader's entry limit and bound to the requested
  `LayoutId`, never substituted;
- every chunk resolves through the pinned catalog's chunk record and is
  hashed by the shared read core before a byte is emitted.

The receipt is the reference receipt (target, exact layout, requested range
where applicable, emitted length) bound to the `DurableView`: catalog
generation and digest, and the retention generation and manifest digest
observed under the same fence. The same coordinates are a
[verification receipt](../../formats/verification-receipt-v1/README.md)'s
durable view.

Memory: the pinned catalog holds the segment bytes the reader's
`CatalogRestartPolicy` admits; chunks are emitted as borrowed slices of
those admitted records, one chunk at a time, with no whole-blob buffer.

## Outcomes

`DurableReadError::View` is the one operational failure at the read
boundary: the pinned catalog could not be re-admitted, and nothing about
content follows. `BlobMissing`, `LayoutMissing`, `LayoutDecode`, and the
reference cores' missing, hash, identity, and length refusals are evidence
against the complete pinned view; the cores' `Output` failures remain
operational and report the exact accepted prefix, which stays untrusted.
Pinning itself refuses as `DurableStoreError` at the admission, fence,
collection, or retained-root boundary.

## Evidence

`src/adapters/durable/tests.rs` over a migrated store with several anchored
blobs (one spanning several chunks) and one committed but unanchored
layout: exact reconstruction with receipts naming the view; ranges across
chunk boundaries emitting exactly the requested bytes and refusing past the
end; absence as evidence with the exact committed layout still readable;
a pinned view surviving a compaction successor and blocking collection
until dropped; identical views yielding identical receipts across reopen;
a refusing writer receiving no receipt beyond its accepted prefix.
`KEEP-RECONSTRUCT-009` and `-010` are Implemented on that evidence.

The Golden File Worldline runs its storage steps against the reference
store; a durable run needs the durable writer (T-24.2) to ingest the
worldline's states and is owed with it.
