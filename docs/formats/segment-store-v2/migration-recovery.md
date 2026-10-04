# Migration Protocol and Recovery

This page owns the ordered one-way migration protocol and partial-migration
recovery for `keep.segment-store/v2`. The root namespace, format marker, reader
fence, and migration record grammars are owned by
[Migration and recovery](recovery.md); the exact process-death boundaries are
owned by [Migration crash points](migration-crash.md).

## One-way migration protocol

Migration performs these ordered steps:

1. Admit and completely recover the exact version-1 store.
2. Revalidate its head, catalog, pools, root identity, and writer authority.
3. Publish `migration.intent` from `migration.intent.next` through the
   no-replacement fixed-stage protocol.
4. Create and verify persistent `reader.lock`.
5. Create the exact `retention`, `retention/roots`,
   `retention/manifests`, `gc`, `recovery`, and
   `recovery/dispositions` directories.
6. Synchronize every created parent and the store root.
7. Publish `FORMAT` from `FORMAT.next` through the fixed-stage protocol.
8. Reopen and verify the complete version-2 view.
9. Publish `migration.receipt` from `migration.receipt.next` through the
   fixed-stage protocol.

The [migration crash-point specification](migration-crash.md) owns
`KEEP-CRASH-053` through `KEEP-CRASH-073`.
Migration never rewrites or deletes admitted version-1 immutable bytes and
provides no automatic downgrade.

`FilesystemStoreMigrationAuthority` retains the writer lock and pinned root
and pools. It admits the version-1 namespace, Linux root identity, `HEAD`,
complete immutable-pool inventory, and selected catalog. Before mutation, it
requires the same canonical intent. Its port retains fixed-record handles,
verifies bytes and inode identity at each publication boundary, and admits only
ordered prefixes. It reopens the complete view before receipt staging and
leaves all version-1 immutable bytes untouched.
Version-1 admission refuses after a migration stage, `migration.intent`,
`reader.lock`, `FORMAT`, or version-2 directory exists. After durable intent,
only version-2 migration recovery may continue.

## Partial migration recovery

The migration recovery boundary admits only these ordered prefixes:

| State | Required response |
| --- | --- |
| no migration artifact | admit exact version 1 |
| intent stage only | finalize an exact stage or explicitly discard an incomplete pre-effect stage |
| durable intent only | verify intent and continue |
| intent plus a canonical prefix of v2 names | verify each name and continue |
| complete v2 shape without marker | verify directories and write marker |
| marker without receipt | reopen full v2 view and publish receipt |
| exact receipt with optional exact receipt stage | clean the stage and admit complete migration |

Every row also requires the admission checks in
[Executable recovery boundary](#executable-recovery-boundary). An intent stage
surviving a namespace effect, or a marker stage surviving a receipt effect,
refuses as `StageAfterEffect`. When a stage and canonical target both exist,
exact bytes and one shared device/inode identity are required before resumption.

A partial migration retry revalidates intent and existing bytes, resumes at the
first absent canonical step, and never replaces an entry. A missing predecessor,
changed version-1 coordinate, out-of-order name, wrong kind or bytes, conflicting
receipt, unknown entry, or changed root identity is unrecoverable ambiguity.

Death before durable intent leaves v1 plus at most its non-authoritative stage.
Death after durable intent leaves recovery-required v2 migration state.

## Executable recovery boundary

`FilesystemStoreMigrationAuthority::reopen_for_recovery` reacquires writer
authority without granting version-1 publication admission.
`recover_store_migration` revalidates the caller's freshly derived current
intent, then observes bounded residue and applies
`plan_store_migration_recovery` before adopting records or discarding stages.
Nested directory membership is checked before mutating recovery. An intent
stage surviving namespace creation, or a marker stage surviving receipt
publication, is refused as `StageAfterEffect`.

When both a stage and its canonical target exist, adoption verifies they share
the same device and inode as well as exact bytes before any forward phase,
including directory synchronization.

The receipt retains the exact observed namespace prefix and bound intent
digest, names the admitted plan, and lists the executed forward phases
through `executed_phases()`. The plan records the earliest unproven boundary;
it does not infer which synchronization calls completed before process death.
For `VersionOne`, `intent_digest()` returns `None`: no migration intent was
admitted. Every other plan reports the intent used by recovery. An incomplete
pre-effect intent stage has no complete persisted intent, so its discard path
uses the freshly verified current intent.
Restart compares device and inode identity; mount identity is same-process
evidence. Whenever an exact intent survives, recovery continues with its
persisted bytes.

The filesystem laws cover every forward prefix, every strict byte-prefix truncation of all three stages and unchanged version-1 bytes. The [restart ambiguity matrix](../../testing-evidence/migration-restart-matrix.md) covers corrupt records, contradictory and byte-equal substituted stages, invalid ordering, copied-root identity, immutable-pool and current-head damage, changed inventory, foreign receipts, unknown names and wrong kinds. Each new refusal retains a complete before/after witness of names, file identities and bytes and checks the existing typed failure boundary. The evidence record distinguishes kernel/filesystem behavior from platform-admission, process-death and power-loss claims.

## Completed namespace admission

After a plan selects `Complete`, recovery calls `StoreMigrationRecoveryStorage::verify_complete` before returning its receipt. The filesystem implementation applies existing version-two namespace admission: reserved GC/recovery entries must match the current protocol, while owned retention state is permitted. A refusal retains the original cause under `StoreMigrationRecoveryError::Observation` and `FilesystemMigrationRecoveryRefusal::NamespacePreflight`, before any recovery effect. This is namespace admission, not verification or recovery of retained content; retention remains the owner of those records and stages.
