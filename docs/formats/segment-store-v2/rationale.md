# Format Rationale

This note records choices local to `keep.segment-store/v2`. ADR-0009 remains
authoritative for the cross-cutting retention and GC decision.

## Use a successor store version

Extending the version-1 root shape was rejected. Version 1 deliberately refuses
unknown entries, so treating new retention state as optional would weaken its
admission law and make old readers misclassify a mutated store.

## Initialize directly instead of migrating

The first revision entered version 2 only by migrating a version-1 store.
ADR-0010 replaced that with direct initialization: nothing was released and
no store of value existed, and carrying a migration made every capability
exist twice. After the first release, a format change ships a separately
invoked upgrade tool that writes a new store and never modifies the source;
see the ADR.

## Commit by linking the marker last

Writing `FORMAT` in place was rejected because process death can expose
partial canonical bytes. `FORMAT.next` is written and synchronized before any
directory exists, so its presence is the evidence of an initialization in
progress, and it is linked to `FORMAT` without replacement only after the
complete namespace is durable. Its removal follows the commit
synchronization, so a root without `FORMAT.next` can only be observed after
the commit is durable. A separate commit record was rejected: it would add a
second fixed record to admit with no state the stage does not already prove.
Recording the root directory's identity was rejected by ADR-0010.

## Keep version-1 grammars byte for byte

Re-encoding segments, catalogs, or publication heads in version 2 was
rejected. Their bytes are already canonical and independently evidenced.

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

## Persist no store identifier

Random or physical-location store identifiers were rejected. The migration
intent once carried a deterministic, content-derived identifier; it was
deleted with migration, and initialization records none. Two stores with the
same bytes are indistinguishable by design; telling an intended store from a
valid stranger needs an expectation the caller keeps, as ADR-0010 states.

## Reserve GC names but refuse their state

Leaving the future GC namespace undefined was rejected because adding it later
would mutate the exact version-2 root grammar. Accepting placeholder bytes was
also rejected. Version 2 reserves the names, while their presence remains an
unsupported mandatory state until issue #21 supplies complete byte, parser,
crash, recovery, corruption, and fuzz evidence.

## Bind no store to its root directory

The migration intent recorded the root's device, mount, and inode, and reopen
compared device and inode. That binding refused every backup restore and copy
while proving less than it seemed: an intact store renamed into the path, or
an older snapshot restored into the same inode, still admitted. ADR-0010
removed it with migration. Live checks remain: lock-file identity, the
same-process protocol-directory binding, and no-follow capability access.

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
transitions. `FilesystemVersionTwoCatalogStages` wraps one under the writer lock
and has no public constructor; it forwards transitions unchanged. (Until #198
deleted version 1, the version-1 `FilesystemCatalogPublisher` owned a second
one beside its own writer lock.)

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
