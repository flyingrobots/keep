# Version 2 Corpus Origin

The corpus was constructed on 2026-07-29 with:

- `rustc 1.96.0 (ac68faa20 2026-05-25)`;
- `cargo 1.96.0 (30a34c682 2026-05-25)`; and
- `b3sum 1.8.5`.

`transitions.tsv` was first added on 2026-09-30 for the 21 migration
boundaries. On 2026-10-27 its migration rows were removed and its 11 rows were
transcribed from the initialization crash points `KEEP-CRASH-081` to `-091`
and the state table in `docs/formats/segment-store-v2/initialization.md`. No
independent oracle constructs this ledger. Its committed-byte guard is `transition_laws.rs`
under `xtask/tests/retention_store_v2_protocol_contract/`. The crash matrix
checks runtime behavior at the same boundaries without reading the TSV.

## Independent inputs

The oracle imports exact bytes only from these previously accepted fixtures:

- the one-zero `BlobId` canonical text and `LayoutId` binary identity from
  `conformance/layout/v1/layouts.tsv`.

The oracle constructs the
59-byte `BlobId` from the accepted binary grammar and verifies its length and
digest against the layout table; the table directly supplies the 60-byte
`LayoutId`. It does not call a production encoder, decoder, retention type,
initializer, serializer, or filesystem implementation.

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

The format-definition digest of the original definition was checked
independently with `b3sum`. For the current definition, this command is
expected to print the digest below, which the handwritten oracle computed; see
[Regeneration on 2026-10-27](#regeneration-on-2026-10-27):

```bash
{
  printf 'keep.segment-store-definition/v2\0'
  cat conformance/segment-store/v2/definition.tsv
} | b3sum --no-names
```

Expected output:

```text
f1a0ce10c0d363d145f8a693ea22651e47cf356caa14f1ef4f8958d8b29f361d
```

## Regeneration on 2026-10-27

Removing the migration rows from `definition.tsv` (ADR-0010,
[#198](https://github.com/flyingrobots/keep/issues/198)) changed the
definition digest from
`32381f1ac332d1277a7e1faf8f11576993cb55b7e85d2a110b74dc9c3b873427` to the
value above, so `format-marker.hex` and its `artifacts.tsv` row were
regenerated. The removed rows are the `domain.migration-*`,
`domain.store-identifier`, `domain.initial-retention-state`,
`domain.initial-gc-state`, `domain.empty-disposition-set`, and `migration.*`
keys; each was read only by the migration records.

The new bytes were taken from the committed handwritten oracle,
`xtask/tests/retention_store_v2_format_oracle.rs`, which hashes the edited
`definition.tsv` and constructs the marker from fixed offsets without any
production codec: its failure output on the unchanged fixtures named the exact
expected table row and marker bytes, which were written verbatim, and the
oracle then passed. `b3sum` was not available on the regeneration host, so the
shell check above was not rerun there; a reviewer should rerun it and expect
the digest shown. The retention fixtures do not depend on the definition and
are unchanged. The migration intent, receipt, inventory, and source table
were deleted rather than regenerated.

## Materialization boundary

A temporary ignored Rust test wrote the initially reviewed TSV and hexadecimal
artifacts from the handwritten oracle. That write path was removed immediately
after materialization. The committed oracle is read-only and rejects drift.
This construction claim covers the original format tables and hexadecimal
records; the later handwritten `transitions.tsv` has the separate verification
boundary described above.

Changing any fixture requires a deliberate specification change, an updated
definition or profile digest when affected, fresh independent construction,
and review of every dependent retention coordinate. A fixture is
never regenerated to make a production implementation pass.
