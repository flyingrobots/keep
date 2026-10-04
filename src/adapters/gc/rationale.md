# GC boundary decisions

## Reader-lock coordinate roles

The public GC and recovery-disposition record API uses distinct `ReaderLockDevice`, `ReaderLockMount` and `ReaderLockFile` values. `ReaderLockIdentity` admits these roles explicitly, so passing a correctly labeled coordinate in another position is a compile-time error. This decision tightens the new API in #107; it does not retrofit unrelated filesystem identity APIs.

Each wrapper preserves the complete unsigned 64-bit wire domain, including zero. These are opaque coordinates, not proof that an object was observed, a lock acquired, or an operation made durable. The boundary that observes or decodes a number assigns its role; a caller can still deliberately mislabel a raw number, so the types do not replace runtime evidence or code review.

The mount identifies a mount instance and remains same-process evidence. This change does not alter the existing restart comparison contract, record layout, endianness, checksums, mismatch coordinates or refusal ordering. Codecs unwrap values only at comparison and byte-encoding boundaries. Filesystem adapters label the observed device, mount and file values where they enter the record API.

A primitive triple and type aliases were rejected because either permits coordinate interchange. New zero or range refusals were rejected because the format admits every unsigned 64-bit coordinate. Explicit named wrappers keep those facts separate and require no allocation, blocking, I/O or new dependency.

The source-level compatibility change requires callers to wrap raw values with the appropriate constructor and unwrap getter results with `get()`. Static compile-fail examples guard pairwise interchange; existing frozen runtime laws and generated cross-revision comparisons guard the separate byte-preservation claim. A compiler rejection is static/API evidence, not a storage runtime test.

## Fixed receipt encoding

The retirement receipt grammar is a fixed array. Public values change bytes, never field widths or record length. A private construction macro binds every emitted expression to an array of its declared width and equates the sum with the destination array length at compile time. The same construction joins the field area, reserved area and checksum. Those two linked checks prove that iterator writes fill exactly the destination; an unproved truncating zip would not be sufficient.

This replaces panicking slice operations while retaining the infallible public constructor and its fixed stack allocation. A future field-width or framing mismatch fails compilation. A heap buffer and a new public encoding-error variant were rejected because no runtime input can cause a layout failure in this construction. Existing exact-byte runtime laws and generated cross-revision evidence guard serialization behavior separately from the static proof. The macro remains private to this encoder; this decision does not certify other codecs.
