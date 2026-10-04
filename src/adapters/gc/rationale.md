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

## Execution failures and namespace evidence

GC mutation and directory synchronization are separate capabilities. `GcExecutionError::executed()` records successful calls, not a list of newly applied or durably completed effects. A successful link can adopt an existing entry; a successful unlink can precede a failed sync. Inferring rollback or durability from that list was rejected because neither follows from a call returning success.

The filesystem implementation preserves the original I/O or typed refusal and annotates the failing capability with the shared `RetentionStorageProgress` vocabulary. `storage_progress()` exposes that payload directly. Existing detailed stage reports pass through unchanged; bare preparation failures receive a pre-effect boundary. Candidate/intent unlink failures report uncertainty; successful unlink followed by failed absence verification reports known removal with unconfirmed directory durability. Directory-sync failure names its precise synchronization boundary and has no namespace effect of its own; earlier successful calls remain separate. An empty failing-capability effect list does not negate those earlier calls. Third-party storage implementations may omit the payload, in which case effects are unreported, never assumed absent.

Execution stops at the first error, and public filesystem execution drops its in-memory context before returning. A new attempt must observe retained residue; returning an existing complete receipt on restart does not retrospectively prove that an earlier failed directory sync succeeded. Original reader fencing, writer authority, no-follow and exact-byte checks remain. This decision changes diagnostic vocabulary and adds a public accessor, without changing durable bytes or operation ordering. New public effect variants require exhaustive matches to be updated. Disposition and nested-directory authority integration remain separate acceptance obligations.
