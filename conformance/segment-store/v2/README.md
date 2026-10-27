# Durable Segment Store Version 2 Corpus

This corpus freezes independent canonical inputs and golden bytes for
`keep.segment-store/v2`. It proves the written format has one executable byte
interpretation. It does not prove that a production encoder, decoder,
initializer, retention transition, or garbage collector exists.

## Corpus files

| File | Contract |
| --- | --- |
| `definition.tsv` | Sorted format-definition key/value bytes |
| `retention-profile.tsv` | Registered realization-profile definition |
| `artifacts.tsv` | Golden artifact lengths, digests, checksums, and filenames |
| `transitions.tsv` | One stable crash identifier per initialization boundary, `KEEP-CRASH-081` to `-091` |
| `format-marker.hex` | Canonical 96-byte `FORMAT` record |
| `one-anchor-root.hex` | Generation-1 root with one nontext namespace |
| `one-root-manifest.hex` | Generation-1 one-namespace manifest |
| `one-root-head.hex` | Generation-1 retention head |
| `ORIGIN.md` | Construction provenance and verification boundary |

Every text file uses UTF-8 or ASCII, LF line endings, and one final newline.
Every hex fixture is one lowercase hexadecimal line with one final newline.
In `artifacts.tsv`, `bound_digest_hex` is the marker content digest for
`format-marker`, the canonical record digest for `retention-root` and
`retention-manifest`, and the referenced manifest digest for
`retention-head`.

The migration records, their inventory and source tables, and the
`KEEP-CRASH-053` to `-073` transition rows were removed with migration; the
[retirement ledger](../../../docs/formats/store/retired-identifiers.md)
records their last evidence.

## Frozen identities

The realization-profile digest is
`db1c1c1a50613ef11f7c0ee0882e37b6d24e2db2ca57783d01197ba51b61ce59`.
It hashes the exact `retention-profile.tsv` bytes under the registered profile
domain.

The format-definition digest is
`f1a0ce10c0d363d145f8a693ea22651e47cf356caa14f1ef4f8958d8b29f361d`.
It hashes the exact `definition.tsv` bytes under the registered format domain.
The definition binds the profile digest, every named domain, magic, version,
field order, record width, and format limit.

The retention fixture uses namespace bytes `00 2f ff`, proving the namespace is
opaque and not a path or Unicode string. Its one anchor combines the canonical
one-zero `BlobId` and `LayoutId` values from the existing layout corpus.

## Transition protocol

`transitions.tsv` mirrors the version-1 table: one row per initialization
durability operation with its pre-state, interrupted-state classification,
post-state, and recovery posture. `KEEP-CRASH-082` and `-091` are the only
rows whose interruption may leave an incomplete pre-commit stage and therefore
the only rows that plan a discard; `-089` and `-090` admit the committed
store.

The `cargo xtask durability-crash-matrix --sequence initialization`
harness executes 41 canonical initialization process-death cases at the same
11 boundaries: each at three positions plus one `during` case per directory
for `KEEP-CRASH-086`. It kills an isolated writer process group, compares the
restarted root against an independent expected-state model, requires the
resumed initializer to report the predicted state, and requires the recovered
store to publish, retain, and reconstruct its first blob. Host power loss
remains outside its claim.

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

Production code still needs parser, corruption, property, model, crash,
recovery, concurrency, fuzz, and public API evidence.
