# Migration and Recovery

This page owns version-2 filesystem migration and recovery.
[Migration protocol and recovery](migration-recovery.md) owns the ordered
one-way migration protocol and partial-migration recovery.

## Exact filesystem namespace

Version 2 preserves version-1 files and directories; it adds:

```text
reader.lock
FORMAT
migration.intent
migration.intent.next
migration.receipt
migration.receipt.next
FORMAT.next
retention/HEAD
retention/head.next
retention/root.next
retention/manifest.next
retention/roots/<namespace-digest>/<generation>-<root-digest>.root
retention/manifests/<generation>-<manifest-digest>.manifest
gc/intent
gc/receipt
recovery/disposition.next
recovery/dispositions/<artifact-digest>.receipt
```

`retention/HEAD`, every fixed `.next` stage, `gc/intent`, and `gc/receipt` are
optional according to the exact state tables below. Immutable-pool coordinates
are data-dependent but canonically named. Every other root or
protocol-directory entry is an unknown entry and unrecoverable ambiguity.
Operations are capability-relative and never follow links.

## Format marker

`FORMAT` is exactly 96 bytes:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:STORE:V2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `96` |
| 20 | 4 | flags | `0` |
| 24 | 32 | format-definition digest | registered v2 digest |
| 56 | 4 | maximum namespace count | `4,096` |
| 60 | 4 | reserved | zero |
| 64 | 32 | checksum | BLAKE3-256 over bytes `0..64` |

The definition and checksum domains are `keep.segment-store-definition/v2\0` and
`keep.segment-store-marker-checksum/v2\0`. A missing marker is version 1 only
when the exact version-1 namespace admits. An unsupported, corrupt,
substituted, or same-name/different-digest marker refuses.

The format-definition digest is BLAKE3-256 of its domain followed by the exact
corpus `definition.tsv` bytes. The format-marker digest is BLAKE3-256 of
`keep.store-format-marker/v2\0` followed by all 96 marker bytes.

`CanonicalStoreFormatMarker` produces the registered marker; `AdmittedStoreFormatMarker` admits its framing, checksum, definition, and namespace bound.
`CanonicalStoreMigrationIntent` retains typed intent coordinates; `CanonicalStoreMigrationReceipt` binds completion; admitted record types verify both.
`StoreMigrationStorage` names all 21 durability capabilities; `execute_store_migration` verifies current authority first and returns only after final synchronization.
`FilesystemStoreMigrationInventoryReader` inventories version-1 bytes under
retained writer authority. The fresh writer executes once; separate migration
recovery plans and executes the lawful remaining suffix after process death.
Version-1 reopen and version-1 recovery both refuse a migrated root.

## Reader fence

`reader.lock` is a persistent regular zero-length file whose contents and
existence alone prove nothing. Admission also admits an optional root
`head.next`, the residue of an interrupted [compaction](compaction.md)
successor, refused by every publication until recovered.

A version-2 reader acquires a kernel-managed shared lock on `reader.lock`
before opening catalog `HEAD` or `retention/HEAD`; the `ReaderFence` owns it
for the snapshot's lifetime; release or process death never deletes the file.

GC takes writer authority and then `reader.lock` exclusively, refusing while
readers hold it; publication proceeds beside readers (immutable successors).

## Migration records

Migration is a one-way explicit migration under exclusive writer authority.

`migration.intent` is exactly 256 bytes:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:MIG:INT2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `256` |
| 20 | 4 | flags | `0` |
| 24 | 8 | catalog generation named by version-1 `HEAD` | positive |
| 32 | 8 | catalog length named by version-1 `HEAD` | exact admitted length |
| 40 | 32 | catalog digest named by version-1 `HEAD` | exact admitted digest |
| 72 | 32 | predecessor catalog digest | zero for generation 1 |
| 104 | 32 | immutable-pool inventory digest | canonical complete inventory |
| 136 | 8 | root device identity | admitted Linux `dev_t` |
| 144 | 8 | root mount identity | admitted Linux `statx.stx_mnt_id` |
| 152 | 8 | root file identity | admitted Linux `statx.stx_ino` |
| 160 | 32 | target format-definition digest | exact registered v2 digest |
| 192 | 32 | new store identifier | deterministic derivation below |
| 224 | 32 | checksum | BLAKE3-256 over bytes `0..224` |

The checksum domain is `keep.store-migration-intent-checksum/v2\0`. The
receipt's intent digest is BLAKE3-256 of
`keep.store-migration-intent/v2\0` followed by all 256 intent bytes.

The [migration inventory](migration-inventory.md) defines its domain and law:
each migration inventory entry is exactly 56 bytes, and the fixed maximum is
2,097,152 entries. The intent therefore binds the exact catalog generation,
length, and digest named by the admitted version-1 `HEAD`.

On Linux, the root device coordinate is `dev_t`, reconstructed from
`statx.stx_dev_major` and `statx.stx_dev_minor`; mount and file use
`statx.stx_mnt_id` and `statx.stx_ino`. Each is big-endian `u64`.

The deterministically derived store identifier is:

```text
BLAKE3-256("keep.store-identifier/v2\0" ||
           catalog-generation-u64 ||
           catalog-length-u64 ||
           catalog-digest ||
           predecessor-catalog-digest ||
           immutable-pool-inventory-digest ||
           target-format-definition-digest)
```

Integer fields use their fixed-width big-endian bytes. Root device, mount, file
identity, caller identity, path, and time do not enter the identifier. The
migration intent separately binds the physical root coordinates so in-place
recovery refuses a substituted store.

### Root identity across restart

The three root coordinates the intent records do not have the same lifetime.
`statx.stx_mnt_id` names a mount instance: it can change when a filesystem
is unmounted and mounted again or after a reboot. A restart path must not
require the historical mount identifier. The device and inode coordinates
name the volume and the root directory and survive remounts on the admitted
platform.

The Linux [statx reference](https://man7.org/linux/man-pages/man2/statx.2.html)
defines the mount identifier separately from device and inode coordinates.
The lifetime distinction above is the reason Keep does not use it as restart
authority.

The restart-stable root identity is therefore the pair `(device, file)`:

- `FilesystemStoreMigrationAuthority` compares all three coordinates, but
  only against the observation it made itself when it opened the root in the
  same process; that comparison catches a root swapped underneath a running
  migration and never crosses a restart.
- `FilesystemVersionTwoAdmission::reopen` compares device and
  file only and refuses with `RootIdentityChanged { coordinate: Device | File,
  .. }`. A remounted store admits; a store copied to another device or
  restored into a different directory refuses.

Partial-prefix migration recovery uses the same restart comparison; its
authority and resumption protocol are specified in
[the executable recovery boundary](migration-recovery.md#executable-recovery-boundary).

The mount coordinate stays in the record as the migration-time observation.
It remains evidence for same-process migration checks, not restart authority.

Limit: `dev_t` is stable across reboots only while the block device keeps its
major and minor numbers. A device-mapper or hot-plug renumbering makes a
correct store refuse with `RootIdentityChanged { coordinate: Device, .. }`;
version 2 defines no re-admission for that case, and a successor coordinate
(the filesystem UUID) is the rationale's recorded alternative if it proves
necessary.

`migration.receipt` is exactly 256 bytes:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:MIG:REC2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `256` |
| 20 | 4 | flags | `0` |
| 24 | 32 | migration-intent digest | exact durable intent |
| 56 | 32 | store identifier | exact intent value |
| 88 | 32 | format-marker digest | exact verified marker |
| 120 | 32 | initial retention-state digest | exact no-payload digest below |
| 152 | 32 | initial GC-state digest | exact no-payload digest below |
| 184 | 32 | disposition namespace digest | exact no-payload digest below |
| 216 | 8 | completed synchronization mask | every mandatory bit set |
| 224 | 32 | checksum | BLAKE3-256 over bytes `0..224` |

Its checksum domain is `keep.store-migration-receipt-checksum/v2\0`. Unknown
synchronization bits, a missing mandatory bit, or any mismatch with the intent
refuses.

The three initial-state fields are the no-payload digests
`BLAKE3-256("keep.initial-retention-state/v2\0")`,
`BLAKE3-256("keep.initial-gc-state/v2\0")`, and
`BLAKE3-256("keep.empty-disposition-set/v2\0")`. At completed migration,
absence of `retention/HEAD` is the canonical empty retention state only while
all retention stages and pools are empty. Any retention artifact routes through
recovery instead. Direct version-2 initialization is undefined.

The exact offsets and fixtures are requirement `KEEP-MIGRATION-002`. The fresh
writer emits only those canonical records; success is not restart evidence.
A migrated store is admitted for forward publication. Partial-prefix recovery
now plans and executes the lawful remaining migration suffix under writer
authority; the 68-case migration process-death matrix supplies restart evidence.
The [restart matrix](../../testing-evidence/migration-restart-matrix.md) and [compatibility evidence](../../testing-evidence/migration-compatibility-fuzz.md) record the additional coverage delivered for #111 and #112.

## Retention publication recovery

[Retention publication recovery](retention-recovery.md) owns fixed-stage classification, synchronization before publication, exact retained evidence, and the `KEEP-CRASH-036` through `052` process-death boundaries.

## Explicit disposition of protected orphans

This operation accepts complete protected orphans only. Incomplete retention stages still refuse before recovery effects and remain preserved under [the bounded recovery contract](retention-recovery.md); incomplete-stage disposition remains deferred to #155.

A complete stage that recovery linked into its pool but that no head ever
committed is a recovery-protected orphan: publication refuses with
`RetainedStage` until a person or an explicit policy decides.
`FilesystemRetentionPublicationAuthority::dispose` takes writer authority,
runs recovery, refuses while any reader holds the fence, acquires the fence
exclusively, and records a `RecoveryDispositionReceipt` through the
fixed-stage protocol (`recovery/disposition.next`, link without replacement
to `recovery/dispositions/<artifact-digest>.receipt`, synchronize, remove the
stage, synchronize `recovery`); only then is the retained retention stage
removed and `retention` synchronized. Process death anywhere leaves a
recoverable stage or a durable decision; the next `dispose` with the same
request resumes from it, and a residue naming another decision is a typed
ambiguity.

> **Warning.** Disposition changes what the store will keep. `Retire` unlinks
> the orphan's immutable pool entry after the receipt is durable; the bytes
> are gone and only the receipt records why. `Finalize` keeps the entry as a
> durable artifact a byte-identical publication may reuse and is refused
> while no retention head is published. A manifest stage must be disposed
> before the root it names. The dry run is `plan_recovery_disposition`;
> verify the result with `recover`, which reports `Clean`.

Every receipt is admitted by namespace census as a regular file under its
canonical name. GC planning admits only the exact receipt: a `segment`
artifact retired under exactly the planned snapshot's coordinates is
released; any other receipt is stale. `GcRetirementIntent` and
`GcRetirementReceipt` are owned by [GC](gc.md) and written by
[GC execution](gc-execution.md).
