# Durable Formats

Keep treats every durable format as a versioned protocol. A format is not
admitted merely because one Rust type can serialize and deserialize it.

## Format registry

| Format | Coordinate | Status | Evidence |
| --- | --- | --- | --- |
| [Flat Chunk Layout v1](flat-chunk-layout-v1/README.md) | `keep.flat-chunks/v1` | Implemented through verified reconstruction in issues #10 and #13 | [Golden corpus](../../conformance/layout/v1/README.md) |
| [Keep store format](store/README.md) | `keep.segment-store/v2` | The one store format: direct initialization and reopen, the segment, catalog, publication, and stage-recovery protocol of issues #14–#17, the gated catalog writer, and retention publication and recovery implemented; collection planned in issue #21 | [Segment, catalog, and head corpus](../../conformance/segment-store/v1/README.md) · [Store corpus](../../conformance/segment-store/v2/README.md) |

`keep.segment-store/v1` is retired as a store format (#198): no code admits a
version-1 root. Its segment, catalog, and publication-head grammars continue
unchanged inside the store format.

The registry records protocol specifications, including formats whose
implementation is still planned. Each format page states its exact proof
boundary and nonclaims.
