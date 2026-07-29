# Keep

**Correctness-first content-addressed storage.**

> For a given content identity, Keep must return exactly the bytes named by
> that identity—or refuse.

Keep is a standalone Rust library for durable, content-addressed storage. It is
intended to provide streaming ingestion, content-defined chunking, physical
deduplication, exact range reads, explicit retention, integrity verification,
crash recovery, and garbage collection without relying on Git or subprocesses
in the storage path.

## Status

Keep exposes strict, versioned `BlobId`, `ChunkId`, `StorageProfileId`, and
`LayoutId` coordinates. It implements the frozen `fastcdc-64k-v1` detector and
the canonical `keep.flat-chunks/v1` layout codec with language-neutral golden
and mutation corpora.

The public
[non-durable reference CAS](docs/architecture/reference-store/README.md)
provides capacity-bounded blocking ingestion, identity-based chunk
deduplication, an explicit staged-to-visible transition, authenticated
whole-blob reconstruction, and authenticated exact byte-range reads.
Reconstruction verifies every chunk, replays the registered storage profile,
and verifies the complete named `BlobId` before writing any bytes. Range reads
load only the minimal overlapping chunks and state their narrower verification
claim explicitly.

The public `keep.segment-store/v1` boundary provides exact segment, record,
seal, catalog, and publication-head codecs plus explicit immutable-segment and
catalog-generation transitions. `StagedSegment` writes only content-admitted
chunk or layout records, while `AdmittedSegment` exposes payloads only after
complete framing, checksum, logical-identity, duplicate, and physical-digest
verification. A platform-admitted `FilesystemCatalogPublisher` exclusively
creates the fixed `current.seg` stage without truncating existing evidence,
and the `FilesystemSegmentStage` lifetime keeps that writer authority borrowed
until the writable stage closes. Publisher construction consumes an
unforgeable `FilesystemPlatformAdmission`. On Linux, its public initializer
admits only a writable, non-casefolded ext4 root, refuses unknown or aliased
namespace entries before mutation, creates or verifies the canonical
`writer.lock`, `staging`, `segments`, and `catalogs` shape, and returns only
after root synchronization with the writer lock retained.

`FilesystemCatalogPublisher` retains one kernel-managed writer lock and pinned
root, staging, segment-pool, and catalog-pool capabilities for the complete
blocking publication. It reopens and verifies synchronized stages, uses
no-replacement immutable-pool links, synchronizes every required file and
directory, verifies the complete `head.next` view, atomically replaces `HEAD`,
and returns a receipt only after root synchronization. New filesystem segment
publication requires `FilesystemCatalogPublisher::select_segment` to consume
the sealed writable stage, prove that this publisher created it, and bind its
synchronized metadata to exact admitted bytes. A storage-agnostic
`ClosedSegment` receipt alone cannot authorize a retained filesystem stage.
`FilesystemCatalogSnapshot` follows only the exact checksummed head, catalog,
and segment coordinates and retains caller-bounded immutable bytes for pinned
logical reads.

The reference CAS is executable evidence for M2 storage laws, not a durable
backend. Its committed state is process memory; process death loses it all.
The durable boundary can initialize and platform-admit a store only under the
documented Linux ext4 contract. Acquiring `FilesystemWriterLock` alone cannot
construct a filesystem publisher. Leftover `head.next`, staged recovery
evidence, and ambiguous crash states remain explicit recovery work. An absent
`HEAD` is admitted for first publication only when both immutable pools are
empty. The public storage-independent recovery inventory counts all four
protocol namespaces before retaining names, applies a configurable ceiling no
greater than 2,097,152 entries, and returns duplicate-free deterministic raw
name order. `FilesystemRecoveryInventoryReader` implements that contract with
pinned, no-follow namespace capabilities and pre/post identity verification on
the admitted Linux ext4 profile. Its bounded stage-fingerprint operation opens
fixed stages relative to those capabilities, refuses links and nonregular
files, and verifies entry identity and length after reading. Complete
caller-supplied segment-stage bytes can be classified as a reusable prefix,
complete admitted segment, or exact truncation. Catalog and next-head stages
likewise distinguish exact truncation from complete canonical bytes.
Materialized bytes enter read-only semantic assessment only after their stage,
length, and recomputed fingerprint match prior observation evidence.
Only an exact truncation assessment may form an explicit discard request; the
semantic executor refuses evidence drift and returns a receipt only after
exact removal or admitted prior absence is followed by parent synchronization.
The pinned-filesystem adapter retains writer authority, revalidates stage
evidence without following links, removes only an exact match, and
synchronizes the protocol-selected parent. Transitive publication-view
admission and filesystem-streaming classification remain planned.
Crash-injection execution, stage completion and next-head finalization,
retention, compaction, and garbage collection remain planned. Presence in the
reference CAS does not claim retention, crash recovery, or durability.

```rust
use keep::BlobId;

let identity = BlobId::hash_bytes(b"exact bytes")?;
let canonical = identity.to_string();
assert_eq!(canonical.parse::<BlobId>()?, identity);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Design boundary

Keep owns physical content storage:

- exact byte identity;
- chunking and physical representation;
- streaming and range reads;
- retention roots and storage generations;
- verification, recovery, compaction, and garbage collection;
- optional storage encryption.

Keep does not own application semantics. In particular, the core library must
remain independent of Echo, Git, Graft, WARP, command-line interfaces, and
application policy.

An application may give stored bytes causal meaning, authority, provenance, or
publication status. Keep reports only what its physical evidence can support.

## Engineering standard

Development is governed by the normative
[Keep Rust Engineering Standard](docs/Rust%20Standards.md). Correctness,
recoverability, auditability, and maintainability outrank performance and
convenience.

Documentation is governed by the
[Keep Documentation Standard](docs/Documentation%20Standards.md), which maps
reader tasks onto Keep's architecture, invariant, format, and recovery
corpus.

The initial implementation will use:

- stable Rust 1.96.0;
- Rust edition 2024;
- one writer and many readers unless a stronger concurrency model is designed;
- synchronous core APIs until a demonstrated consumer requires otherwise;
- versioned, canonical, independently testable durable formats;
- no unsafe Rust in Keep-owned version-1 crates; dependency unsafe requires an
  explicit review and cannot alter canonical identity.

## Golden File Worldline

The first executable vertical is split into deliberately narrow milestones. M1
proves that exact finite logical bytes have one canonical versioned identity,
that calculation is invariant to tested input partitioning, that malformed or
unsupported identity encodings are refused precisely, and that a bounded
reference model returns exactly the bytes named by an admitted identity or
refuses.

The complete Golden File Worldline is planned to demonstrate that Keep can:

1. ingest exact logical bytes;
2. retain and recover multiple nearby versions;
3. reuse stable chunks after an early insertion;
4. read an exact byte range without materializing the whole blob;
5. refuse corrupted or ambiguous storage;
6. recover to a documented lawful state after interruption.

Items 1 through 6 describe the multi-milestone destination. M1 establishes the
canonical identity boundary. M2 now provides deterministic chunk detection,
canonical layouts, capacity-bounded ingestion, and authenticated whole-blob
reconstruction and minimal exact range reads through the non-durable reference
CAS. M3 now provides exact immutable-segment construction and verified
admission. Catalog publication, durable retention and recovery, nearby-version
workflows, complete namespace verification, and restart recovery remain future
milestones.

See the [M1 conformance contract](docs/conformance/golden-file-worldline.md),
the [CDC profile corpus](conformance/cdc-profile/v1/README.md), the
[chunk identity invariant](docs/invariants/chunk-identity/README.md), the
[Flat Chunk Layout v1 specification](docs/formats/flat-chunk-layout-v1/README.md),
the [layout corpus](conformance/layout/v1/README.md), and the
[reference CAS contract](docs/architecture/reference-store/README.md) for the
implemented proof boundaries and explicit nonclaims. The
[Durable Segment Store v1 specification](docs/formats/segment-store-v1/README.md)
and [segment-store corpus](conformance/segment-store/v1/README.md) define the
implemented segment boundary and the still-planned publication and recovery
work. The
[streaming CAS baseline protocol](docs/benchmarks/streaming-cas-baseline-v1/README.md)
defines reproducible performance evidence without treating measurements as
correctness proof or weakening verification.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Every change must preserve Keep's core
law and satisfy the repository's formatting, linting, testing, and review
standards.

## Security

Please report vulnerabilities using the process in [SECURITY.md](SECURITY.md).
Do not include plaintext content, keys, or other sensitive material in a public
issue.

## License

Licensed under the [Apache License 2.0](LICENSE).
