# Dependency Admission: allocation-counter 0.8.1

- Status: Accepted for allocation regression tests only
- Date: 2026-07-25
- Owner: Keep conformance testing
- Upstream:
  [fornwall/allocation-counter](https://github.com/fornwall/allocation-counter)

## Admitted use

Keep admits the exactly pinned `allocation-counter` 0.8.1 package only as a
development dependency. One isolated integration-test binary uses
`allocation_counter::measure` to observe total, live, and peak heap allocation
while `FastCdc` processes caller-owned input.

The dependency is absent from Keep's published library graph, public API,
identity preimages, durable formats, and production behavior.

## Why it is required

`size_of::<FastCdc>()` measures inline layout but cannot detect dependency-owned
or future hidden heap allocation. Issue #8 requires measured evidence that peak
working memory is independent of blob length. The standard library exposes no
stable, safe API for measuring thread-local heap allocation.

The test prepares its input before the measured region and uses an
allocation-free callback. Zero observed allocations therefore bounds the
detector-owned heap contribution independently of caller input and output.

## Safety and dependency boundary

The package installs a process-global allocator wrapper in the test binary. Its
implementation contains unsafe `GlobalAlloc` delegation to `System`; Keep-owned
code calls only its safe `measure` API. The wrapper is linked only into the
dedicated integration test that imports it.

Version 0.8.1 declares `MIT/Apache-2.0`, has no transitive dependencies, and
introduces no default features. Keep selects its Apache-2.0 license option.

## Maintenance and exit

The package declares no Rust-version metadata. Keep's pinned MSRV, Clippy,
debug, release, policy, and advisory gates provide point-in-time compatibility
evidence.

Keep can remove the dependency without changing production code by replacing
the allocation test with another isolated measurement that preserves total,
live, and peak-allocation evidence.

Reopen this admission if its version, graph, allocator implementation, license,
or test-only boundary changes.
