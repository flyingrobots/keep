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

The original 2026-09-30 addition recorded `rustc 1.98.1 (48a229cea 2026-09-01)` and `cargo 1.98.1`. That historical entry did not record the Cargo commit hash or independent GC digest output; those missing facts cannot be reconstructed from the entry. It is not an approved toolchain exception or evidence of a current run. The pinned reconstruction below supersedes it as current verification evidence without rewriting fixture bytes.

The GC fixtures import exact bytes only from these previously accepted fixtures, at fixed offsets:

- `conformance/segment-store/v1/one-zero-segment.hex` (segment digest at
  273, record checksum at 177);
- `conformance/segment-store/v1/one-zero-catalog-generation-two.hex`
  (catalog digest at 320);
- `conformance/segment-store/v1/one-zero-head-generation-two.hex` (head
  checksum at 96);
- `conformance/segment-store/v1/empty-segment.hex` (segment digest at 128);
- the version-2 manifest digest, inventory digest, and profile digest the
  oracle already constructs.

At the initial GC record addition, no definition row was added because both grammars were already frozen in `definition.tsv`; the later GC derivation registration above changed the definition digest. The original entry reports that the temporary, removed write path materialized the two `.hex` files and the `artifacts.tsv` rows. The committed oracle is read-only and rejects drift.

## Pinned GC reconstruction and external digest verification

On 2026-10-03, the handwritten format oracle reconstructed the accepted GC records in memory under pinned `rustc 1.96.0` (commit `ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`) and `cargo 1.96.0` (commit `30a34c6821b57de0aaec83a901aca39f88f6778c`). Both debug and release comparisons to the frozen fixture bytes passed; the public GC intent/receipt runtime laws also passed in both profiles. No fixture was regenerated or re-baselined. Change kind: provenance correction; no implementation, API or format change.

[Verification output](../../../docs/testing-evidence/gc-fixture-provenance/verification.txt) records the compiler, Cargo and external `b3sum 1.8.5` versions, exact results and fixture hashes for source baseline `9a5fb8f938a5d4395c5b8d2677cd1153b0df60f6`. The [original archive](../../../docs/testing-evidence/gc-fixture-provenance/original-records.tar.gz) preserves the executable verification script and raw output; the readable copy removes trailing whitespace and terminal blank lines only. Execution used copied Docker source with the checked-in lockfile and an offline Cargo graph; each focused Cargo command had a 120-second deadline and 5-second kill grace. This does not establish per-test sandbox compliance or approve the larger PR.

The independent hashing step reads accepted fixture bytes and the specified domain-separated preimages; it does not call Keep's encoder, decoder or hashing helper. External hashing independently checks the preimage construction and stored digest, while `b3sum` still shares the BLAKE3 algorithm/library family, so this is not independent cryptographic-implementation validation.

For reproduction inside the Docker validation environment, decode each frozen `.hex` file without using Keep code:

```bash
scratch=$(mktemp -d)
for name in one-candidate-gc-intent one-candidate-gc-receipt; do
  perl -0ne 's/\s+//g; /\A(?:[0-9a-fA-F]{2})*\z/ or die "invalid fixture hex"; print pack "H*", $_;' \
    "conformance/segment-store/v2/$name.hex" > "$scratch/$name.bin"
done
intent=$scratch/one-candidate-gc-intent.bin
receipt=$scratch/one-candidate-gc-receipt.bin
```

These fixture-specific ranges follow the [GC grammar](../../../docs/formats/segment-store-v2/gc.md): candidate count at 44, one candidate beginning at 320, intent digest after its 392-byte preimage, and the intent/receipt checksums after their 424/288-byte preimages. They are commands for these accepted vectors, not a general parser.

```bash
{ printf 'keep.gc-candidate-set/v2\0'; dd if="$intent" bs=1 skip=44 count=4 status=none; dd if="$intent" bs=1 skip=320 count=72 status=none; } | b3sum --no-names
{ printf 'keep.gc-retirement-intent/v2\0'; dd if="$intent" bs=1 count=392 status=none; } | b3sum --no-names
{ printf 'keep.gc-retirement-intent-checksum/v2\0'; dd if="$intent" bs=1 count=424 status=none; } | b3sum --no-names
{ printf 'keep.gc-retirement-receipt-checksum/v2\0'; dd if="$receipt" bs=1 count=288 status=none; } | b3sum --no-names
```

| Claim | Expected digest and stored coordinate |
| --- | --- |
| Candidate-set digest | `6676bc9d70134a74cc9dfe397fb8a033fc45ebd903290d29b93018a097ee2b50`, intent offset 288 |
| Intent digest | `a9dd523326a686b89ac8f0c410421766afc221f2963bdafd430dce7f59edc701`, intent offset 392 |
| Intent checksum | `cefd325fbf7b1900e1a208c9c66ec5cfd03e565ea04a3bf9011338e9dabae749`, intent offset 424 |
| Receipt checksum | `d3dd8ddeea9ce78a278b39a3bdf61b957fff8f6cfee647f138fc720091389147`, receipt offset 288 |

Each stored value is 32 bytes; inspect it with `dd if="$intent" bs=1 skip=288 count=32 status=none | od -An -v -tx1 | tr -d ' \n'`, substituting the fixture and offset from the table. The archived script compares each computed result with that stored value and fails on any mismatch. The earlier profile and format-definition digest commands on this page were also rerun and their displayed outputs remain correct.
