# Dependency Admission: allocation-counter 0.8.1

- Status: Accepted for allocation regression tests and benchmark evidence
- Date: 2026-07-25
- Owner: Keep conformance testing
- Upstream:
  [fornwall/allocation-counter](https://github.com/fornwall/allocation-counter)

## Admitted use

Keep admits the exactly pinned `allocation-counter` 0.8.1 package only in
test and non-published benchmark targets. Isolated integration-test and
benchmark binaries use `allocation_counter::measure` to observe total, live,
and peak heap allocation while Keep processes caller-owned input.

The dependency is absent from Keep's published library graph, public API,
identity preimages, durable formats, and production behavior. Benchmark
reports name `bytes_max` as incremental peak live heap; they do not substitute
that allocator-local evidence for whole-process resident memory.

## Why it is required

`size_of::<FastCdc>()` measures inline layout but cannot detect dependency-owned
or future hidden heap allocation. Issue #8 requires measured evidence that peak
working memory is independent of blob length. The standard library exposes no
stable, safe API for measuring thread-local heap allocation.

Tests and benchmarks prepare their deterministic inputs before the measured
region. Zero observed allocations in the CDC isolation test therefore bounds
the detector-owned heap contribution independently of caller input and output.
Integrated benchmark scenarios retain allocation evidence because storage
operations are expected to allocate.

## Safety and dependency boundary

The package installs a process-global allocator wrapper in each importing
binary. Its implementation contains unsafe `GlobalAlloc` delegation to
`System`; Keep-owned code calls only its safe `measure` API. The wrapper is
linked only into dedicated tests and the non-published benchmark executable.

Version 0.8.1 declares `MIT/Apache-2.0`, has no transitive dependencies, and
introduces no default features. Keep selects its Apache-2.0 license option.

## Maintenance and exit

The package declares no Rust-version metadata. Keep's pinned MSRV, Clippy,
debug, release, policy, and advisory gates provide point-in-time compatibility
evidence.

Keep can remove the dependency without changing production code by replacing
the allocation tests and benchmark observer with isolated measurement that
preserves total, live, and peak-allocation evidence.

Reopen this admission if its version, graph, allocator implementation, license,
or test-only boundary changes.
