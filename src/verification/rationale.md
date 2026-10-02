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
