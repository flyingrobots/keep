# Dependency Admission: BLAKE3 1.8.5

- Status: Accepted for version-1 identities and their independent corpus oracle
- Date: 2026-07-19
- Owner: Keep identity layer
- Governing decision:
  [ADR-0001](../adr/0001-exact-logical-byte-identity.md) and the
  [chunk identity rationale](../invariants/chunk-identity/rationale.md)
- Upstream: [BLAKE3 official Rust implementation][blake3]

## Admitted use

Keep admits the `blake3` crate at locked version 1.8.5 to calculate the
32-byte digests in the `BlobId` and `ChunkId` version-1 preimages. Keep uses
the incremental `Hasher` API only. No dependency-owned type appears in Keep's
public API.

The private `xtask` crate also uses the same pinned implementation to recompute
Golden File Worldline digest witnesses directly from corpus source bytes and to
name deterministic fuzz seeds. The oracle does not import Keep production
types or hashing wrappers. This separation independently checks preimage
framing, length encoding, canonical text and binary encodings, mutation
semantics, and committed witness bytes. For every identity and content mutation,
the repository checker also streams the canonical preimage through external
`b3sum`; a mismatch with the in-process result is a refusal. Golden File
Worldline and protocol conformance share one deadline-bounded process adapter,
but retain separate preimage construction. The
[Golden File Worldline reference model](../conformance/golden-file-worldline.md#reference-model)
records the process-adapter contract. The checked-in vectors and runtime
cross-check therefore cover the algorithm boundary without claiming that the
Rust path independently implements the BLAKE3 compression function.

The manifest disables default features and enables exactly:

- `std`, because Keep version 1 is a standard-library crate and admits the
  upstream standard-library integration as its supported host posture;
- `pure`, which forces upstream's pure-Rust build path instead of its
  handwritten assembly or C implementations.

The production Keep call paths use only `Hasher::new`, `Hasher::update`, and
`Hasher::finalize`; repository tasks additionally use the one-shot `hash`
function for deterministic seed naming. The `std` feature is not part of
content identity and may be removed in a dedicated dependency-policy change if
Keep adopts a `no_std` lower layer. The `pure` feature is also not part of
identity. Any future change to either feature must reproduce every identity
vector exactly.

`pure` does not mean “free of unsafe code.” It selects Rust implementations,
including platform intrinsics and dispatch that contain upstream-audited unsafe
blocks. The upstream Rust-2024 build script also contains a small unsafe
environment update. Keep-owned crates remain `unsafe_code = "forbid"`.

## Why this dependency is needed

ADR-0001 and the chunk identity rationale make BLAKE3-256 part of Keep's
permanent version-1 identity contracts. Keep therefore needs an implementation
that supports:

- exact incremental hashing without content-sized allocation;
- stable, independently specified output;
- broad target support;
- high single-threaded software throughput;
- an implementation maintained alongside the BLAKE3 specification.

Rust's standard library does not provide BLAKE3 or another cryptographic hash.
Writing a local implementation would create a cryptographic maintenance and
portability burden far beyond 50 auditable lines. It would not reduce the need
for independent vectors, differential testing, or platform review.

## Safety and build posture

Keep does not treat dependency code as covered by its own unsafe-code ban.
Instead, this admission records the boundary explicitly:

- Keep passes exact byte slices to an owned `blake3::Hasher`;
- Keep never passes raw pointers, aliases, or caller-owned mutable state across
  the dependency boundary;
- `pure` excludes upstream C and handwritten assembly implementations;
- upstream Rust SIMD intrinsics and runtime dispatch may execute unsafe code;
- the build script and its `cc` dependency remain in the resolved build graph
  even when `pure` prevents C or assembly objects from being selected;
- independent `b3sum` vectors and runtime digest cross-checks detect an output
  change at the protocol boundary.

This posture accepts upstream's unsafe implementation boundary. Re-enabling C,
assembly, AVX-512, NEON, or WASM SIMD requires a dedicated change with target
coverage, differential identity tests, and benchmark evidence. Performance
alone cannot waive the exact-output tests.

## MSRV and maintenance

The `blake3` 1.8.5 package does not declare Cargo `rust-version` metadata.
Keep therefore does not infer an upstream MSRV contract. Keep's pinned Rust
1.96.0 CI compiles the complete selected graph in all-feature and
minimal-feature jobs; that executable gate is the compatibility evidence.

At admission, Keep classifies the upstream project as actively maintained. The
official repository was neither archived nor disabled, release
[1.8.5 was published on 2026-04-25][blake3-release], and the repository recorded
code activity on 2026-05-21. These are point-in-time signals, not a promise of
future maintenance. Dependency updates remain isolated changes and must repeat
the policy, identity, MSRV, audit, and benchmark gates.

## Features and transitive graph

The admitted normal dependency graph for supported targets is:

- `blake3` 1.8.5 with `std` and `pure`;
- `arrayref` 0.3.9;
- `arrayvec` 0.7.8;
- `cfg-if` 1.0.4;
- `constant_time_eq` 0.4.2 with `std`.

The admitted build dependency graph is:

- `cc` 1.3.0;
- `find-msvc-tools` 0.1.9;
- `shlex` 2.0.1.

The lockfile is authoritative if resolution moves. A resolution change that
adds or moves any dependency requires renewed review; this document is not a
wildcard approval for compatible-version upgrades.

## License posture

The `blake3` crate offers Apache-2.0 as one of its license choices. Most of the
selected graph also offers an Apache-compatible choice. `arrayref` is
BSD-2-Clause and therefore requires a crate-specific policy exception. That
exception does not admit BSD-2-Clause globally for unrelated future crates.

Dependency-policy and advisory checks must remain green for the locked graph.
An audit result supports known-advisory posture; it does not prove the absence
of implementation defects.

## Public API and compatibility

Keep exposes `BlobHasher`, `BlobId`, `ChunkId`, `FastCdc`, and typed Keep
errors. It does not expose `blake3::Hasher`, `blake3::Hash`, or a dependency
error type. Consequently a compatible implementation can replace the crate
without breaking the Rust API.

It cannot silently change the algorithm. Lawful exit paths are:

1. upgrade or replace the implementation while reproducing all BLAKE3-256
   vectors, canonical `BlobId` encodings, and `ChunkId` digests exactly;
2. maintain a small audited internal BLAKE3 implementation after independent
   differential and conformance evidence justifies the maintenance burden;
3. introduce a new explicit identity version or algorithm coordinate through a
   new ADR and compatibility plan.

Existing version-1 blob and chunk identities remain BLAKE3-256 identities
forever. A migration may add another coordinate, but it may not reinterpret or
rewrite version 1.

## Rejected alternatives

### Standard-library hashing

Rejected because the standard library provides no stable cryptographic content
hash suitable for a durable identity protocol.

### A short local BLAKE3 implementation

Rejected because cryptographic parsing, compression, tree hashing, platform
behavior, and optimization are not a safe local utility. Less code would not
mean less risk.

### Upstream default implementation selection

Rejected for the first milestone because it may select handwritten assembly or
C paths according to target and toolchain. `pure` narrows the implementation
surface while Keep freezes identity and conformance evidence.

### A slower algorithm implemented locally

Rejected because algorithm choice is a durable compatibility decision, not an
excuse to trade away established analysis and performance. A different
algorithm would require its own identity coordinate and evidence.

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
[blake3-release]: https://github.com/BLAKE3-team/BLAKE3/releases/tag/1.8.5
