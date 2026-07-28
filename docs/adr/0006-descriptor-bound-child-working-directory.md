# ADR-0006: Descriptor-Bound Child Working Directory

- Status: Accepted
- Date: 2026-07-28
- Owners: Keep repository verification
- Related issue: none — review remediation for pull request 61
- Depends on: ADR-0004

## Context

Documentation verification inventories Git paths and runs pinned validation
tools. Opening the repository as a capability protects file reads, but passing
its ambient pathname to a child creates another replacement window. An
attacker could move the admitted directory, substitute another repository
while the tools run, and restore the original before the final identity check.

The standard library accepts only a pathname for
`std::process::Command::current_dir`. On macOS, an open directory exposed
through `/dev/fd` cannot be used as that pathname. Changing the parent process
directory through the safe Rustix `fchdir` wrapper was measured and rejected:
parallel xtask tests observed the temporary global state and failed.

The remaining standard-library hook is
`std::os::unix::process::CommandExt::pre_exec`, which is unsafe because code
between fork and exec must obey strict rules.

## Decision

Keep isolates the hook in the private `repository-process-spawn` workspace
crate. The crate admits one operation:

1. Own a close-on-exec duplicate of the admitted repository directory.
2. Register a child setup hook that calls only Rustix `fchdir`.
3. Spawn through the existing bounded process adapters.

POSIX specifies `fchdir` as async-signal-safe. The hook performs no allocation,
locking, buffered I/O, ambient path lookup, or user callback. The descriptor
closes on successful exec. A setup failure is returned by `Command::spawn`.

The workspace denies unsafe code by default. Only the dedicated crate carries
an explained `unsafe_code` allowance. It contains no storage, identity, format,
network, or application policy.

## Alternatives considered

- Rechecking the repository pathname before and after tool execution does not
  detect a transient substitution.
- `/dev/fd/<n>` and `/proc/self/fd/<n>` are not a portable child working
  directory. The former is not traversable as a directory on macOS, and the
  latter is not available there.
- Changing the parent directory under a mutex still mutates process-global
  state visible to threads outside that mutex.
- Copying the repository into a temporary tree changes Git, ignore, link, and
  untracked-file semantics and creates an unbounded materialization.
- Reimplementing subprocess management would duplicate the standard library's
  file-descriptor, environment, signal, and error handling.

## Consequences

Git inventory and documentation tools start in the exact opened repository
even if its pathname is replaced. Parent process state remains unchanged, so
parallel tests and readers are deterministic.

The boundary is Unix-specific and deliberately narrow. Any additional unsafe
operation, child hook, captured state, or consumer requires a new decision and
new executable evidence. The crate's regression test replaces the ambient
path, proves the child reads the retained directory, and proves the parent
working directory is unchanged.

No durable or authoritative state is written. Failure refuses the repository
task, so no recovery protocol is required.
