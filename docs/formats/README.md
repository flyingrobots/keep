# Durable Formats

Keep treats every durable format as a versioned protocol. A format is not
admitted merely because one Rust type can serialize and deserialize it.

## Format registry

| Format | Coordinate | Status | Evidence |
| --- | --- | --- | --- |
| [Flat Chunk Layout v1](flat-chunk-layout-v1/README.md) | `keep.flat-chunks/v1` | Implemented through verified reconstruction in issues #10 and #13 | [Golden corpus](../../conformance/layout/v1/README.md) |
| [Durable Segment Store v1](segment-store-v1/README.md) | `keep.segment-store/v1` | Specified in issue #14; production implementation is planned in issues #15–#17 | [Golden corpus](../../conformance/segment-store/v1/README.md) |

The registry records protocol specifications, including formats whose
implementation is still planned. Each format page states its exact proof
boundary and nonclaims.
