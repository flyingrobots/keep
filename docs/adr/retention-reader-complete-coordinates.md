# Complete retention reader coordinates

Status: accepted.

Reader double collection must compare every validated semantic field of both selected heads, including catalog length and retention manifest length and predecessor.

The filesystem reader previously projected both heads to generation and digest, so a correctly checksummed catalog head with a changed admitted length could appear unchanged and select a view that the later head would not admit.

Catalog coordinates now retain validated generation, length, and digest; retention coordinates retain the existing validated `RetentionHead` value, whose equality includes generation, manifest length, digest, and predecessor.

Invalid head checksums and other decoding failures refuse through the existing source-error boundary; they are not retried as successfully decoded coordinate changes.

This intentionally changes the unreleased public `RetentionViewCoordinates` field types, requiring port implementations to supply the missing validated coordinates; it does not change durable bytes, identity preimages, retry limits, or write/recovery protocols.

Retaining exact encoded head bytes was rejected because the storage-independent collection port exchanges validated semantic values rather than codecs or filesystem representations.

Full coordinate equality still cannot observe an arbitrary external replacement and restoration entirely between its reads, or establish that an arbitrary port's loaded value actually belongs to those coordinates; these are separate limits of the existing collection contract.
