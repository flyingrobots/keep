# Dependency Admission: cpu-time 1.0.0

- Status: Accepted for non-published benchmark evidence only
- Date: 2026-07-27
- Owner: Keep performance conformance
- Upstream: [tailhook/cpu-time](https://github.com/tailhook/cpu-time)

## Admitted use

Keep admits the exactly pinned `cpu-time` 1.0.0 package only in the
non-published `keep-benchmark` workspace member. The benchmark calls the
fallible, safe `ProcessTime::try_now` and `ProcessTime::try_elapsed` APIs to
record process CPU duration around each authenticated workload sample.

The dependency is absent from Keep's published library graph, public API,
identity preimages, durable formats, and production behavior. CPU duration is
environment-sensitive benchmark evidence, never correctness evidence.

## Why it is required

Rust's stable standard library exposes monotonic wall time but no portable
process CPU clock. Wall time cannot distinguish scheduler delay from time spent
executing the benchmark. Shell timing utilities would include process startup
and require platform-specific diagnostic parsing outside the timed workload.

The package provides one small safe abstraction over the operating-system
process CPU clock. Keep refuses the measurement if that clock is unavailable;
it does not silently substitute wall time.

## Safety and dependency boundary

The package uses `libc::clock_gettime` on Unix and process-time APIs on Windows
behind its safe, fallible interface. Keep-owned code contains no unsafe block
and does not expose dependency-owned clock types.

Version 1.0.0 declares `MIT/Apache-2.0`; Keep selects Apache-2.0. Its platform
dependencies are locked by `Cargo.lock`. The benchmark remains synchronous and
single-threaded, so process CPU time has an explicit interpretation.

## Maintenance and exit

The package declares no Rust-version metadata. Keep's pinned MSRV, Clippy,
debug, release, policy, and advisory gates provide point-in-time compatibility
evidence.

Keep can remove this dependency without changing production code if stable Rust
gains a fallible process CPU clock or the benchmark adopts an equally bounded,
portable observer. Reopen this admission if the version, graph, supported
platforms, clock semantics, implementation safety, or license changes.
