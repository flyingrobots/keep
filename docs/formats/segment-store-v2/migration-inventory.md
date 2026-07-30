# Migration Inventory

This page owns the bounded canonical digest over version-1 immutable segment
and catalog pools used by `migration.intent`.

## Entry grammar

One migration inventory entry is exactly 56 bytes:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 1 | artifact kind | `1` segment or `2` catalog |
| 1 | 7 | reserved | zero |
| 8 | 8 | catalog generation | zero for segment; positive for catalog |
| 16 | 8 | artifact length | exact positive length |
| 24 | 32 | artifact digest | exact admitted segment or catalog digest |

Entries are sorted by their complete 56-byte canonical encoding and are
duplicate-free. The maximum is 2,097,152 entries across both pools. Count and
length arithmetic is checked before bytes are retained or allocated.

The inventory digest is:

```text
BLAKE3-256("keep.store-v1-pool-inventory/v2\0" ||
           entry-count-u32 ||
           canonical-entry-bytes)
```

The digest is streamed; the complete encoded inventory is never required in
memory.

## Admission

Migration inventories the exact pinned version-1 `segments` and `catalogs`
directories under writer authority. Every regular entry must have the one
canonical physical name derived from its verified semantic digest and, for a
catalog, generation. Each artifact is reopened without following links and
completely admitted before its semantic coordinate enters the digest.

An unknown name, alternate case or width, alias, duplicate semantic coordinate,
wrong kind, link, changed directory, changed artifact, corrupt bytes, count
overflow, or entry above the fixed maximum refuses migration. File existence,
iteration order, path spelling, modification time, and physical file identity
do not enter the digest.

The exact one-segment, one-catalog input and its canonical entries are frozen in
the version-2 corpus
[`inventory.tsv`](../../../conformance/segment-store/v2/inventory.tsv).

`StoreMigrationInventoryEntry` derives canonical bytes only from admitted
artifacts. `StoreMigrationInventoryHasher` requires the bounded entry count
before streaming, retains only the preceding entry, refuses duplicate or
out-of-order evidence, and reproduces the frozen digest. Capability-relative
filesystem inventory and mutation revalidation remain unimplemented.
