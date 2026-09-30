# Durable Formats

Keep treats every durable format as a versioned protocol. A format is not
admitted merely because one Rust type can serialize and deserialize it.

## Format registry

| Format | Coordinate | Status | Evidence |
| --- | --- | --- | --- |
| [Flat Chunk Layout v1](flat-chunk-layout-v1/README.md) | `keep.flat-chunks/v1` | Implemented through verified reconstruction in issues #10 and #13 | [Golden corpus](../../conformance/layout/v1/README.md) |
| [Durable Segment Store v1](segment-store-v1/README.md) | `keep.segment-store/v1` | Implemented through initialization, publication, restart, and recovery in issues #14–#17 | [Golden corpus](../../conformance/segment-store/v1/README.md) |
| [Verification Receipt v1](verification-receipt-v1/README.md) | `keep.verification-receipt/v1` | Canonical 384-byte replayable projection of one verification report or refusal onto one view; codec, corpus, corruption matrix, and fuzz target implemented | [Golden corpus](../../conformance/verification-receipt/v1/README.md) |
| [Durable Segment Store v2](segment-store-v2/README.md) | `keep.segment-store/v2` | One-way migration, version-two reopen, retention publication and recovery, reader fencing, explicit disposition, GC retirement, and identity-preserving compaction implemented; benchmarks and re-encoding compaction planned in issue #21 | [Golden corpus](../../conformance/segment-store/v2/README.md) |

The registry records protocol specifications, including formats whose
implementation is still planned. Each format page states its exact proof
boundary and nonclaims.
