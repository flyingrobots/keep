# ADR-0007: Terminal Signal Process-Group Guard

- Status: Accepted
- Date: 2026-07-28
- Owners: Keep repository verification
- Related issue: none — review remediation for pull request 61
- Depends on: ADR-0006

## Context

Repository tasks run external tools in dedicated process groups. A deadline or
collection failure terminates the whole group, but the operating system's
default terminal-signal action terminates the `xtask` parent immediately.
Signals such as `SIGINT` therefore bypass Rust cleanup and can leave an
isolated child or descendant running.

Captured output creates a second wait boundary. A child leader can exit while
a descendant retains an inherited pipe, so guarding only the child wait does
not cover the complete operation.

## Decision

While an external repository task is active, `xtask` installs one process-wide
guard for `SIGINT`, `SIGTERM`, `SIGHUP`, and `SIGQUIT`. A dedicated signal
thread records the first signal for every active operation. Child waits and
captured-output readers poll that state at the existing ten-millisecond process
interval.

An observed terminal signal becomes a typed `ProcessError::Interrupted`
refusal. The normal failure path then sends `SIGKILL` to the dedicated child
process group, kills and reaps the child, and joins captured-output readers
before returning. The terminal signal is not sent directly to the child group;
one cleanup authority avoids races between signal delivery and mandatory
process-group termination.

When no external operation is active, the guard restores the signal's default
behavior. A signal observed while the final operation retires, or a second
signal received during cleanup, also restores default termination instead of
being swallowed. Handler registration is initialized once because removing a
`signal-hook` action does not restore a previous operating-system handler. The
[signal-hook dependency admission](../dependencies/signal-hook-0.4.4.md)
records the selected package boundary.

## Alternatives considered

- Relying on the default terminal action abandons Rust cleanup and can strand
  descendants.
- Forwarding the original terminal signal directly to the child group races
  mandatory cleanup. On macOS, a descendant can exit from `SIGINT` before the
  subsequent group kill, which makes the second operation report `EPERM`
  despite successful termination.
- A parent-death signal is not portable across the supported Unix platforms
  and does not cover descendants that change their parent relationship.
- Polling only the child status misses descendants that retain captured output
  pipes after the child leader exits.

## Consequences

Terminal interruption follows the same typed cleanup and reaping protocol as
timeouts and collection failures. Unbounded child waits and output collection
now wake at most one process-poll interval after the signal thread records an
interrupt.

Signal registration is process-global, but active state is operation-local and
supports concurrent repository tasks. No lock is held while a child is
spawned, polled, killed, reaped, or read.

The guard is private to `xtask`. It changes no Keep library API, content
identity, durable format, or recovery protocol. Regression evidence sends
`SIGINT` to a supervisor with an isolated descendant, requires the exact typed
refusal, and proves the descendant is no longer reachable.
