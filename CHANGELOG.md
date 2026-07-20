# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

### Added

- Canonical version-1 `BlobId` calculation over exact logical bytes using a
  one-pass, length-committing BLAKE3-256 preimage.
- Strict, allocation-bounded text and fixed-width binary `BlobId` codecs with
  typed refusal for malformed and unsupported encodings.
- The implementation-independent Golden File Worldline v1 conformance corpus,
  independent vector checker, mutation cases, and bounded reference model.
- A versioned Gear64/FastCDC content-defined chunking profile, canonical
  `StorageProfileId`, and language-neutral golden boundary corpus.
- Initial repository foundation.
