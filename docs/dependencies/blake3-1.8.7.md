# Dependency Admission: BLAKE3 1.8.7

- Status: Accepted for version-1 identities and their independent corpus oracle
- Date: 2026-10-03
- Owner: Keep identity layer
- Governing decision:
  [ADR-0001](../adr/0001-exact-logical-byte-identity.md) and the
  [chunk identity rationale](../invariants/chunk-identity/rationale.md)
- Upstream: [BLAKE3 official Rust implementation][blake3]

## Admitted use

Keep admits the `blake3` crate at locked version 1.8.7 for the 32-byte `BlobId` and `ChunkId` version-1 digests and the existing domain-separated segment, catalog, layout, retention and migration hashes. No dependency-owned type appears in Keep's public API.

The private `xtask` crate also uses the same pinned implementation to recompute Golden File Worldline digest witnesses directly from corpus source bytes and to name deterministic fuzz seeds. The oracle does not import Keep production types or hashing wrappers. This separation independently checks preimage framing, length encoding, canonical text and binary encodings, mutation semantics, and committed witness bytes. For every identity and content mutation, the repository checker also streams the canonical preimage through external `b3sum`; a mismatch with the in-process result is a refusal. Golden File Worldline and protocol conformance share one deadline-bounded process adapter, but retain separate preimage construction. The [Golden File Worldline reference model](../conformance/golden-file-worldline.md#reference-model) records the process-adapter contract. The checked-in vectors and runtime cross-check therefore cover the algorithm boundary without claiming that the Rust path independently implements the BLAKE3 compression function.

The manifest disables default features and enables exactly:

- `std`, because Keep version 1 is a standard-library crate and admits the
  upstream standard-library integration as its supported host posture;
- `pure`, which forces upstream's pure-Rust build path instead of its
  handwritten assembly or C implementations.

Production streaming paths use `Hasher::new`, `Hasher::update`, and `Hasher::finalize`; the segment digest builder clones bounded Hasher state before appending its seal suffix. The migration receipt's initial-state digests additionally use the one-shot `blake3::hash` over fixed domains in `migration_receipt_initial_state.rs`; repository tasks also use one-shot hashing for deterministic seed naming and reviewed Node installer/lock byte admission. The `std` feature is not part of content identity and may be removed in a dedicated dependency-policy change if Keep adopts a `no_std` lower layer. The `pure` feature is also not part of identity. Any future change to either feature must reproduce every identity vector exactly.

`pure` does not mean “free of unsafe code.” It selects Rust implementations, including platform intrinsics and dispatch that contain upstream-audited unsafe blocks. The upstream Rust-2024 build script also contains a small unsafe environment update. The Keep identity crate remains `unsafe_code = "forbid"`.

## Upgrade boundary

Compared with 1.8.5, the selected implementation replaces arrayref macros with bounded slices, checked array conversions and disjoint mutable splits. Review must verify each length and partition; dependency checksums alone do not establish output equivalence. The 1.8.6 mmap seek/fallback changes are outside Keep's selected features and call paths. No mmap, rayon, C or assembly path is newly admitted.

The separate external b3sum executable remains pinned at 1.8.5; updating the in-process dependency does not re-baseline that oracle or the committed identity vectors. Existing historical benchmark, crash and conformance receipts retain their original build coordinates. New validation must identify the candidate it actually executed; no performance improvement or universal platform equivalence is claimed.

## Why this dependency is needed

ADR-0001 and the chunk identity rationale make BLAKE3-256 part of Keep's permanent version-1 identity contracts. Keep therefore needs an implementation that supports:

- exact incremental hashing without content-sized allocation;
- stable, independently specified output;
- broad target support;
- high single-threaded software throughput;
- an implementation maintained alongside the BLAKE3 specification.

Rust's standard library does not provide BLAKE3 or another cryptographic hash. Writing a local implementation would create a cryptographic maintenance and portability burden far beyond 50 auditable lines. It would not reduce the need for independent vectors, differential testing, or platform review.

## Safety and build posture

Keep does not treat dependency code as covered by its own unsafe-code ban. Instead, this admission records the boundary explicitly:

- Keep passes exact byte slices to an owned `blake3::Hasher`;
- Keep never passes raw pointers, aliases, or caller-owned mutable state across
  the dependency boundary;
- `pure` excludes upstream C and handwritten assembly implementations;
- upstream Rust SIMD intrinsics and runtime dispatch may execute unsafe code;
- the build script and its `cc` dependency remain in the resolved build graph
  even when `pure` prevents C or assembly objects from being selected;
- independent `b3sum` vectors and runtime digest cross-checks detect an output
  change at the protocol boundary.

This posture accepts upstream's unsafe implementation boundary. Re-enabling C, assembly, AVX-512, NEON, or WASM SIMD requires a dedicated change with target coverage, differential identity tests, and benchmark evidence. Performance alone cannot waive the exact-output tests.

## MSRV and maintenance

The `blake3` 1.8.7 package does not declare Cargo `rust-version` metadata. Keep therefore does not infer an upstream MSRV contract. Keep's pinned Rust 1.96.0 CI compiles the complete selected graph in all-feature and minimal-feature jobs; that executable gate is the compatibility evidence.

The upstream [1.8.7 release][blake3-release] was published on 2026-08-20 and removes the arrayref dependency after an upstream report of its crates.io owner being compromised. This is the upstream rationale, not evidence that Keep's previously locked arrayref 0.3.9 bytes were compromised. Dependency updates remain isolated changes and must repeat the policy, identity, MSRV, audit, and benchmark gates.

## Features and transitive graph

The admitted normal dependency graph for supported targets is:

- `blake3` 1.8.7 with `std` and `pure`;
- `arrayvec` 0.7.8;
- `cfg-if` 1.0.4;
- `cpufeatures` 0.3.0 on x86 and x86_64, as selected by upstream's target dependency;
- `constant_time_eq` 0.4.2 with `std`.

On x86/x86_64, cpufeatures supplies runtime SSE2/SSE4.1/AVX2 detection even with `pure`. Its published manifest declares Rust 1.85 and an MIT OR Apache-2.0 license choice, within Keep's Rust 1.96.0 and Apache-2.0 policy.

The target-independent lockfile also records libc 0.2.186 in cpufeatures' dependency metadata for certain non-x86 targets. BLAKE3 selects cpufeatures only on x86/x86_64, where that libc edge is inactive. Other workspace dependencies can independently activate libc; lockfile presence alone does not establish an active BLAKE3 path.

The admitted build dependency graph is:

- `cc` 1.3.0;
- `find-msvc-tools` 0.1.9;
- `shlex` 2.0.1.

The lockfile is authoritative if resolution moves. A resolution change that adds or moves any dependency requires renewed review; this document is not a wildcard approval for compatible-version upgrades.

## License posture

The `blake3` crate offers Apache-2.0 as one of its license choices. The selected normal and build graph offers an Apache-compatible choice. Version 1.8.7 removes arrayref, so its crate-specific BSD-2-Clause exception is removed from Keep's dependency policy.

Dependency-policy and advisory checks must remain green for the locked graph. An audit result supports known-advisory posture; it does not prove the absence of implementation defects.

## Public API and compatibility

Keep exposes `BlobHasher`, `BlobId`, `ChunkId`, `FastCdc`, and typed Keep errors. It does not expose `blake3::Hasher`, `blake3::Hash`, or a dependency error type. Consequently a compatible implementation can replace the crate without breaking the Rust API.

It cannot silently change the algorithm. Lawful exit paths are:

1. upgrade or replace the implementation while reproducing all BLAKE3-256
   vectors, canonical `BlobId` encodings, and `ChunkId` digests exactly;
2. maintain a small audited internal BLAKE3 implementation after independent
   differential and conformance evidence justifies the maintenance burden;
3. introduce a new explicit identity version or algorithm coordinate through a
   new ADR and compatibility plan.

Existing version-1 blob and chunk identities remain BLAKE3-256 identities forever. A migration may add another coordinate, but it may not reinterpret or rewrite version 1.

## Rejected alternatives

### Standard-library hashing

Rejected because the standard library provides no stable cryptographic content hash suitable for a durable identity protocol.

### A short local BLAKE3 implementation

Rejected because cryptographic parsing, compression, tree hashing, platform behavior, and optimization are not a safe local utility. Less code would not mean less risk.

### Upstream default implementation selection

Rejected for the first milestone because it may select handwritten assembly or C paths according to target and toolchain. `pure` narrows the implementation surface while Keep freezes identity and conformance evidence.

### A slower algorithm implemented locally

Rejected because algorithm choice is a durable compatibility decision, not an excuse to trade away established analysis and performance. A different algorithm would require its own identity coordinate and evidence.

## Review triggers

Reopen this admission when any of these changes:

- `blake3` version or enabled features;
- any resolved normal or build dependency;
- Keep's MSRV or supported target set;
- the upstream unsafe, C, assembly, or runtime-dispatch boundary;
- a dependency-owned type crosses Keep's public API;
- the independent oracle begins importing production identity code;
- an advisory, maintenance, or license fact changes;
- measured performance motivates enabling another implementation path.

[blake3]: https://github.com/BLAKE3-team/BLAKE3
[blake3-release]: https://github.com/BLAKE3-team/BLAKE3/releases/tag/1.8.7
