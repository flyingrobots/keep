# Authenticated Reconstruction Contract

**Status:** Normative for every Keep operation that claims authenticated
reconstruction. The public non-durable `ReferenceStore` implements the
complete-object and exact-range forms. A consolidated durable logical-read
surface is not yet implemented.

## Contract

For a requested content identity, Keep either:

1. establishes the proof scope named by the operation;
2. emits exactly the bytes supported by that proof;
3. returns a receipt stating the exact proposition established;

or it returns a precise, evidenced content-related refusal;
or it returns an operational failure that makes no claim about content truth.

Keep never substitutes different content, promotes a narrower proof into a
broader one, infers truth from physical existence, or silently repairs
ambiguous state.

```text
typed content coordinate
+ admitted immutable evidence
+ declared proof scope
+ caller-owned output
────────────────────────────
authenticated bytes + receipt
                     or evidenced refusal
                     or operational failure
```

This contract refines Keep's core law:

> For a given content identity, Keep must return exactly the bytes named by
> that identity—or refuse.

## Meaning of authenticated

In this contract, **authenticated** means that Keep established the emitted
bytes against the requested Keep content coordinate using the admitted
identity law and the evidence required by the operation's proof scope.

Authentication here does not establish:

- authorship or provenance;
- application meaning;
- trustworthiness of the content;
- caller authorization;
- causal or publication authority outside Keep;
- durability beyond the evidence named by the receipt.

Those propositions require their own owners and evidence.

## Coordinates and evidence

A `BlobId` names one exact finite logical byte sequence. It is independent of
chunking, layouts, representations, physical locations, and retention. Parsing
a `BlobId` proves only that the coordinate is canonical and supported. It does
not prove presence or retention.

A `LayoutId` names one canonical reconstruction plan. One `BlobId` may have
more than one admitted layout. Every admitted layout binds exactly one target
`BlobId`.

Physical paths, file names, offsets, inode values, object keys, and successful
raw reads are evidence inputs, not stable content identity. They cannot
authorize output without the required identity and structural verification.

The following claims remain distinct:

```text
canonical coordinate
    ≠ content present
    ≠ content published
    ≠ content retained
    ≠ content durable
    ≠ content reconstructible from this admitted view
```

## Complete-object reconstruction

A successful complete-object reconstruction proves that:

- every selected chunk matched its `ChunkId`;
- the admitted storage-profile boundaries matched the layout;
- the complete reconstructed sequence matched the requested `BlobId`;
- the exact byte count reported by the receipt was written.

The current public reference shape is:

```rust
fn reconstruct<W: std::io::Write>(
    &self,
    target: BlobId,
    output: &mut W,
) -> Result<ReconstructionReceipt, ReconstructionError>;
```

When more than one committed layout names the requested blob and the caller
does not request an exact layout, `ReferenceStore` deterministically selects
the lowest canonical `LayoutId`.

When the caller requests an exact `LayoutId`, Keep must use that layout or
refuse. It must not fall back to another layout, even if another layout could
lawfully reconstruct the same `BlobId`.

Different bytes or a different target `BlobId` are never lawful substitutes.

## Exact-range reconstruction

An exact-range operation has a deliberately narrower proof scope. Its receipt
proves that the requested bytes came from completely authenticated overlapping
chunks under the admitted layout.

It does not prove that Keep authenticated:

- unrequested chunks;
- the complete logical blob;
- every storage-profile boundary.

A `RangeReadReceipt` must never satisfy an API that requires a complete-object
`ReconstructionReceipt`. Complete and range operations remain separate public
surfaces; an optional range parameter must not erase the proof distinction.

## Output visibility and failure

The generic reconstruction API accepts an ordinary caller-owned `Write` sink.
An ordinary sink is not transactional. It may fail after accepting a prefix,
and an emission-time storage failure may occur after a prefix has been
written.

Therefore:

> A successful receipt authenticates the complete emitted sequence. On
> failure, any bytes already written are uncommitted and must not be consumed
> as authenticated output.

Callers that require atomic visibility must provide a quarantined or
transactional sink and reveal or promote its bytes only after validating the
complete success receipt. Keep does not claim that an unsuccessful call left
an arbitrary `Write` untouched.

The current `ReferenceStore` verifies the complete realization before its
first output write, then reverifies each immutable chunk immediately before
emission. This prevents known unauthenticated content from being emitted; it
does not make the caller's sink atomic.

## Decisions, refusals, and operation failures

A typed error does not automatically constitute evidence about content truth.
Consumers must distinguish at least:

- authenticated success;
- an evidenced refusal supported by a complete admitted view;
- an operational failure from which no content conclusion follows.

For example, absence can be evidenced only when the admitted view and its
indexes are complete enough to prove non-membership. A timeout, unreadable
catalog, exhausted resource limit, cancellation, or unavailable capability
does not prove absence.

Corruption evidence establishes only the exact integrity proposition its
evidence supports. An unreadable or internally inconsistent view may prevent
both a positive reconstruction claim and a negative absence claim.

The current `ReferenceStore` exposes boundary-specific Rust errors rather than
a persisted refusal-receipt format. Any future durable refusal receipt must
bind enough coordinates to replay its proposition, including:

- the requested `BlobId`;
- the admitted immutable view or generation;
- an exact requested `LayoutId`, when present;
- refusal classification;
- proof stage or evidence coordinate;
- contract version.

## Receipt posture

A reconstruction receipt is, by default, an ephemeral statement about one
completed operation. It is not automatically a portable, self-contained
cryptographic proof.

A receipt may support later revalidation only when all evidence it names
remains available and admitted under the same contract. A durable receipt must
name the immutable store view or generation against which the result was
established. Retaining a receipt without retaining its supporting evidence
does not preserve the original proof.

Current `ReconstructionReceipt` values bind:

- target `BlobId`;
- selected `LayoutId`;
- exact authenticated byte count written.

Current `RangeReadReceipt` values additionally bind the requested half-open
range and explicitly carry the narrower range proof posture.

## Durable reconstruction requirement

A future operation claiming durable logical reconstruction must additionally:

- bind reads to one admitted immutable snapshot or catalog generation;
- prevent required supporting evidence from being garbage-collected, deleted,
  or otherwise invalidated during the read;
- verify the retained closure required by its declared proof scope;
- resolve and authenticate exact immutable records;
- preserve the selected view while successor generations publish;
- return a receipt naming that view;
- separate evidenced refusal from operational failure;
- preserve the output-visibility rule above.

The current durable segment, catalog, publication, retention, and recovery
surfaces do not yet form this consolidated high-level `BlobId`-to-writer
contract. Their existence must not be described as an implemented durable
logical reconstruction API.

## Current public evidence

The non-durable `ReferenceStore` is the executable oracle for this contract.
Its committed state is process memory; process death loses it all.

Evidence anchors:

- [`ReferenceStore` architecture](../../architecture/reference-store/README.md)
- [`ReferenceStore` rationale](../../architecture/reference-store/rationale.md)
- [exact logical byte identity](../../adr/0001-exact-logical-byte-identity.md)
- [identity and physical-storage separation](../../adr/0002-separate-identity-from-physical-storage.md)
- [`ReferenceStore` contract tests](../../../tests/reference_store_contract.rs)
- [reconstruction implementation](../../../src/reference/reconstruction.rs)
- [range-read implementation](../../../src/reference/range_read.rs)
- [reconstruction receipt](../../../src/reference/reconstruction_receipt.rs)
- [range-read receipt](../../../src/reference/range_read_receipt.rs)

## Consumer rule

Consumers may wrap Keep in application-specific ports, transactional output,
identity bindings, and causal workflows. Those adapters may strengthen output
visibility or attach additional meaning. They must not weaken Keep's proof
scope, treat an operational failure as content evidence, or import
application-specific semantics into Keep core.
