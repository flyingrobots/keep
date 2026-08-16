# Authenticated Reconstruction Rationale

## Decision

Keep names authenticated reconstruction as one proof-scoped operation rather
than a generic blob lookup. A successful operation emits exact bytes and
returns a receipt for the proposition it established. An evidenced refusal
supports a narrower content proposition. An operational failure supports no
content conclusion.

Complete-object and exact-range operations remain separate. Complete-object
reconstruction verifies every chunk, the registered storage-profile
boundaries, and the complete `BlobId`. Exact-range reconstruction verifies the
complete identities of overlapping chunks under a layout-to-target binding
admitted by the selected store view. It verifies neither the complete blob nor
any storage-profile boundary.

The generic output boundary remains an ordinary caller-owned `Write`. A
successful receipt authenticates the complete emitted sequence. Failure may
leave an untrusted prefix, so consumers that require atomic visibility must
quarantine output and publish it transactionally after receipt validation.

Current `ReferenceStore` behavior is the executable oracle for non-durable
complete-object and range forms. It is not evidence that Keep has a durable
logical reconstruction API. A future durable form must pin one immutable view,
retain its complete supporting evidence, and bind the view into its result.

## Governed surfaces

This decision governs:

- logical identity versus physical realization;
- complete-object and exact-range proof scopes;
- receipt and refusal meaning;
- output visibility after failure;
- layout selection and committed layout-to-target binding;
- the future durable read aperture and evidence-retention obligation; and
- the public integration boundary available to consumers.

It does not govern Echo semantics, causal authority, application retry law, or
cross-store publication.

## Alternatives rejected

### Return `Option<Arc<[u8]>>`

This collapses absence, corruption, unavailable evidence, and operational
failure. It also requires hidden whole-object materialization and cannot carry
proof scope or a receipt.

### Use one read operation with an optional range

An optional range makes it easy to pass a range receipt where complete-object
proof is required. Separate operations and receipt types keep the narrower
proof scope visible.

### Trust any structurally admitted caller layout for range reads

Structural validation proves offsets, lengths, and canonical layout shape. It
does not prove that the target `BlobId` names the listed chunks. Range receipts
therefore require the exact layout-to-target binding admitted by the selected
store view.

### Treat every error as an evidenced refusal

Writer failure, cancellation, resource exhaustion, and unreadable evidence do
not establish a proposition about content. Converting them into refusals would
let infrastructure weather become false storage truth.

### Promise atomic output from an ordinary `Write`

An ordinary writer can accept a prefix and fail. Keep cannot roll back an
arbitrary external sink. Transactional visibility belongs to a consumer or
adapter that owns a quarantine and commit protocol.

### Treat a receipt as a portable durable proof

Current receipts state what one completed process established. They do not
carry all supporting evidence and remain replayable only while named evidence
is retained under the same contract.

## Consequences

- Consumers must preserve proof scope and outcome class.
- Range receipts cannot satisfy complete-object requirements.
- Caller-supplied range layouts must resolve through an admitted store view.
- A failure after output began returns no success receipt; accepted bytes
  remain untrusted.
- Durable integration remains blocked on a pinned-view consumer capability and
  evidence retention.
- Application-specific meaning remains outside Keep core.
