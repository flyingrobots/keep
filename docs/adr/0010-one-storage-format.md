# ADR-0010: One Storage Format

- Status: Accepted
- Date: 2026-10-27
- Owners: Keep format, initialization, publication, recovery, and retention
  boundaries
- Related issue:
  [#198](https://github.com/flyingrobots/keep/issues/198)
- Depends on: ADR-0005, ADR-0009
- Amends: ADR-0009 §Durable namespace compatibility (migration as the only
  entry into version 2)
- Supersedes: the restart root-identity requirement in
  [Retention reader root identity](retention-reader-root-identity.md) and
  [Root identity across restart](../formats/segment-store-v2/recovery.md#root-identity-across-restart)

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

Keep admits, writes, and recovers exactly one storage format. The codebase
never carries a second runtime format alongside it. The format is the
version-2 record set and namespace without migration artifacts:

- the segment, catalog, and publication-head grammars, byte-for-byte as
  ADR-0005 and the version-1 pages define them;
- `FORMAT`, `reader.lock`, retention records, GC records, and
  recovery-disposition records, as the version-2 pages define them;
- no `migration.intent`, `migration.receipt`, their `.next` stages, or
  `FORMAT.next`.

Its documentation is consolidated into one normative set of pages for the
Keep store format.

### Format identity

`FORMAT` remains. Its magic, version, and format-definition digest identify
the format contract, not the software release that wrote a store. Normal
opening refuses every other marker, and refuses a root without a marker once
version-1 admission is removed.

Every grammar change changes the definition digest. Before the first release,
such a change regenerates the affected golden fixtures and needs no
conversion. After the first release, it follows [Future format
changes](#future-format-changes).

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

Keep does not claim that admission detects an unintended but valid store, or
an older valid state. Detecting either requires an expectation the caller
supplies and keeps outside the state being checked, such as an exact catalog
generation and digest or a retention head digest. A record restored alongside
everything else cannot show that newer history existed. Keep may later offer
such an expected-state check, but only as an explicit caller input.

The store identifier stays a deterministic, content-derived value. It is not
a unique installation identity, and two stores with identical initial states
may share it.

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
   refuse with its evidence intact.
5. Version-1 admission is deleted and the format pages are consolidated.

Retired requirement and crash identifiers are never reused. A retirement
ledger maps each one to its last evidence and states whether its obligation
disappeared with migration or moved to the single format.

### Future format changes

After the first release, a release that changes the format ships a separately
invoked upgrade tool. Normal opening never converts a store.

The tool reads the source store with the previous release's code, pinned to
its tag, and writes a new store with the current code. It never modifies the
source. It checks every content identity and every retention root against
the source. The destination does not admit until the tool's completion
record is durable. The tool's crash protocol is defined with it, and its test
upgrades a fixture store from every prior release.

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
- **Upgrade in place, as the migration does.** Rejected for future format
  changes. In-place conversion needs a crash matrix over every partial prefix.
  Copying into a new store leaves the source intact at every crash point, at
  the cost of space for two stores during the upgrade.
- **Self-describing records with tagged, skippable fields.** Rejected. They
  give up fixed offsets and bounds checked before allocation, and they
  conflict with refusing unknown and noncanonical content.
- **Scrub the whole store on every open, because stores are portable.**
  Rejected. Admission, verification of requested reads, and complete backup
  verification stay distinct operations.

## Consequences

- The definition digest changes when initialization replaces migration and
  the migration rows are removed. Golden fixtures are regenerated then.
- Until the units in #198 land, main still enforces the persistent root
  binding and still creates version-2 stores only by migration. This ADR is a
  protocol commitment, not evidence that the change has shipped.
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
- #190 still applies: segment and catalog stage recovery moves to the single
  format rather than being deleted, and its recovery outcomes move with it.
  The typed migration refusals in #191 are deleted with migration.
- [#108](https://github.com/flyingrobots/keep/issues/108) (migration
  recovery) and migration-specific requirement rows are retired through the
  ledger, not reused.
- Security: removing a persistent binding does not widen filesystem
  authority. All access stays capability-relative and no-follow. The stronger
  wrong-store and rollback claims are now stated as unsupported instead of
  implied.
