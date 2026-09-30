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

<!-- markdownlint-disable MD013 -->

| State | Required response |
| --- | --- |
| no migration artifact | admit exact version 1 |
| intent stage only | finalize an exact stage or explicitly discard an incomplete pre-effect stage |
| durable intent only | verify intent and continue |
| intent plus a canonical prefix of v2 names | verify each name and continue |
| complete v2 shape without marker | verify directories and write marker |
| marker without receipt | reopen full v2 view and publish receipt |
| exact receipt with optional exact receipt stage | clean the stage and admit complete migration |

<!-- markdownlint-enable MD013 -->

A partial migration retry revalidates intent and existing bytes, resumes at the
first absent canonical step, and never replaces an entry. A missing predecessor,
changed version-1 coordinate, out-of-order name, wrong kind or bytes, conflicting
receipt, unknown entry, or changed root identity is unrecoverable ambiguity.

Death before durable intent leaves v1 plus at most its non-authoritative stage.
Death after durable intent leaves recovery-required v2 migration state.

### Recovery planning

`plan_store_migration_recovery` is the storage-independent form of the table.
It takes the intent the version-1 store derives today and one
`StoreMigrationResidue`, the observed presence and exact bytes of every fixed
name, and returns the one `StoreMigrationRecoveryPlan` the row prescribes:
admit version 1, discard one incomplete pre-effect stage and resume, resume
at the earliest forward phase the residue cannot prove complete, or complete.
Every other residue is a typed `StoreMigrationRecoveryAmbiguity`. The
persisted intent is compared on every coordinate but the mount identity, so
a rebooted root does not reject its own intent.

The planner reads no storage. An observer that produces the residue and a
storage that executes the plan by reopening each stage by device and inode
identity are not implemented; until they are, an interrupted migration still
waits for a human
([#108](https://github.com/flyingrobots/keep/issues/108)).
