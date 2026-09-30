# Version 2 Corpus Origin

The corpus was constructed on 2026-07-29 with:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- `cargo 1.96.0 (30a34c682 2026-05-25)`; and
- `b3sum 1.8.5`.

`transitions.tsv` was added on 2026-09-30 by transcribing the 21 boundaries of
`StoreMigrationPhase::ALL` and the recovery table in
`docs/formats/segment-store-v2/migration-recovery.md`; the crash matrix in
`xtask` is its executable check.

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
32381f1ac332d1277a7e1faf8f11576993cb55b7e85d2a110b74dc9c3b873427
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
