# Non-Durable Reference CAS Rationale

## Decision

Keep provides a capacity-bounded in-memory `ReferenceStore` as its first public
content-addressed storage adapter. Ingestion is blocking and streaming, staged
work owns new unique chunk bytes, visibility requires an explicit commit, and
whole-blob reconstruction authenticates chunks, registered profile boundaries,
and the complete `BlobId` before output.

The adapter makes no retention, recovery, crash-durability, or power-loss
durability claim.

## Why materialize in the reference adapter

Issue #13 needs executable storage behavior before the durable segment protocol
exists. An in-memory adapter keeps the first implementation auditable: logical
identity, chunk deduplication, layout admission, publication, and
reconstruction can be tested without conflating them with filesystem ordering
or recovery.

Materialization is explicit rather than hidden. `ReferenceStoreCapacity` bounds
owned chunk bytes, `LayoutEntryLimit` bounds layout metadata, and `StagedBlob`
reports the number and bytes of chunks absent from the store used during
staging. Commit rechecks that another destination already owns any required
chunks omitted by that deduplication.

## Why stage before commit

Reading, chunking, hashing, allocation, and canonical layout calculation can
all fail. Publishing incrementally would expose a prefix without a complete
`BlobId` or admitted layout. Owned staged work therefore remains invisible and
`#[must_use]`; commit is the explicit synchronous visibility transition.

Commit revalidates all state that intervening work could affect. Missing or
inconsistent committed state refuses instead of being silently repaired.
Repair belongs to a future explicit recovery protocol with its own evidence.

## Why authenticate before output

Writing a verified prefix before discovering a later missing chunk, false
profile boundary, or full-blob mismatch would expose bytes from an
unauthenticated claim. Reconstruction first verifies the entire plan without
output. It then reverifies each chunk immediately before writing because the
output pass is a separate traversal.

This costs two chunk-verification passes. Correct refusal and a simple audit
story outweigh throughput until measured evidence justifies another design.

## Alternatives considered

- **Publish chunks as they arrive.** Rejected because later source, allocation,
  layout, or identity failure would leave a visible prefix.
- **Return bytes after chunk verification only.** Rejected because individually
  correct chunks can still be ordered under the wrong complete `BlobId` or
  divided by false storage-profile boundaries.
- **Buffer the complete blob and write once.** Rejected because it hides a
  whole-blob allocation and duplicates the reference store's explicit
  materialization.
- **Treat commit as durable.** Rejected because no filesystem write, flush,
  synchronization, journal, or recovery protocol exists.
- **Repair missing committed state during normal publication.** Rejected
  because it would erase evidence of corruption and make ordinary success
  ambiguous.

## Consequences

- The adapter is deterministic and straightforward to model, but unsuitable
  for durable application data.
- Staging memory can grow with unique content only up to explicit capacity.
- Layout metadata remains bounded but can be large at the protocol maximum.
- Reconstruction performs two verification passes before reporting success.
- Durable storage must implement a different adapter with documented
  publication order, crash states, recovery behavior, and synchronization.
