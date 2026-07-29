# Dependency Admission: signal-hook 0.4.4

- Status: Accepted for repository-task terminal-signal handling only
- Date: 2026-07-28
- Owner: Keep repository verification
- Upstream:
  [rust-cli/signal-hook](https://github.com/rust-cli/signal-hook)

## Admitted use

Keep admits the exactly pinned `signal-hook` 0.4.4 package only behind the
`xtask` crate's `repository-tasks` feature. The private bounded-process adapter
uses its iterator API to observe `SIGINT`, `SIGTERM`, `SIGHUP`, and `SIGQUIT`
on a dedicated thread while external repository tools are active.

The adapter records the first signal as a typed interruption. Existing
process-group cleanup then terminates and reaps the child and its descendants.
When no repository tool is active, the adapter invokes the operating system's
default signal behavior. The
[terminal signal process-group decision](../adr/0007-terminal-signal-process-group-guard.md)
records the process-wide concurrency and cleanup policy.

The package is absent from Keep's published library graph, public API, content
identities, durable formats, and production storage behavior. No
dependency-owned type crosses out of the private repository-task adapter.

## Why the standard library is insufficient

The standard library exposes Unix signal constants through platform APIs but
does not provide a safe registration and delivery mechanism for process
signals. Implementing one locally would require unsafe signal-handler code,
async-signal-safety analysis, self-pipe or equivalent wakeup machinery, handler
composition, and platform-specific restoration behavior.

The admitted package owns that unsafe operating-system boundary. Keep-owned
code consumes signals only through its safe iterator and default-handler APIs.

## Features and resolved graph

The direct dependency disables default features and enables only `iterator`.
That feature also enables `channel`. Both are activated solely by
`repository-tasks`.

The locked graph introduced for this boundary is:

- `signal-hook` 0.4.4;
- `signal-hook-registry` 1.4.8;
- `errno` 0.3.14; and
- `libc` 0.2.186.

The latter two packages were already present in the locked workspace graph.

## Safety, licensing, and compatibility

`signal-hook` 0.4.4 and `signal-hook-registry` 1.4.8 declare
`MIT OR Apache-2.0`. Their declared Rust-version floors are 1.66 and 1.26,
respectively. Compatibility remains established by Keep's pinned stable, MSRV,
debug, release, Clippy, dependency-policy, and advisory lanes.

The packages contain unsafe code around operating-system signal registration
and delivery. Keep-owned code invokes only safe APIs, performs no work in a
signal handler, retains no dependency-owned public type, and confines global
registration to the private repository-task adapter. `cargo deny` and RustSec
checks remain mandatory point-in-time evidence.

## Failure and recovery boundaries

Registration, iterator creation, or signal-thread creation failure is a typed
refusal before the external command starts. Registry poisoning also refuses
the task. Once a signal is observed, process-group cleanup remains authoritative
and any cleanup failure is preserved alongside the interruption.

No repository, durable, or authoritative application state is written by this
boundary. Keep can remove the dependency without changing public or durable
behavior by replacing it with an equally portable safe mechanism that
preserves default behavior outside active tasks, typed interruption, bounded
wakeup, concurrent-operation handling, and whole-process-group cleanup.

Reopen this admission if the direct version, selected features, resolved graph,
license, supported signals, process-global handler policy, or
repository-task-only boundary changes.
