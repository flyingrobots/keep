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
| `format-marker.hex` | Canonical 96-byte `FORMAT` record |
| `migration-intent.hex` | Canonical 256-byte migration intent |
| `migration-receipt.hex` | Canonical 256-byte migration receipt |
| `one-anchor-root.hex` | Generation-1 root with one nontext namespace |
| `one-root-manifest.hex` | Generation-1 one-namespace manifest |
| `one-root-head.hex` | Generation-1 retention head |
| `ORIGIN.md` | Construction provenance and verification boundary |

Every text file uses UTF-8 or ASCII, LF line endings, and one final newline.
Every hex fixture is one lowercase hexadecimal line with one final newline.
In `artifacts.tsv`, `bound_digest_hex` is the marker content digest for
`format-marker`, the intent digest for `migration-intent`, the referenced
intent digest for `migration-receipt`, the canonical record digest for
`retention-root` and `retention-manifest`, and the referenced manifest digest
for `retention-head`.

## Frozen identities

The realization-profile digest is
`db1c1c1a50613ef11f7c0ee0882e37b6d24e2db2ca57783d01197ba51b61ce59`.
It hashes the exact `retention-profile.tsv` bytes under the registered profile
domain.

The format-definition digest is
`32381f1ac332d1277a7e1faf8f11576993cb55b7e85d2a110b74dc9c3b873427`.
It hashes the exact `definition.tsv` bytes under the registered format domain.
The definition binds the profile digest, every named domain, magic, version,
field order, record width, format limit, and migration synchronization mask.

The migration fixture preserves the version-1 one-zero segment and generation-1
catalog. Its canonical two-entry inventory digest is
`40bf5d49c34847ac9cf46a256f343cee80cd980d1405d2dd02ceff8f58d674f9`.
The derived logical store identifier is
`0cd9d3dfbec9b349fe42d21475271b0e8de23c043440d6427a1c37898ad1dd79`.
Fixture-only root device, mount, and file coordinates are `1`, `2`, and `3`;
they bind in-place recovery but do not enter the logical store identifier.

The retention fixture uses namespace bytes `00 2f ff`, proving the namespace is
opaque and not a path or Unicode string. Its one anchor combines the canonical
one-zero `BlobId` and `LayoutId` values from the existing layout corpus.

## Verification

Run:

```bash
cargo test --manifest-path xtask/Cargo.toml \
  --test retention_store_v2_format_oracle
```

The test-only oracle constructs every record from handwritten offsets and
domain preimages, compares exact fixture bytes and tables, and imports no
production version-2 codec. The repository protocol and documentation gates
route this corpus separately.

Passing this corpus is necessary but insufficient for issue #19. Production
code still needs parser, corruption, property, model, crash, recovery,
concurrency, fuzz, and public API evidence.
