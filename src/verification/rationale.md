# Subject-specific verification evidence

Status: implementation in progress for #114.

Verification reports name the evidence established for a subject; the enum's ordering does not turn a catalog-membership proof into complete logical blob verification.

The domain owns immutable report, subject, depth, and refusal vocabulary without importing storage adapters.

Adapters construct reports only after admitting the corresponding evidence.

The initial catalog operation reports existing admitted evidence and explicitly documents the admission work already performed before reporting.

An unsupported request returns its exact subject, requested depth, and supported set, because the supported catalog depths are not a contiguous interval.

Using a minimum/maximum range would incorrectly admit `CompleteBlobIdentity` between `Checksum` and `CatalogReachability`.

Reports expose a subject slice and private construction; the initial single-subject representation adds no allocation and does not imply that aggregate or other-subject verification has been implemented.

Copying a report preserves its claims and carries no fence or retention authority.

The prepared reference-store report was not imported unchanged because its layout/blob-only coordinates do not describe the durable catalog subject required here.

No durable format, hashing preimage, write protocol, or recovery action changes in this slice.

Segment reports certify only physical framing/checksum evidence; chunk and layout claims are attached to their own logical record subjects, so an empty physical segment cannot masquerade as evidence of some logical object.

`SegmentDigest` now lives in a domain-owned module rather than under the codec adapters; its public name and byte representation are unchanged, and reporting does not introduce an inward dependency on filesystem or codec implementations.

Logical-record reports use their admitted immutable payload evidence without claiming catalog membership or publication, including records prepared for writing but not persisted.

The supported-depth matrix is explicit per subject, and reporting never uses enum ordering to accept a request.

Logical verification now names its exact catalog provenance and constructs a report only after the requested work succeeds; no failing anchor can return a root-closure report.

Blob discovery uses canonical layout order, matching the distinction between multiple lawful representations and conflicting evidence; it holds only one decoded layout at a time rather than constructing an additional whole-catalog index.

The semantic refusal keeps bounded coordinates inline; narrowly scoped Clippy expectations preserve precise diagnostics without introducing extra allocation solely to reduce an error enum's stack footprint. Original adapter causes are boxed only on error and remain typed.
