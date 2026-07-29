# Dependency Admission: cap-std, cap-fs-ext 4.0.2, and rustix 1.1.4

- Status: Accepted for repository-task and segment-store filesystem boundaries
- Date: 2026-07-26
- Owner: Keep repository verification
- Upstream:
  [bytecodealliance/cap-std](https://github.com/bytecodealliance/cap-std)

## Admitted use

Keep admits the exactly pinned `cap-std` 4.0.2 and `cap-fs-ext` 4.0.2 packages
for the library's segment-store filesystem adapter and behind the `xtask`
crate's `repository-tasks` feature. The library also admits Rustix 1.1.4 for
safe Linux filesystem-profile inspection and no-symlink root opening; `xtask`
uses the same exact version behind `repository-tasks`.

`cap-std::fs::Dir` pins the admitted repository or corpus directory and opens
entries relative to that capability. `cap-fs-ext` supplies no-follow and
nonblocking open options plus cross-platform device and file identifiers.
After Git inventory, the source check compares the retained repository
directory's identity with a fresh open of the configured pathname. Together,
these operations let repository checks refuse persistent root replacement,
path substitution, symlinked protocol tables, FIFOs, sockets, devices, and
other ambiguous filesystem state before reading source or protocol bytes.

The capability packages are present in Keep's published library graph and
production filesystem behavior. No dependency-owned type crosses Keep's public
API or enters content identities or durable formats. The segment-store writer
lock retains the root capability plus root-lock and writer-lock file handles
behind `FilesystemWriterLock`; its public acquisition boundary accepts only
`std::path::Path`. The production initializer uses Rustix's safe `openat2`,
`fstatfs`, `fstatvfs`, and ext4 inode flag APIs to admit only the documented
writable, non-casefolded Linux ext4 profile.

The bounded subprocess adapter uses Rustix's safe filesystem API to mark child
stdin nonblocking before deadline-bounded input transfer. It uses Rustix's safe
process API to send `SIGKILL` to a dedicated child process group after a
subprocess deadline or collection failure. This prevents a non-reading child
from blocking its parent indefinitely and prevents descendants that inherited
an output pipe from surviving the failed repository task.
The
[signal-hook dependency admission](signal-hook-0.4.4.md)
records the terminal-signal guard that routes interruption through the same
authoritative cleanup boundary.

Documentation verification also duplicates the admitted repository directory
handle and uses the isolated `repository-process-spawn` crate to start Git and
validation tools from that exact directory. Its child-only setup hook performs
one POSIX async-signal-safe `fchdir` after fork and before exec. Parent process
state never changes. A transient ambient-path replacement therefore cannot
redirect corpus inventory or tool execution.
The
[descriptor-bound child working-directory decision](../adr/0006-descriptor-bound-child-working-directory.md)
records the unsafe-boundary invariants and rejected alternatives.

## Why the standard library is insufficient

Checking a path and then reopening it with `std::fs` leaves a
time-of-check/time-of-use window. Canonicalizing a pathname does not retain the
checked directory or file and therefore cannot prove that later bytes came from
the admitted object.

The standard library also has no portable API that combines
capability-relative opens with no-follow and nonblocking semantics. Recreating
that boundary locally would require operating-system-specific flags, handle
conversion, descriptor-relative directory changes, and path-resolution code.
That would exceed a small local helper, duplicate security-sensitive upstream
work, and require unsafe code that Keep otherwise forbids.

## Features and resolved graph

All three direct dependencies disable default features. Keep enables only
`cap-fs-ext`'s `std` feature and Rustix's `fs` and `std` features in the
library; `cap-std` has no enabled feature. The library's filesystem
dependencies are unconditional because the production segment-store adapter
requires them. The `xtask` declarations remain optional and are activated
solely by `repository-tasks`; that feature additionally enables Rustix's
`process` feature.

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
The library therefore exempts only Clippy's `multiple_crate_versions` cargo
lint; exact direct versions, the committed lockfile, dependency policy, and
advisory checks remain authoritative.

## Safety, licensing, and compatibility

The capability packages declare
`Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`; Keep selects an admitted
license through repository policy. Rustix declares `Apache-2.0 OR MIT`.
The locked `winx` 0.36.4 transitive package declares only
`Apache-2.0 WITH LLVM-exception`, so `deny.toml` admits that exact package and
license combination rather than broadening the global license allowlist.
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

An open, metadata, platform-profile, read, writer-lock acquisition,
descriptor-duplication, descriptor-flag, child-directory setup, child-spawn,
stdin-write, output-collection, deadline, or cleanup failure is a typed refusal.
Repository tasks never repair, rewrite, or substitute repository data.
Repository-task handles exist only for one verification process and carry no
durability or recovery semantics. `FilesystemWriterLock` retains the pinned
store root and persistent lock-file handles for the complete writer-authority
lifetime; dropping the guard releases only the kernel lock and never mutates
the lock file.

Keep can remove these dependencies without changing public or durable behavior
by replacing them with an equally portable, safe implementation that preserves
capability-relative, no-follow, nonblocking, regular-file, retained-handle, and
whole-process-group cleanup tests on every supported platform.

Reopen this admission if either direct version, selected feature, resolved
graph, license, supported platform, handle-retention invariant, or
repository-task-only boundary changes.
