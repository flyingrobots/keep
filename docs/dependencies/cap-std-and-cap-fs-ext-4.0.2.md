# Dependency Admission: cap-std, cap-fs-ext 4.0.2, and rustix 1.1.4

- Status: Accepted for repository-task filesystem boundaries only
- Date: 2026-07-26
- Owner: Keep repository verification
- Upstream:
  [bytecodealliance/cap-std](https://github.com/bytecodealliance/cap-std)

## Admitted use

Keep admits the exactly pinned `cap-std` 4.0.2, `cap-fs-ext` 4.0.2, and
`rustix` 1.1.4 packages only behind the `xtask` crate's `repository-tasks`
feature.

`cap-std::fs::Dir` pins the admitted repository or corpus directory and opens
entries relative to that capability. `cap-fs-ext` supplies no-follow and
nonblocking open options plus cross-platform device and file identifiers.
After Git inventory, the source check compares the retained repository
directory's identity with a fresh open of the configured pathname. Together,
these operations let repository checks refuse persistent root replacement,
path substitution, symlinked protocol tables, FIFOs, sockets, devices, and
other ambiguous filesystem state before reading source or protocol bytes.

These packages are absent from Keep's published library graph, public API,
content identities, durable formats, and production behavior. No
dependency-owned type crosses out of the private repository-task adapter.

The bounded subprocess adapter uses Rustix's safe process API to send
`SIGKILL` to a dedicated child process group after a subprocess deadline or
collection failure. This prevents descendants that inherited an output pipe
from surviving the failed repository task.

## Why the standard library is insufficient

Checking a path and then reopening it with `std::fs` leaves a
time-of-check/time-of-use window. Canonicalizing a pathname does not retain the
checked directory or file and therefore cannot prove that later bytes came from
the admitted object.

The standard library also has no portable API that combines
capability-relative opens with no-follow and nonblocking semantics. Recreating
that boundary locally would require operating-system-specific flags, handle
conversion, and path-resolution code. That would exceed a small local helper,
duplicate security-sensitive upstream work, and require unsafe code that Keep
otherwise forbids.

## Features and resolved graph

All three direct dependencies disable default features. Keep enables only
`cap-fs-ext`'s `std` feature and Rustix's `process` and `std` features;
`cap-std` has no enabled feature. All declarations are optional and are
activated solely by `repository-tasks`.

The locked non-Windows graph introduced for this boundary is:

- `ambient-authority` 0.0.2;
- `bitflags` 2.13.1;
- `cap-primitives` 4.0.2;
- `errno` 0.3.14;
- `fs-set-times` 0.20.3;
- `io-extras` 0.19.0;
- `io-lifetimes` 2.0.4 and 3.0.1;
- `ipnet` 2.12.0;
- `libc` 0.2.186;
- `linux-raw-sys` 0.12.1;
- `maybe-owned` 0.3.4;
- `once_cell` 1.21.4;
- `rustix` 1.1.4; and
- `rustix-linux-procfs` 0.1.1.

Windows resolution additionally retains the locked `windows-sys`,
`windows-targets`, and architecture packages recorded in `Cargo.lock`.

## Safety, licensing, and compatibility

The capability packages declare
`Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`; Keep selects an admitted
license through repository policy. Rustix declares `Apache-2.0 OR MIT`.
Their manifests declare no Rust-version floor. Compatibility is therefore
established only by Keep's pinned stable, MSRV, debug, release, Clippy,
dependency-policy, and advisory lanes.

The admitted packages and their platform dependencies may contain unsafe code
around operating-system calls and handles. Keep-owned code invokes only their
safe APIs, retains handles and process identifiers in private adapter types,
checks resulting metadata, bounds reads, and never treats a dependency as
proof of content identity. `cargo deny` and RustSec checks remain mandatory
point-in-time evidence; they do not transfer Keep's unsafe-code guarantee to
dependencies.

## Failure and recovery boundaries

An open, metadata, or read failure is a typed refusal. The task never repairs,
rewrites, or substitutes repository data. Retained handles exist only for one
verification process and carry no durability or recovery semantics.

Keep can remove these dependencies without changing public or durable behavior
by replacing them with an equally portable, safe implementation that preserves
capability-relative, no-follow, nonblocking, regular-file, retained-handle, and
whole-process-group cleanup tests on every supported platform.

Reopen this admission if either direct version, selected feature, resolved
graph, license, supported platform, handle-retention invariant, or
repository-task-only boundary changes.
