# Durable Segment Store Version 2 Corpus

This corpus freezes independent canonical inputs and golden bytes for
`keep.segment-store/v2`. It proves the written format has one executable byte
interpretation. It does not prove that a production encoder, decoder,
migration, retention transition, or garbage collector exists.

## Corpus files

| File | Contract |
| --- | --- |
| `definition.tsv` | Sorted format-definition key/value bytes |
| `retention-profile.tsv` | Registered realization-profile definition |
| `inventory.tsv` | Canonical one-segment, one-catalog migration inventory |
| `migration-source.tsv` | Exact version-1 and derived migration coordinates |
| `artifacts.tsv` | Golden artifact lengths, digests, checksums, and filenames |
| `transitions.tsv` | One stable crash identifier per migration and GC boundary, `KEEP-CRASH-053` to `-087` |
| `gc-plan.tsv` | The deterministic GC plan for the frozen version-2 store: every segment's classification |
| `mutations.tsv` | The corruption ledger: one frozen byte mutation per structural field of `FORMAT`, both migration records, the retention root, manifest, and head, both GC records, and the disposition receipt, with its exact first refusal, verification stage, and requirement |
| `format-marker.hex` | Canonical 96-byte `FORMAT` record |
| `migration-intent.hex` | Canonical 256-byte migration intent |
| `migration-receipt.hex` | Canonical 256-byte migration receipt |
| `one-anchor-root.hex` | Generation-1 root with one nontext namespace |
| `one-root-manifest.hex` | Generation-1 one-namespace manifest |
| `one-root-head.hex` | Generation-1 retention head |
| `one-candidate-gc-intent.hex` | Generation-1 GC retirement intent naming one candidate |
| `one-candidate-gc-receipt.hex` | Generation-1 GC retirement receipt completing that intent |
| `one-orphan-retire-disposition.hex` | Disposition retiring the one-zero segment as a complete orphan |
| `ORIGIN.md` | Construction provenance and verification boundary |

Every text file uses UTF-8 or ASCII, LF line endings, and one final newline.
Every hex fixture is one lowercase hexadecimal line with one final newline.
In `artifacts.tsv`, `bound_digest_hex` is the marker content digest for
`format-marker`, the intent digest for `migration-intent`, the referenced
intent digest for `migration-receipt`, the canonical record digest for
`retention-root` and `retention-manifest`, the referenced manifest digest
for `retention-head`, the intent digest for `gc-intent`, the referenced
intent digest for `gc-receipt`, and the artifact identity digest for
`recovery-disposition`.

## Frozen identities

The realization-profile digest is
`db1c1c1a50613ef11f7c0ee0882e37b6d24e2db2ca57783d01197ba51b61ce59`.
It hashes the exact `retention-profile.tsv` bytes under the registered profile
domain.

The format-definition digest is
`a4a010cee5da8aa3ba153c5034f436b92742c6c1f7cf6b43d890ad5fd5b5cf89`.
It hashes the exact `definition.tsv` bytes under the registered format domain.
The definition binds the profile digest, every named domain (including the
GC catalog-successor-proof, segment-pool, and disposition-set derivations),
magic, version, field order, record width, format limit, migration
synchronization mask, and the registered recovery-disposition enumerations.

The migration fixture preserves the version-1 one-zero segment and generation-1
catalog. Its canonical two-entry inventory digest is
`40bf5d49c34847ac9cf46a256f343cee80cd980d1405d2dd02ceff8f58d674f9`.
The derived logical store identifier is
`a046bebd6d1b05d33b56e26e131e5d44bc1872d875d339f837eecd21caa0c1e1`.
Fixture-only root device, mount, and file coordinates are `1`, `2`, and `3`;
they bind in-place recovery but do not enter the logical store identifier.

The retention fixture uses namespace bytes `00 2f ff`, proving the namespace is
opaque and not a path or Unicode string. Its one anchor combines the canonical
one-zero `BlobId` and `LayoutId` values from the existing layout corpus.

## Transition protocol

`transitions.tsv` mirrors the version-1 table: one row per migration and GC
durability operation with its pre-state, interrupted-state classification,
post-state, and recovery posture. `KEEP-CRASH-053`, `-062`, `-068`, `-074`,
and `-082` are the only rows whose interruption may leave an incomplete
pre-effect stage and therefore the only rows that plan a discard; `-072` and
`-073` admit the complete migration, `-086` and `-087` the complete
retirement. `cargo xtask durability-crash-matrix --sequence gc` executes the
42 GC cases the same way over one disposed orphan.

The `cargo xtask durability-crash-matrix --sequence migration` harness
executes 68 canonical process-death cases at the same boundaries: 21 boundaries at
three positions plus one `during` case per admitted directory-prefix length
for `KEEP-CRASH-060`. It kills an isolated writer process group, compares the
restarted root against an independent expected-state model, requires the
production planner to report the predicted recovery plan, and requires the
recovered store to be one complete migration with every version-1 byte
intact. Host power loss remains outside its claim.

The GC fixtures name the version-1 one-zero segment as their one candidate,
with that segment's record checksum standing in for its verification-evidence
digest; the generation-two catalog digest and head checksum stand in for the
catalog-successor coordinates; the migration inventory digest is the
segment-pool identity; the empty-segment digest is the post-retirement pool
state; and the reader-lock coordinates are the fixture-only values `4`, `5`,
and `6`. These are format evidence for the record grammars, not a consistent
store: the catalog they name still lists the candidate, which a real planner
would refuse.

## Verification

Run:

```bash
cargo test --manifest-path xtask/Cargo.toml \
  --test retention_store_v2_format_oracle
cargo test --manifest-path xtask/Cargo.toml \
  --test retention_store_v2_protocol_contract transition_laws
```

The test-only oracle constructs every record from handwritten offsets and
domain preimages, compares exact fixture bytes and tables, and imports no
production version-2 codec. The repository protocol and documentation gates
route this corpus separately.

`transition_laws.rs` checks the handwritten transition ledger's committed
shape, operation order, and recovery-posture claims. The format oracle does
not construct or compare `transitions.tsv`; the crash matrix checks runtime
behavior without reading its bytes.

Passing this corpus is necessary but insufficient for migration recovery
(#108), restart corruption (#111), or compatibility and fuzz coverage (#112).
Production code still needs parser, corruption, property, model, crash, recovery,
concurrency, fuzz, and public API evidence.

## Mutation ledger

`mutations.tsv` rows are `case`, `record`, `base_fixture`, `operation`
(`replace-v1`, `xor-v1`, `truncate-v1`, `append-v1`, `delete-v1`), `offset`,
`span_length`, `parameter` (lowercase hex or `-`), `checksum_posture`
(`preserve-v1`; `recompute-v1` recomputes inner set digests and the trailer;
`recompute-trailer-v1` recomputes only the record's digest and checksum;
`recompute-checksum-v1` only its checksum), `expected_outcome` as
`<record>.<variant>` of the public decoder's first refusal, `stage`
(`framing`, `checksum`, `identity`, or `binding`, ordered like
`VerificationDepth`), and `requirement`. `tests/segment_store_mutations.rs`
applies every row through the public decoders and requires the exact outcome
and stage; `cargo xtask conformance-check` refuses a malformed row. A ledger
row is never regenerated to make a decoder pass: a differing first refusal is
a specification question.
