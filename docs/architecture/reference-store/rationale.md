# Non-Durable Reference CAS Rationale

## Decision

Keep provides a capacity-bounded in-memory `ReferenceStore` as its first public
content-addressed storage adapter. Ingestion is blocking and streaming, staged
work owns new unique chunk bytes, visibility requires an explicit commit, and
whole-blob reconstruction authenticates chunks, registered profile boundaries,
and the complete `BlobId` before output. Exact range reads authenticate only
the complete chunks overlapping the requested bytes and report that narrower
claim explicitly.

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

## Why range reads authenticate selected chunks only

Verifying the complete `BlobId` or replaying every storage-profile boundary
would require loading content outside the requested range. That would violate
the minimal-overlap capability and turn a range API into disguised whole-blob
I/O.

Range reads therefore plan from admitted metadata, authenticate every complete
overlapping chunk before output, then reauthenticate each chunk immediately
before slicing and emission. Their receipt names the requested range and
explicitly does not claim complete-blob identity, unrequested chunks, or
storage-profile boundaries. Callers choose whole-blob reconstruction when they
need those stronger claims.

Preverification ensures a later selected chunk cannot fail after an earlier
range byte has been emitted. Reverification protects the separate output pass
without buffering selected chunks or the requested result.

## Why caller-supplied ranges require a committed layout

Structural admission does not prove that a layout's target `BlobId` names the
chunks it lists. A same-length target can be paired with unrelated chunk
identities while preserving every structural law. A partial read cannot
authenticate that complete target without loading the complete blob.

Caller-supplied admitted layouts and canonical records are therefore used only
to calculate a `LayoutId`. That identity must already name a committed layout
in the store, and the range operation uses the committed layout for planning,
receipt coordinates, and chunk lookup. An uncommitted target-layout
association refuses before output.

## Alternatives considered

- **Publish chunks as they arrive.** Rejected because later source, allocation,
  layout, or identity failure would leave a visible prefix.
- **Return bytes after chunk verification only.** Rejected because individually
  correct chunks can still be ordered under the wrong complete `BlobId` or
  divided by false storage-profile boundaries.
- **Buffer the complete blob and write once.** Rejected because it hides a
  whole-blob allocation and duplicates the reference store's explicit
  materialization.
- **Verify the complete blob for every range.** Rejected because it loads
  unrequested chunks and misrepresents whole-blob I/O as a minimal range read.
- **Verify and emit each selected chunk in one pass.** Rejected because a later
  selected-chunk failure could follow already emitted range bytes.
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
- Exact range reads perform two verification passes over only the selected
  chunks and deliberately make no complete-blob verification claim.
- Durable storage must implement a different adapter with documented
  publication order, crash states, recovery behavior, and synchronization.
