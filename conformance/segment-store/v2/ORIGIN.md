# Version 2 Corpus Origin

The corpus was constructed on 2026-07-29 with:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- `cargo 1.96.0 (30a34c682 2026-05-25)`; and
- `b3sum 1.8.5`.

`transitions.tsv` was added on 2026-09-30 by transcribing the 21 boundaries of
`StoreMigrationPhase::ALL` and the recovery table in
`docs/formats/segment-store-v2/migration-recovery.md`; the crash matrix in
`xtask` is its executable check.

`gc-plan.tsv` was added on 2026-09-30 by running `plan_gc` over the frozen
store (the one-zero bundle catalog at generation one, retained by the
one-anchor root under the generation-one manifest) and transcribing the
classification of its one segment; `src/adapters/gc/planner_tests.rs`
recomputes it from the fixtures on every run.

## Disposition enumeration registration

On 2026-09-30 `definition.tsv` gained four rows: the
`keep.recovery-disposition-artifact/v2\0` domain and the artifact-kind,
decision, and classification enumerations the `RecoveryDispositionReceipt`
grammar had left unregistered. That changed the format-definition digest
and therefore the format marker, the migration intent, the migration
receipt, the derived store identifier, and their `artifacts.tsv` and
`migration-source.tsv` rows. Every affected fixture was rematerialized
through the same temporary, removed write path from the handwritten oracle;
no other fixture changed. `one-orphan-retire-disposition.hex` was added in
the same pass: it retires the one-zero segment (identity digest at 273 of
`one-zero-segment.hex`, content digest of its exact bytes under the
artifact domain, length 337) as a complete orphan under the generation-two
catalog and head, the generation-one manifest, and the fixture-only
reader-lock coordinates `4`, `5`, `6`; its decision-evidence digest is the
segment's record checksum at 177, fixture-only evidence like the GC intent's
candidate evidence.

## GC derivation registration

On 2026-09-30 `definition.tsv` gained three rows: the
`keep.gc-catalog-successor-proof/v2\0`, `keep.gc-segment-pool/v2\0`, and
`keep.gc-disposition-set/v2\0` domains under which GC execution derives the
intent's proof, pool-identity, and disposition-set digests. As with the
disposition registration, the format-definition digest changed and the
format marker, migration intent, migration receipt, store identifier, and
their `artifacts.tsv` and `migration-source.tsv` rows were rematerialized
through the same temporary, removed write path; every other fixture is
byte-identical. The GC intent fixture keeps its fixture-only proof, pool,
and disposition-set digests. `transitions.tsv` gained rows `KEEP-CRASH-074`
through `-087` by transcribing `GcExecutionPhase::ALL` and the state table in
`docs/formats/segment-store-v2/gc-execution.md`.

## Independent inputs

The oracle imports exact bytes only from these previously accepted fixtures:

- `conformance/segment-store/v1/one-zero-segment.hex`;
- `conformance/segment-store/v1/one-zero-catalog.hex`;
- `conformance/segment-store/v1/one-zero-head.hex`;
- the one-zero `BlobId` canonical text and `LayoutId` binary identity from
  `conformance/layout/v1/layouts.tsv`.

It parses the version-1 head coordinate, catalog predecessor, and segment and
catalog semantic digests directly from fixed offsets. The oracle constructs the
59-byte `BlobId` from the accepted binary grammar and verifies its length and
digest against the layout table; the table directly supplies the 60-byte
`LayoutId`. It does not call a production encoder, decoder, retention type,
migration adapter, serializer, or filesystem implementation.

## Definition verification

The profile digest was checked independently with:

```bash
{
  printf 'keep.retention-realization-profile/v1\0'
  cat conformance/segment-store/v2/retention-profile.tsv
} | b3sum --no-names
```

Exact output:

```text
db1c1c1a50613ef11f7c0ee0882e37b6d24e2db2ca57783d01197ba51b61ce59
```

The format-definition digest was checked independently with:

```bash
{
  printf 'keep.segment-store-definition/v2\0'
  cat conformance/segment-store/v2/definition.tsv
} | b3sum --no-names
```

Exact output:

```text
a4a010cee5da8aa3ba153c5034f436b92742c6c1f7cf6b43d890ad5fd5b5cf89
```

## Materialization boundary

A temporary ignored Rust test wrote the initially reviewed TSV and hexadecimal
artifacts from the handwritten oracle. That write path was removed immediately
after materialization. The committed oracle is read-only and rejects drift.

Changing any fixture requires a deliberate specification change, an updated
definition or profile digest when affected, fresh independent construction,
and review of every dependent migration and retention coordinate. A fixture is
never regenerated to make a production implementation pass.

## GC record addition

The GC retirement intent and receipt fixtures were added on 2026-09-30 with
`rustc 1.98.1 (48a229cea 2026-09-01)` and `cargo 1.98.1`. They import exact
bytes only from these previously accepted fixtures, at fixed offsets:

- `conformance/segment-store/v1/one-zero-segment.hex` (segment digest at
  273, record checksum at 177);
- `conformance/segment-store/v1/one-zero-catalog-generation-two.hex`
  (catalog digest at 320);
- `conformance/segment-store/v1/one-zero-head-generation-two.hex` (head
  checksum at 96);
- `conformance/segment-store/v1/empty-segment.hex` (segment digest at 128);
- the version-2 manifest digest, inventory digest, and profile digest the
  oracle already constructs.

The definition digest is unchanged: no definition row was added, because
both grammars were already frozen in `definition.tsv`. The same temporary,
removed write path materialized the two `.hex` files and the `artifacts.tsv`
rows; the committed oracle is read-only and rejects drift.
