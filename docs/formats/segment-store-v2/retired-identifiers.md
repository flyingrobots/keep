# Retired Identifiers

This ledger records requirement and crash identifiers that no longer name
anything in the code. A retired identifier is never reused, even for a
similar obligation; a new obligation takes a new identifier.

Each row names the last commit on `main` that carried the identifier's
evidence, and where its obligation went: it either disappeared with the
subject it constrained, or moved to a current identifier.

## Migration (ADR-0010, #198)

[ADR-0010](https://github.com/flyingrobots/keep/pull/199) replaced the
version-1 to version-2 migration with direct initialization and deleted the
migration in the same change. Nothing had been released and no store with
data of value existed, so no conversion replaces it.
[#108](https://github.com/flyingrobots/keep/issues/108) (migration recovery)
is retired with it.

### Requirements

<!-- markdownlint-disable MD013 -->

| ID | Obligation | Last evidence on `main` | Disposition |
| --- | --- | --- | --- |
| `KEEP-MIGRATION-001` | Version-1 stores stay admitted until a durable migration artifact exists | `tests/store_migration_recovery.rs` at `a58f237`; migration recovery laws under `src/adapters/store_migration/` at `6e7e2d6` | Disappeared: no migration artifact exists. Version-2 code refuses version-1 roots under `KEEP-RETENTION-014` until version-1 admission is deleted |
| `KEEP-MIGRATION-002` | Marker, intent, and receipt byte tables, domains, bounds, checksums, store identity, initial-state digests | `tests/store_format_marker.rs` at `aabfb89`; `tests/store_migration_intent.rs` at `fc20097`; `tests/store_migration_receipt.rs` at `94092ac`; `migration_format` fuzz target at `85919fb` | Marker part moved to `KEEP-FORMAT-001`. Intent, receipt, store identifier, and initial-state digests disappeared with their records |
| `KEEP-MIGRATION-003` | Migration revalidates the version-1 head, catalog, pools, root identity, and writer authority before mutation | `tests/store_migration_inventory.rs` at `89429a5`; `tests/store_migration_execution.rs` at `3fc9664`; authority laws at `6e7e2d6` | Disappeared: there is no version-1 source to revalidate. The persisted root-identity binding is retired by ADR-0010; live checks are `KEEP-PORTABILITY-003` |
| `KEEP-MIGRATION-004` | Every partial migration prefix continues idempotently, comparing restart-stable root coordinates | `tests/store_migration_recovery.rs` at `a58f237`; prefix and truncation laws at `6e7e2d6` | Prefix continuation moved to `KEEP-INITIALIZATION-002`. The root-coordinate comparison disappeared (ADR-0010) |
| `KEEP-MIGRATION-005` | Unknown, out-of-order, substituted, corrupt, or changed evidence is unrecoverable ambiguity | restart matrix laws at `6e7e2d6`; `docs/testing-evidence/migration-restart-matrix.md` at `e206298` | Moved to `KEEP-INITIALIZATION-002`: a root that is not an initialization prefix refuses unchanged |
| `KEEP-MIGRATION-006` | Migration never rewrites or deletes admitted version-1 immutable bytes | storage and recovery witnesses at `6e7e2d6`; migration crash restart witnesses at `c2414bf` | Disappeared: nothing converts a store |
| `KEEP-MIGRATION-007` | Process death at every migration boundary reaches a documented lawful state | `cargo xtask durability-crash-matrix --sequence migration` at `c2414bf` | Moved to `KEEP-INITIALIZATION-004` under new crash identifiers |
| `KEEP-MIGRATION-008` | Version-1 admission refuses every version-2 or partial-migration artifact | `tests/store_migration_compatibility.rs` at `85919fb`; `docs/testing-evidence/migration-compatibility-fuzz.md` at `1c886fd` | The version-2 part continues under `KEEP-RETENTION-014` until version-1 admission is deleted; the partial-migration part disappeared |

<!-- markdownlint-enable MD013 -->

The persisted reader root-identity requirement recorded in
[Retention reader root identity](../../adr/retention-reader-root-identity.md)
is superseded by ADR-0010 and replaced by the `KEEP-PORTABILITY-*` laws. Its
last evidence on `main` is
`src/adapters/retention/filesystem_retention_snapshot_identity_tests.rs` at
`5c32126`.

### Crash identifiers

`KEEP-CRASH-053` to `-073` named the 21 migration boundaries. The last
evidence for every row is the migration crash child and restart model at
`abbfe30` and `c2414bf`, and the transition ledger at `c2414bf`.

<!-- markdownlint-disable MD013 -->

| ID | Migration boundary | Disposition |
| --- | --- | --- |
| `KEEP-CRASH-053` | write `migration.intent.next` | Disappeared |
| `KEEP-CRASH-054` | synchronize the intent stage | Disappeared |
| `KEEP-CRASH-055` | link `migration.intent` | Disappeared |
| `KEEP-CRASH-056` | synchronize the root after the intent | Disappeared |
| `KEEP-CRASH-057` | remove the intent stage | Disappeared |
| `KEEP-CRASH-058` | synchronize the root after intent cleanup | Disappeared |
| `KEEP-CRASH-059` | admit the reader fence | Moved to `KEEP-CRASH-085` |
| `KEEP-CRASH-060` | admit the namespace prefix | Moved to `KEEP-CRASH-086` |
| `KEEP-CRASH-061` | synchronize the root after the namespace | Folded into `KEEP-CRASH-086`: every directory now synchronizes its parent |
| `KEEP-CRASH-062` | write `FORMAT.next` | Moved to `KEEP-CRASH-082` |
| `KEEP-CRASH-063` | synchronize the marker stage | Moved to `KEEP-CRASH-083` |
| `KEEP-CRASH-064` | link `FORMAT` | Moved to `KEEP-CRASH-087` |
| `KEEP-CRASH-065` | synchronize the root after the marker | Moved to `KEEP-CRASH-088`, now the commit point |
| `KEEP-CRASH-066` | remove the marker stage | Moved to `KEEP-CRASH-089` |
| `KEEP-CRASH-067` | synchronize the root after marker cleanup | Moved to `KEEP-CRASH-090` |
| `KEEP-CRASH-068` | write `migration.receipt.next` | Disappeared: the marker link is the commit point |
| `KEEP-CRASH-069` | synchronize the receipt stage | Disappeared |
| `KEEP-CRASH-070` | link `migration.receipt` | Disappeared |
| `KEEP-CRASH-071` | synchronize the root after the receipt | Disappeared |
| `KEEP-CRASH-072` | remove the receipt stage | Disappeared |
| `KEEP-CRASH-073` | synchronize the root after receipt cleanup | Disappeared |

<!-- markdownlint-enable MD013 -->

"Moved" means the obligation continues at a boundary of the
[initialization protocol](initialization.md); the boundary is new code with a
new identifier, not the migration boundary renamed. `KEEP-CRASH-084`
(the root synchronization after the marker stage), `-081` (the writer lock),
and `-091` (discarding an incomplete stage inside recovery) have no migration
predecessor. `KEEP-CRASH-078` to `-080` are reserved by version-2 catalog
recovery and are not assigned here.

### Code and evidence removed

- `src/adapters/store_migration/`: the migration records, inventory,
  authority, recovery planner and executor, and their unit laws.
- `tests/store_migration_*`, `tests/migration_descriptor_exhaustion.rs`, and
  the migration-completion and reader root-identity unit laws.
- The `migration_format` fuzz target and its seeds, replaced by
  `format_marker` for the marker decoder that remains.
- The migration crash child, restart model, and transition rows.
- `conformance/segment-store/v2/migration-intent.hex`,
  `migration-receipt.hex`, `inventory.tsv`, and `migration-source.tsv`, and
  the migration rows of `definition.tsv` and `artifacts.tsv`.
- `docs/formats/segment-store-v2/migration-crash.md`,
  `migration-inventory.md`, and `migration-recovery.md`, and the migration
  evidence records under `docs/testing-evidence/`.

PR [#191](https://github.com/flyingrobots/keep/pull/191) proposed typed
refusals at the migration storage boundaries; those parts are moot.
