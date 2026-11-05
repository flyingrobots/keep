# ADR-0010: One Current Storage Format

- Status: Accepted
- Date: 2026-10-27; amended 2026-11-04 and 2026-11-05
- Owners: Keep format, initialization, publication, recovery, and retention
  boundaries
- Related issue:
  [#198](https://github.com/flyingrobots/keep/issues/198)
- Depends on: ADR-0005, ADR-0009
- Amends: ADR-0009 §Durable namespace compatibility (migration as the only
  entry into version 2)
- Supersedes: the restart root-identity requirement in
  [Retention reader root identity](retention-reader-root-identity.md) and
  [Root identity across restart](../formats/store/recovery.md#root-identity-across-restart)

## Context

Main supports two durable formats. `keep.segment-store/v1` admits the writer
lock, staging, the segment and catalog pools, and `HEAD`.
`keep.segment-store/v2` keeps those bytes unchanged and adds the `FORMAT`
marker, `reader.lock`, retention state, and reserved GC and
recovery-disposition namespaces. A one-way migration connects the two.

The two formats split Keep's capabilities between them:

- A version-2 store can be created only by migrating a version-1 store.
  Version-2 initialization is undefined.
- The content writer, `FilesystemCatalogPublisher`, and segment and catalog
  stage recovery admit only the version-1 namespace.
  `FilesystemVersionTwoAdmission` deliberately cannot be passed to them, so a
  migrated store can be read and can publish retention roots, but cannot
  ingest. General version-2 catalog publication waits on the retained-closure
  gate in [#125](https://github.com/flyingrobots/keep/issues/125).
- Every version-2 test, fixture, and crash case first builds a version-1
  store and migrates it. The migration accounts for about 10,500 lines under
  `src/adapters/store_migration`, 2,000 lines of integration tests, 1,100
  lines of repository tasks, two golden records and their source table, and the
  `KEEP-CRASH-053..073` process-death matrix.

Nothing has been released. The crate is `0.0.0` with `publish = false`. On
2026-10-27 the owner stated that no Keep store with data of value exists
anywhere. Both formats can therefore be consolidated now without a
conversion.

The migration also introduced a persistent binding between a store and its
directory. The migration intent records the root directory's device and
inode, and every later reopen and fenced snapshot compares the live root
against them (`require_root_identity`). The binding proves one thing: the
migration records were not transplanted from another directory. It proves
less than the "is this the store I meant to open" guarantee it has sometimes
been read as:

- Rename store A away and rename an intact store B into A's path. B's records
  match B's directory, so admission passes, even though the caller wanted A.
- Restore an older, internally consistent snapshot into the original
  directory. The inode is unchanged, so admission passes. The binding is not
  rollback protection.
- Restore a backup, copy a store to a new disk, or let device-mapper renumber
  a volume. Every byte is intact, yet admission refuses with
  `RootIdentityChanged`, and version 2 defines no re-admission.

## Decision

### One supported format

Keep writes and recovers exactly one storage format, the current format. The
codebase never carries a second writer. Until the first release that is also the
only format Keep reads. After it, Keep also reads released sealed segments, as
[Future format changes](#future-format-changes) defines. The format is the
version-2 record set and namespace without migration artifacts:

- the segment, catalog, and publication-head grammars, byte-for-byte as
  ADR-0005 and the version-1 pages define them;
- `FORMAT`, `reader.lock`, retention records, GC records, and
  recovery-disposition records, as the version-2 pages define them;
- no `migration.intent`, `migration.receipt`, or their `.next` stages; and
- no `FORMAT.next` in a committed store. `FORMAT.next` exists only as direct
  initialization's stage, and normal opening refuses a root that still holds it.

Its documentation is consolidated into one normative set of pages for the
Keep store format.

### Format identity

`FORMAT` remains. Its magic, version, and format-definition digest identify
the format contract, not the software release that wrote a store. Normal
opening refuses every other marker, and refuses a root without a marker once
version-1 admission is removed.

Every grammar change must change the definition digest. Today the digest does
not cover the segment, catalog, and head grammars
([#205](https://github.com/flyingrobots/keep/issues/205)), so this is a
requirement rather than a current property, and #205 must close before the first
release. Before the first release, a grammar change regenerates the affected
golden fixtures and needs no conversion. After the first release, it follows
[Future format changes](#future-format-changes).

### Direct initialization

A store is created directly by an initialization protocol under exclusive
writer authority. The protocol is a recoverable state machine:

- every partial prefix, synchronization failure, interrupted cleanup, and
  concurrent initialization attempt has exactly one result: admit, recover,
  or refuse;
- a nonempty directory that is not a recoverable initialization prefix is
  refused, never adopted;
- neither the existence of `FORMAT` nor the presence of some protocol
  directories proves completion. The protocol names one durable commit point
  before which the store does not admit;
- missing retention state never means "nothing is retained" when other
  artifacts contradict it; and
- an initialized store with no published catalog is a defined state, not an
  inferred one.

Initialization replaces migration in the same change that deletes it, so a
migrated store and an initialized store never coexist in the codebase and no
discriminator between them is needed. That change removes the migration
rows from the format definition, which changes the definition digest; the
marker it writes carries the new digest.

### Portable stores, live authority

A store is not bound to the directory where it was created. A complete store
copied while nothing was writing to it, or taken from a consistent filesystem
snapshot, opens on any admitted filesystem after normal admission. The
opening process acquires fresh local authority.

Removed: the requirement that the live root's device and inode match
coordinates persisted in the store. Initialization records no replacement
binding.

Retained without change:

- capability-relative, no-follow filesystem access;
- the writer lock and the reader fence, including lock-file identity;
- in-process checks that protocol names still identify the directories
  admitted by the same process
  ([recovery directory binding](retention-recovery-directory-binding.md)),
  and any comparison against an observation the same process made; and
- all content, closure, catalog, retention, and recovery validation.

This is not permission to copy a live store, to repair a store silently, or
to admit an incomplete store. A copy that is corrupt or incomplete refuses
exactly as the original would.

Admission alone does not detect an unintended but valid store, or an older valid
state. Detecting either requires an expectation the caller supplies and keeps
outside the state being checked, such as an exact catalog generation and digest
or a retention head digest. A record restored alongside everything else cannot
show that newer history existed.

Before the first release, opening a store therefore requires a typed choice:
the caller passes the state it expects, and a mismatch refuses, or the caller
explicitly states that it has no expectation. Neither is a default, so removing
the binding never silently removes the question. Keep does not store the
expectation. An expected state names an exact catalog generation and digest, so it
does not detect a rollback that leaves the catalog unchanged.

The choice is required by the writer's reopen
(`FilesystemStoreAdmission::reopen`), the catalog snapshot load
(`FilesystemCatalogSnapshot::load`), the fenced retention snapshot load
(`FilesystemRetentionSnapshot::load`), and both durable readers
(`DurableSnapshot::open` and `DurableStore::open`, which checks when a snapshot
is pinned). Four routes take no expectation, so their callers do not get this
check:

- recovery tools, because they examine whatever state is there;
- verification loads, for the same reason;
- initialization, including a call that resumes an interrupted one. Its
  marker-linked state admits a populated committed namespace
  ([initialization](../formats/store/initialization.md)), so that call can finish
  the cleanup of a store that already holds content; and
- the repository-task reopen, which is compiled only for tests and the
  `repository-tasks` feature and also skips platform admission.

Keep persists no store identifier. The only one, the content-derived
identifier in the migration records, is deleted with migration, and
initialization records no installation identity in its place.

### Writer before deletion

Version 1 and migration are deleted only after the single format can do
everything they do today. The order, tracked in #198, is:

1. The #125 retained-closure gate on every route that can publish a catalog,
   including recovery that finalizes a staged catalog. Changing the admission
   type that recovery accepts is not sufficient.
2. The segment and catalog writer, with segment and catalog stage recovery,
   on version-2 stores, which until step 4 are still created by migration.
   Publication of immutable catalogs and retention roots keeps running beside
   readers. Only GC takes the reader fence exclusively, after writer
   authority. This unit defines the input envelope it supports and does not
   claim the segment rollover in [#121](https://github.com/flyingrobots/keep/issues/121).
3. Populated test fixtures are built through that writer, so that test code is
   not a second implementation of store construction.
4. Direct initialization and its crash matrix replace migration, and
   migration is deleted in the same change. Before that change merges, one
   integrated public-API sequence passes: initialize, ingest, reopen,
   reconstruct exact bytes, retain, publish a second catalog, interrupt
   publication, recover, and read both retained contents. The sequence also
   includes a candidate catalog that would lose retained content, which must
   refuse with its evidence intact. The change also carries laws for two copies:
   a copy torn by a concurrent writer never returns bytes other than those its
   identities name, and an older valid snapshot refuses when the caller passes
   newer expected state.
5. Version-1 admission is deleted and the format pages are consolidated.

One departure from this order is recorded. Step 2's stage recovery reserved two
process-death rows that are not implemented: `KEEP-CRASH-079`, death during
recovery's stage-completion link, and `KEEP-CRASH-080`, death during gated
finalization's `HEAD` replacement. The order placed them before step 5, and
version 1 was deleted without them. The owner accepted that on 2026-11-05: the
rows never existed on `main`, nothing is released, and both routes refuse when
their state is ambiguous. Until they land, process death during recovery, and
the gate running again when a finalization is retried after one, are untested.
[#221](https://github.com/flyingrobots/keep/issues/221) tracks the rows and must
close before the first release, as #205 must.

Step 4's torn-copy law is met narrowly. The laws enumerate every mix of two
quiescent published states, and a half-length copy of any one file over the
newer state. A copy that holds a writer's in-flight stage, a file cut at another
length, and a tear over the older state are untested.

Retired requirement and crash identifiers are never reused. A retirement
ledger maps each one to its last evidence and states whether its obligation
disappeared with migration or moved to the single format.

### Future format changes

After the first release, Keep reads old data and writes the current format. No
format change requires rewriting a store's segments, except a change forced by
a broken integrity primitive, which needs its own decision.

Sealed segments are immutable, and each records its own magic and
`format_version`. Keep therefore retains a frozen decoder for every released
segment format and selects it per segment. Each frozen decoder keeps its golden
fixtures and fuzz targets in every release. An unknown or unlisted version is
still refused. New data is always written in the current format. Released
segment formats stay readable indefinitely. A support window that retires a
decoder would need a later decision, because it could strand a store that has
not been rewritten.

Catalog, publication-head, retention, and other mutable metadata has one
current version. A release that changes it, or that changes the definition
digest, ships a separately invoked metadata upgrade, which also rewrites the
`FORMAT` marker. It holds the writer lock from before its first read until its
completion is durable, and refuses if the lock is held, so it sees one consistent
source and blocks writers by design. Normal opening never converts a store, and it refuses a marker
other than the current one until that upgrade has run.

Reads never upgrade. A read holds no writer authority, must behave the same on
a full disk or a read-only mount, and must not change bytes that backups and
synchronization observe. It would also leave unread data behind, so it could
not retire a decoder.

Rewriting old segments into the current format is optional. It is one explicit
operation under writer authority that is resumable, can be limited by bytes or
time, and verifies every content identity before an old segment becomes
removable. The operator chooses when it runs, including never. Keep stores no
per-store upgrade policy. An application may schedule the operation, for
example as a background job, through a public interface that reports progress
and accepts cancellation. Keep owns no scheduler or user interface.

Keep reports how many segments and bytes each format holds, so an application
can show that state to an operator. Format is physical layout, so it is not
reported per content identity and is not a warning about content. Content in a
frozen format returns exactly the same bytes as content in the current format.

These principles bind every future upgrade:

- content identity never depends on the format that stores it;
- every crash leaves a store that admits or refuses, never one that silently
  loses content;
- a store can be read throughout segment rewriting; and
- old segments are removed only through GC's exclusive reader fence.

Open questions, to be answered by a minimal upgrade tool built after #198
lands rather than specified here:

- how long a metadata upgrade takes and how much space it needs for a store
  with many files, since catalog size grows with the number of files;
- whether a metadata upgrade writes a new store or converts in place, and what
  crash matrix that choice requires;
- what the operation's resume state is, and where it lives; and
- whether operators want a stored schedule or a recorded order, such as most
  read segments first. Any such hint must not make a read write to the store.

## Alternatives considered

- **Keep both formats and the migration.** Rejected. Every capability would be
  built and tested twice. A version-2 store would stay unable to ingest until
  that duplicated work landed.
- **Delete version 1 first.** Rejected. Keep would have no durable writer and
  no segment or catalog recovery.
- **Define the format in GraphQL and generate codecs with Wesley.** Rejected.
  Wesley's breaking-change check follows API rules, which call an added field
  safe and do not report field reordering. A stored record breaks under both
  changes, so the safety would still have to come from a Keep check.
  `definition.tsv` already gives Keep a declarative, hashed format definition.
  The approach would add a pre-1.0 build dependency, an IR change, and a
  generator for no stronger guarantee.
- **Keep the persistent root binding and add a rebind command.** Rejected. It
  adds a durable mutation protocol while answering a narrower question than
  the threat it is cited against. A rebind that checks internal consistency
  cannot show that the store was intended or current.
- **Rebind automatically on open.** Rejected. It silently rewrites the
  evidence the check exists to enforce.
- **Bind the store to the filesystem UUID instead of the device number.**
  Rejected. It survives renumbering but still refuses backups and copies, and
  it has the same limits for an intended or current store.
- **Copy the whole store for every format change.** Rejected. Every byte is read,
  hashed, and rewritten, the store needs space for two copies, and writers stay
  blocked throughout. That cost grows with the store for a change that sealed
  segments do not need.
- **Read and write every released format.** Rejected. Every write path and crash
  matrix would multiply per format.
- **Upgrade on read.** Rejected. See [Future format
  changes](#future-format-changes).
- **A stored per-store upgrade policy.** Rejected for now. It would add states to
  test without evidence that running an explicit operation is insufficient.
- **Mark content stored in an older format.** Rejected. It would expose physical
  layout as a property of content, and content can share or span segments.
- **Self-describing records with tagged, skippable fields.** Rejected. They
  give up fixed offsets and bounds checked before allocation, and they
  conflict with refusing unknown and noncanonical content.
- **Scrub the whole store on every open, because stores are portable.**
  Rejected. Admission, verification of requested reads, and complete backup
  verification stay distinct operations.

## Consequences

- The definition digest changes when initialization replaces migration and
  the migration rows are removed. Golden fixtures are regenerated then.
- The units in #198 land as one integration. Its laws and evidence, not this
  record, establish that they shipped.
- Removing the binding relaxes `RootIdentityChanged` on reopen and on fenced
  snapshots. Live replacement checks keep their existing refusals.
- Portability is accepted through these laws:
  - a valid quiescent copy opens;
  - a corrupt or incomplete copy refuses as the original would;
  - a replaced lock or protocol directory during an operation still refuses;
  - a copied store and its original, each written independently, diverge
    with independent locks while unchanged content keeps its identity; and
  - backup round trip: close, copy, make the original unavailable, reopen and
    verify the copy, then interrupt a new write on the copy and recover it.

  Independent copies are not replication and are never described as
  coordinated.
- Frozen segment decoders stay in the codebase permanently. This narrows the
  owner's original one-format policy of 2026-10-26, in exchange for upgrades that
  do not rewrite a whole store. Each released segment format is a permanent
  commitment, so a format ships only when Keep will support it indefinitely.
- A broken integrity primitive still forces a rewrite, because a frozen decoder
  cannot repair it.
- #190 still applies: segment and catalog stage recovery moves to the single
  format rather than being deleted, and its recovery outcomes move with it.
  The typed migration refusals in #191 are deleted with migration.
- [#108](https://github.com/flyingrobots/keep/issues/108) (migration
  recovery) and migration-specific requirement rows are retired through the
  ledger, not reused.
- Security: removing a persistent binding does not widen filesystem
  authority. All access stays capability-relative and no-follow. The stronger
  wrong-store and rollback claims hold only for a caller that passes expected
  state, and every caller of an open that takes the choice must make it.
