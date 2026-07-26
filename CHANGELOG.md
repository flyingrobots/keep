# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

### Changed

- CI now runs every registered fuzz target with pinned `cargo-fuzz` and
  nightly versions, bounded per-target resources, and retained failure
  artifacts.
- Moved the canonical text and binary `BlobId` codecs out of the `blob`
  identity module into a new `adapters` boundary layer, per
  [ADR-0004](docs/adr/0004-hexagonal-boundary-architecture.md). `blob` now
  owns only identity calculation; encoding and decoding live at the
  boundary. No public API or format change.
- `BlobId`'s `Debug` output changed from `BlobId(<canonical text>)` to
  `BlobId { logical_length: ..., digest: [..] }` so core's `Debug` impl no
  longer depends on the adapter-owned `Display` impl. `Debug` output carries
  no stability contract; this is not a format change.

### Added

- Canonical version-1 `ChunkId` calculation in a domain distinct from
  `BlobId`, with independent golden vectors.
- A constant-memory `FastCdc` detector for `fastcdc-64k-v1` that preserves
  boundaries and chunk identities across arbitrary feed partitioning.
- Typed `ChunkLength`, `ChunkOffset`, and `ChunkSpan` values, corpus-driven
  property and adversarial tests, retained-memory evidence, and a streaming
  CDC fuzz target.
- Canonical version-1 `BlobId` calculation over exact logical bytes using a
  one-pass, length-committing BLAKE3-256 preimage.
- Strict, allocation-bounded text and fixed-width binary `BlobId` codecs with
  typed refusal for malformed and unsupported encodings.
- The implementation-independent Golden File Worldline v1 conformance corpus,
  independent vector checker, mutation cases, and bounded reference model.
- A versioned Gear64/FastCDC content-defined chunking profile, canonical
  `StorageProfileId`, and language-neutral golden boundary corpus.
- Initial repository foundation.
