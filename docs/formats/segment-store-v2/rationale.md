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

## Gate catalog publication on every retained closure

ADR-0009 requires version-2 catalog publication to verify every current
retained closure against the candidate catalog before it replaces `HEAD`.
`publish_retained_catalog_generation` implements that rule over the
`RetainedCatalogPublicationStorage` port (#125). It is the entry point a
version-2 catalog writer plugs into; this change adds no filesystem version-2
catalog publisher.

Under writer authority, the gate runs the pure catalog preflight first. It
then reads the retention head, the manifest that head names, and every root
the manifest names. Each record is decoded and cross-checked against the
record that selected it: the head names the manifest's digest, generation,
predecessor and length, and each manifest entry names its root's namespace,
generation and digest. The gate then calls `verify_retention_closure` for each
root against the candidate snapshot, using the root's own profile and limits.
Only then does the first catalog storage call run. A refused candidate
therefore leaves no stage, pool entry, or head change. After `head.next`
verification, the gate rereads the retention head and requires it to equal the
admitted head before it calls `replace_head`.

Storage returns exact bytes and the gate does all the decoding. Storage that
decoded its own records would be vouching for evidence it supplies. The
catalog stages are reachable only through `catalog_stages`, which takes a
`RetainedClosureAdmission`. Only the gate can create that value. A version-2
publisher whose stage type has no other constructor therefore cannot be passed
to the ungated `publish_catalog_generation`, which stays unchanged for
version 1.

The gate holds one manifest of at most 4,096 entries and one root at a time.
Each closure is limited by its root's admitted limits. Roots are verified in
manifest order, and the first refusal is returned.

Revalidation compares only the head, because manifest and root pool entries
are immutable and named by digest. An unchanged canonical head therefore
selects exactly the admitted evidence. The candidate catalog is held in
memory, and `verify_head_view` reverifies its pool copy.

Crash and refusal states:

- Refusal during evidence admission or closure verification happens before any
  catalog call, so there is nothing to recover.
- A head change refused at revalidation leaves the linked catalog, the linked
  segment if one was staged, and a verified `head.next`. These are the same
  states a crash at that point leaves in the version-1 protocol. They are
  recovery state, never repaired automatically.
- Process death at any point leaves exactly the version-1 crash states. The
  gate adds only reads.

Recovery implication: finalizing a staged catalog head must run this same
retained-closure admission against the staged catalog; changing the admission
type that recovery accepts is not enough. Version-2 next-head finalization
(#198) does exactly that, as recorded below.

Rejected alternatives:

- Extending `CatalogPublicationStorage` with retention reads. Any implementation
  could then be driven through the ungated entry point.
- Having storage return admitted evidence. The gate would trust the storage
  it is checking.
- Revalidating before `head.next` creation. That leaves a wider unchecked
  window before the atomic replacement.
- Rereading every root at commit. Immutable digest-named records make the head
  comparison sufficient.
- Relying on ingestion's preserve-all-records behaviour. That is a property of
  one writer, not of the publication boundary.

## Run one stage implementation under both writer authorities

The version-2 writer (#198) needs the same segment, catalog, and head
transitions as version 1. A copy would let the two formats drift in exactly
the steps that decide durability. `FilesystemCatalogStages` therefore holds the
pinned catalog directories, the writable stages, and the stage-authority
token, and it is the only `CatalogPublicationStorage` implementation of those
transitions. `FilesystemCatalogPublisher` owns one beside its version-1 writer
lock. `FilesystemVersionTwoCatalogStages` wraps one under the version-2 writer
lock and has no public constructor. Each forwards transitions unchanged.

The version-2 stage type is reachable only through `catalog_stages`, so only
the retained-closure gate can drive it. Repository tasks install an observer
on that type for crash injection rather than wrapping the port, because a
wrapper cannot obtain the stages without the gate's admission.

Readers admit a retained catalog `head.next`. It is a version-1 file that
version 2 preserves, and a reader selects only `HEAD`, so refusing it would
stop readers for the whole window in which publication stages a head. Writer
admission still refuses it, so a writer never starts over unrecovered state.

Catalog-stage recovery on version 2 reuses the version-1 executors under a
separate `FilesystemVersionTwoRecoveryAdmission`. That admission differs from
writer admission only in admitting `head.next`. Next-head finalization is the
one recovery route that changes `HEAD`, so a version-2 finalizer runs the
gate's own `admit_retained_closures` before every candidate verification that
can precede replacement, including the one immediately before the rename.
Refusal leaves `head.next` and the pools unchanged.

Rejected alternatives:

- Copying the stage code into a version-2 publisher. The formats would drift.
- Implementing the ungated port for the version-2 publisher behind a runtime
  check. The type system would no longer prevent ungated publication.
- Letting `FilesystemVersionTwoAdmission` admit `head.next`. Writers and
  retention publication would then start over unrecovered catalog state.
- Gating only the planner. Planning is pure and runs before the writer lock.
  The retention head can change between planning and replacement.
