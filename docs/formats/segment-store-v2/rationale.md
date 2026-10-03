# Format Rationale

This note records choices local to `keep.segment-store/v2`. ADR-0009 remains
authoritative for the cross-cutting retention and GC decision.

## Use a successor store version

Extending the version-1 root shape was rejected. Version 1 deliberately refuses
unknown entries, so treating new retention state as optional would weaken its
admission law and make old readers misclassify a mutated store. A durable
migration intent creates an explicit authority boundary.

Direct version-2 initialization was rejected for this version. Requiring one
admitted version-1 predecessor gives migration, compatibility, and recovery one
starting law instead of defining a second initialization protocol without a
consumer requirement.

## Stage fixed migration records

Writing `migration.intent`, `FORMAT`, or `migration.receipt` in place was
rejected because process death can expose partial canonical bytes. Exact
`.next` stages make incomplete bytes non-authoritative and publish each
canonical fixed record through an immutable no-replacement link.

## Preserve version-1 immutable bytes

Re-encoding segments, catalogs, or publication heads during migration was
rejected. Their bytes are already canonical and independently evidenced.
Preservation narrows migration to new authority and namespace state and permits
byte-for-byte rollback analysis without promising an automatic downgrade.

## Keep namespace bytes out of paths

Using caller namespace text as a directory name was rejected. Namespace bytes
may contain separators, zero bytes, or non-Unicode data and have no filesystem
semantics. A domain-separated digest supplies the physical coordinate while
the root record retains the exact bytes to detect collision or substitution.

## Retain empty namespace generations

Deleting empty namespaces was rejected because an old absent-state compare and
swap could become valid again. Persistent empty generations prevent that ABA
hazard. The fixed 4,096-namespace ceiling bounds the resulting manifest and
makes capacity refusal explicit.

## Use one global manifest

Enumerating mutable namespace directories during GC was rejected. One immutable
manifest binds the complete namespace map under a `LivenessGeneration`, so a
reader or GC planner cannot miss a concurrently created namespace.

## Store semantic records, not serializer output

Serde-defined persistence was rejected. Fixed headers, explicit widths,
big-endian integers, zero reserved bytes, canonical ordering, named digest
domains, and golden fixtures keep the protocol independent of Rust layout and
dependency defaults.

## Start with one realization profile

Version-2 catalogs expose one canonical location per logical record identity.
Pretending to support multiple representation policies would add an unproved
abstraction. The registered single-witness profile states the current law
exactly; another profile requires a successor specification and evidence.

## Charge closure evidence and reconstruction work separately

Counting unique closure members alone would let a layout repeat one small
chunk into an effectively unbounded reconstruction. Counting every repeated
identity as another node would misstate the canonical physical evidence.
Version 2 therefore deduplicates node, encoded-metadata, and member-digest
accounting by logical identity, while charging physical record length for
every chunk occurrence consumed during reconstruction. This keeps both the
evidence set and the work bound truthful.

## Use a kernel reader fence

A durable reader registry, lease, clock, and process liveness inference were
rejected. A persistent file with shared reader locks and an exclusive GC lock
has an observable process-death lifecycle. The fixed writer-then-reader lock
order avoids lock inversion. Publication does not take the reader lock because
it deletes no published immutable segment. Readers therefore double-collect
both mutable heads around transitive admission and reject a mixed view.

## Derive logical store identity

Random or physical-location store identifiers were rejected. A deterministic
digest of the admitted version-1 catalog, immutable pools, and target format
definition gives byte-identical stores one logical identity while the migration
intent separately binds physical coordinates for in-place recovery.

## Reserve GC names but refuse their state

Leaving the future GC namespace undefined was rejected because adding it later
would mutate the exact version-2 root grammar. Accepting placeholder bytes was
also rejected. Version 2 reserves the names, while their presence remains an
unsupported mandatory state until issue #21 supplies complete byte, parser,
crash, recovery, corruption, and fuzz evidence.

## Compare restart-stable root coordinates on reopen

The migration intent records the root's device, mount, and inode identity.
Only the first and last survive a remount: `statx.stx_mnt_id` is a mount
instance, not a volume. Comparing it on reopen made a correctly remounted
store refuse and would have made partial-prefix migration recovery reject
the store's own intent after a reboot.

Reopen therefore compares device and inode only, and future restart recovery
must apply the same rule. Meanwhile,
the migrating process still compares all three against its own observation.
The intent bytes are unchanged.

Rejected: removing the mount field. The definition digest covers the intent
layout, so the change would re-derive the format marker, the receipt, and
every version-2 fixture for no gain in restart safety. Rejected: the
filesystem UUID as the device coordinate in this change. That would require
separate platform admission, compatibility, and format decisions. Device
renumbering remains a refusal; this change does not introduce re-admission or
silently substitute a different identity coordinate.

## Verify completed migration before reporting completion

An exact receipt proves the migration records agree; it does not prove the current reserved namespace still admits. Recovery therefore invokes the read-only `StoreMigrationRecoveryStorage::verify_complete` capability after planning selects `Complete` and before returning success. The filesystem adapter reuses version-two namespace admission, retaining the original cause through `Observation` and `NamespacePreflight`. Existing corruption/planning refusals retain priority because this check follows planning. No namespace effect, synchronization or retention recovery is initiated.

Completed stores may contain published retention roots, manifests and heads. Reusing the partial-migration empty-directory preflight would reject valid post-migration state, so completion uses the existing version-two admission policy instead. Migration completion verifies its root/reserved directory contract; retention owns the interpretation of retention artifacts and stages. This adds a required method to the public recovery storage port; external implementations must implement equivalent effect-free admission. On-disk formats and identities are unchanged.
