# Preserve typed storage refusals

- Status: Accepted
- Date: 2026-10-26
- Owners: Keep filesystem, retention, migration, and recovery adapters
- Related issue: [#110](https://github.com/flyingrobots/keep/issues/110)
  (roadmap T-01.2); salvaged from #133 on the closed #107 branch

## Context

T-01.2 requires every refusal to be typed by its boundary, to carry expected
and observed evidence where the boundary has it, and to preserve its source.
Several filesystem adapters on `main` still constructed `io::Error` values from
static strings or formatted messages, and two retention current-state checks
discarded their cause: a selected-namespace open failure became
`CommittedNamespaceUnavailable`, and a predecessor-root decode failure became
`PredecessorRootChanged`. A caller could distinguish those failures only by
parsing diagnostic text, and the original OS error or decoder refusal was
lost.

The storage ports return `io::Error`. That signature is an operational
contract, not a reason to erase the semantic payload.

## Decision

Every adapter-owned refusal is a typed `Error` value carried as the payload of
the `io::Error` its port returns. Callers recover it with `io::Error::get_ref`
and `downcast_ref`, then follow `Error::source`. Message text is presentation,
never a classification API.

Existing vocabularies are extended before new ones are created:

- `RetentionRecordRefusal` gains the publication-attempt and recovery-context
  state refusals, and reuses `KindOrLength` and `LengthOverflow` for a
  non-regular or unboundable retained stage.
- `RetentionCurrentStateRefusal` gains `CommittedNamespaceRead`, carrying the
  selected namespace and the original open failure, and
  `PredecessorRootRefused`, carrying the exact `RetentionRootDecodeError`.
  `PredecessorRootChanged` now means only a decoded selection mismatch.
  `CommittedNamespaceUnavailable` remains for compatibility but is no longer
  constructed.

New types own refusals that no existing vocabulary fits:

- `FilesystemOperationRefusal` for writer-authority state, pinned-entry
  identity and kind, recovery-inventory bounds, crash-hook prefix bounds, and
  the Linux ext4 platform profile (with `statx`, filesystem, mount, and inode
  evidence);
- `FilesystemEntryIdentity` for device-and-inode evidence;
- `ReaderFenceRefusal` with one admission law shared by retention snapshots and
  store migration;
- `FilesystemMigrationRefusal` for forward-migration fixed-stage state, phase
  dispatch, and namespace admission.

These adapters deliberately do not reuse `FilesystemNamespaceRefusal`:
verification classifies that payload as structural corruption, so reusing it
would reclassify failures that `main` reports as operational.

Every outer `io::ErrorKind` is unchanged from the textual error it replaces.
In particular, `CommittedNamespaceRead` remains `InvalidData`; the original
kind and OS code stay on its source.

## Alternatives considered

Keeping strings preserves familiar text but forces callers to parse messages.
A generic message wrapper is still a string refusal. Adding variants to the
exhaustive `WriterLockAcquireError`, `RecoveryInventoryError`, or
`FilesystemCatalogPublicationError` enums would break exhaustive matches in
external code; payloads carried in their existing `io::Error` sources do not.
Changing the I/O port signatures to protocol-specific error types would force
a breaking change on every storage port and implementation. Mapping a missing
namespace to `InvalidData` but other open failures to their operational kind,
as the #107 branch did, was rejected here because it would change how a
substituted namespace (`ENOTDIR`, `ELOOP`) is reported.

Source-text tests that scan production code for string constructors were
rejected as evidence (issue #168). They show what the source contains, not
what a caller can recover at runtime.

## Consequences

The public API gains four types and additive variants on two
`#[non_exhaustive]` enums. A caller that matched `PredecessorRootChanged` for
corrupt predecessor bytes must also handle `PredecessorRootRefused`.
Verification classification is unchanged: both new current-state variants are
classified like the variants they refine.

Identity, canonical bytes, publication order, synchronization, durability,
and recovery decisions do not change. Refusal evidence carries coordinates,
lengths, flags, and protocol names, never content bytes or physical paths.
Device and inode numbers are host-local diagnostics, not stable identity.

Runtime downcast laws cover each wired adapter. On the parent commit, which
added the vocabulary but did not wire it, all 24 laws failed at runtime. They
cannot cover refusals reachable only on non-Linux targets or through overflow
at the address-space limit.
