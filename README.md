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

Keep has completed the implementation portion of its first identity milestone.
The crate exposes a strict, versioned `BlobId` that can be calculated in one
pass from exact bytes or a blocking stream, and parsed from canonical text or
binary form. The language-neutral Golden File Worldline corpus independently
checks those identity rules.

Keep does **not** expose a storage API yet. Chunking, physical storage,
durability, retention, recovery, and garbage collection remain planned work.
Calculating or parsing a `BlobId` does not claim that Keep possesses, retained,
or verified the named bytes.

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

Items 1 through 6 describe the multi-milestone destination, not current
storage behavior. See the
[M1 conformance contract](docs/conformance/golden-file-worldline.md) for the
implemented proof boundary and explicit nonclaims.

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
