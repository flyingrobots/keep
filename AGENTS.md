AGENTS.md — Keep

Keep is foundational storage infrastructure. Optimize for correctness, recoverability, auditability, and maintainability before performance or convenience.

Core Law

For a given content identity, Keep must return exactly the bytes named by that identity—or refuse.

Never silently repair, approximate, substitute, weaken verification, or continue from ambiguous state.

Rust Standards

* Use stable, pinned Rust with edition 2024.
* cargo fmt --check and Clippy with -D warnings must pass.
* Deny unwrap, expect, panic!, todo!, unimplemented!, dbg!, stdout/stderr printing, unchecked indexing, lossy casts, and unsafe code.
* unsafe is forbidden unless isolated in a dedicated crate with documented invariants, tests, and measured necessity.
* All externally influenced arithmetic must use checked operations.
* Use TryFrom/try_from; do not use as for potentially lossy conversions.
* Public functions must not take boolean parameters. Use enums.
* Prefer typed newtypes over primitive IDs, lengths, offsets, generations, and namespaces.
* Parse, validate, then admit. Do not deserialize untrusted bytes directly into trusted types.
* Make illegal states unrepresentable where practical.
* Prefer concrete types. Add traits only for real substitution boundaries.
* Prefer synchronous core APIs. Do not introduce async without a demonstrated consumer need.
* Do not use HashMap iteration where order can affect identity, serialization, tests, or behavior.

Hexagonal Architecture and Determinism

* Use hexagonal architecture. The domain core owns invariants and policy; ports
  name semantic capabilities; adapters own technologies and protocols.
* Dependencies point inward. Core and port modules must not import adapters,
  filesystems, networks, CLIs, runtimes, or application policy.
* Codecs exist only at ingress and egress boundaries. Ports exchange semantic
  or validated types, never serializer-owned JSON, CBOR, or wire values.
* Boundary adapters parse, validate, canonicalize where the protocol permits
  it, then admit validated domain types. They invert that translation on egress.
* Determinism is a correctness property. Identity, persistence, comparison,
  tests, and protocol behavior must not depend on iteration order, host state,
  clocks, locale, or serializer defaults.
* JSON and CBOR that cross a boundary, persist, or affect identity must use a
  named canonical profile with golden fixtures. Reject duplicate fields and
  noncanonical identity-bearing encodings.
* Never hash arbitrary serializer output. Hash only typed, domain-separated
  canonical bytes.

Structure and Findability

* Target file size: 200 lines.
* Review threshold: 300 lines.
* Hard maximum: 500 lines.
* Target function size: 20 logical lines.
* Review threshold: 40 lines.
* Hard maximum: 60 lines.
* Maximum nesting depth: 3.
* Maximum function parameters: 5.
* Maximum boolean parameters: 0.
* Split by semantic ownership, not arbitrary file size.
* Every module must clearly answer: “This module owns…”
* Do not create utils.rs, helpers.rs, common.rs, misc.rs, shared.rs, manager.rs, service.rs, types.rs, or models.rs.
* Name files after the concept they own: segment_header.rs, root_generation.rs, range_plan.rs.
* Public concepts should be locatable by filename search within two attempts.
* Keep lower layers independent of Echo, Git, Graft, WARP, CLI, and application policy.

API and Type Design

* Everything is private by default.
* Use pub(crate) unless external consumers require more.
* Public validated types must have private fields and checked constructors.
* Use #[must_use] for staged work, plans, commits, and consequential results.
* Option means normal absence, not unexplained failure.
* Errors must be typed by boundary: ingestion, decoding, validation, reading, retention, recovery, verification, GC.
* Preserve error sources. Do not stringify errors early.
* Error variants must include expected and observed state where useful.
* Never expose physical storage locations as stable public identity.
* Keep logical identity separate from layout, representation, and physical location.

Storage and Durability

* Treat filesystem operations as adversarial and fallible.
* Never infer completeness, identity, retention, or durability from file existence alone.
* Use explicit commit methods. Do not rely on Drop for durability.
* Sealed segments are immutable.
* Publication order and sync behavior must be documented and crash-tested.
* Recovery must be designed alongside every write protocol.
* Every write transition must define:
    1. forward protocol;
    2. possible crash states;
    3. recovery behavior.
* Use one-writer/many-reader semantics unless concurrent writers are deliberately designed.
* Channels must be bounded.
* Do not hold locks across external I/O, callbacks, long computation, or other locks without documented ordering.

Formats

* On-disk formats are protocols, not private implementation details.
* Every format needs magic bytes, versioning, canonical encoding, explicit endianness, bounds, checksums, and golden fixtures.
* Do not define durable formats as arbitrary Serde or Rust struct output.
* Reject trailing bytes, duplicate fields, noncanonical encodings, overflow, invalid ordering, excessive depth, and unknown mandatory flags.
* Hash typed, domain-separated canonical preimages.
* Round-trip tests alone are insufficient.

Testing

Every meaningful change must include tests appropriate to its failure modes.

Required test classes:

* unit tests;
* public API integration tests;
* golden-format tests;
* property tests;
* model-based tests;
* corruption tests;
* crash-injection tests;
* recovery tests;
* concurrency tests;
* fuzz tests for all parsers and decoders;
* benchmark regression tests for performance-sensitive changes.

Testing rules:

* Test names describe laws, not functions.
* Do not use sleeps for synchronization.
* Do not depend on wall-clock time, filesystem ordering, global state, network access, locale, or test order.
* Assert exact typed failures, not merely is_err().
* Every discovered fuzz or corruption bug becomes a permanent regression test.
* Test debug and release builds.
* Coverage is evidence, not proof. Do not add meaningless tests to satisfy metrics.

Performance

* Measure before optimizing.
* Preserve bounded memory and streaming behavior.
* No hidden whole-blob allocation.
* APIs that materialize full content must say so explicitly.
* Track throughput, latency, peak memory, allocations, bytes read/written, write amplification, read amplification, sync count, and deduplication ratio.
* Performance improvements must not weaken verification, durability, recovery, or diagnostics.
* No clone, allocation, Arc, mmap, unsafe, async, compression, or caching optimization without evidence that it solves a measured problem.

Dependencies

* Dependencies require justification.
* Disable default features unless explicitly needed.
* Do not expose dependency-owned types in public APIs casually.
* Commit the lockfile.
* Run dependency audit and policy checks.
* Features must be additive and must not alter identity, canonical encoding, verification, or durability semantics.

Documentation

* Document all public items.
* Explain invariants, errors, allocation, blocking, I/O, verification, complexity, and durability implications.
* Comments explain why, ordering, invariants, and recovery—not syntax.
* Important public examples must compile as doctests.
* Decisions affecting identity, format, durability, recovery, concurrency, GC, encryption, or public compatibility require an ADR.

Pull Requests

Prefer small, single-purpose PRs.

Each PR must state:

* problem;
* invariant affected;
* approach;
* alternatives rejected;
* failure modes;
* tests;
* benchmark impact;
* format/API compatibility;
* recovery implications;
* security implications.

Do not mix semantic changes with unrelated refactoring.

Review Questions

Before approving, ask:

* What invariant does this code establish?
* What malformed state can reach it?
* What happens after process death here?
* What survives power loss?
* Is arithmetic checked?
* Is allocation bounded?
* Is ordering deterministic?
* Does dependency flow point inward through explicit ports?
* Is every codec confined to an ingress or egress adapter?
* Are identity-bearing JSON and CBOR bytes canonical under a named profile?
* Can invalid state be represented?
* Is the error precise?
* Is identity stable across migration and compaction?
* Does recovery agree with the write protocol?
* Is the abstraction simpler than the code it replaced?

Final Standard

Keep should be boring, explicit, searchable, bounded, deterministic, and difficult to misuse.

If code is clever but harder to audit, reject it.

If code is faster but weakens truthfulness, reject it.

If a state cannot be proved safe, represent it as uncertainty and refuse.
