# ADR-0009: Retention Roots, Release, and GC Liveness

- Status: Accepted
- Date: 2026-07-29
- Owners: Keep retention, catalog, recovery, verification, and GC boundaries
- Related issue:
  [#18](https://github.com/flyingrobots/keep/issues/18)
- Depends on: ADR-0002, ADR-0005, issues #3, #16, and #17

## Context

Keep separates logical content identity from layouts, representations, and
physical locations. Its durable segment store publishes one immutable catalog
generation at a time and refuses to treat file existence as visibility,
durability, or recovery evidence. Those decisions do not yet say which
published material external policy requires Keep to preserve.

Keep cannot infer that policy. Echo, Git, Graft, paths, timestamps, caller
identity, catalog membership, and recent access may be meaningful to an
application, but none establishes storage liveness. Conversely, removing one
application reference does not prove that no other policy still requires the
same bytes.

Garbage collection needs a complete, stable answer to a narrower question:
which verified physical records must remain reachable for the exact retention
state being collected? That answer must remain deterministic while retention
or catalog publication continues. Missing or corrupt evidence must stop
collection rather than shrink the live set.

This ADR defines the authority and liveness model. It does not define the
versioned byte grammar or implement retention transitions, verification, or
garbage collection.

## Decision

External policy supplies explicit retention namespaces and reconstruction
anchors. Keep validates, versions, and durably publishes that physical
retention state. Keep never assigns application-level meaning to it.

### Durable namespace compatibility

Retention and GC state do not extend `keep.segment-store/v1`. Its exact root
shape admits only the existing writer lock, staging, immutable pools, and
`HEAD`; adding another entry would correctly make a version-1 store ambiguous.
Version-1 admission continues to refuse every unknown retention, GC, or
migration artifact.

Issue #19 must first specify `keep.segment-store/v2` as a successor durable
format. Version 2 preserves the admitted version-1 segment, catalog, and head
bytes, but defines a new exact root shape, version marker, retention
generations and head, GC intent and receipt state, recovery-disposition
evidence, bounds, checksums, golden fixtures, and recovery table. No
implementation may write those artifacts before that specification and its
parser, corruption, fuzz, and crash evidence exist.

Migration is a one-way explicit migration under the existing exclusive writer
lock:

1. Open and completely recover the version-1 store; refuse every fixed stage,
   orphan ambiguity, corrupt artifact, and noncanonical root entry.
2. Durably publish a canonical migration intent that binds the exact version-1
   head, catalog, pool, root identity, and target version.
3. Create and synchronize only the canonical version-2 namespaces and initial
   empty retention and idle GC state.
4. Reopen and verify the complete version-2 view, synchronize the root, and
   durably publish the migration receipt.

Process death before the intent leaves a version-1 store. Once any migration
artifact is durable, version-1 admission refuses and only the version-2
migration recovery boundary may proceed. It admits each documented partial
migration prefix, revalidates the intent and existing bytes, and idempotently
continues; unknown entries, substitution, conflicting receipts, or a changed
version-1 coordinate are unrecoverable ambiguity. Migration never deletes or
rewrites admitted version-1 immutable bytes and provides no automatic
downgrade.

### Namespace authority

`RetentionNamespace` is an opaque, validated, canonical logical identifier
supplied by the caller. It is not a filesystem path, account, process, lock
owner, or application identity. Its future wire format must be versioned,
bounded, byte-preserving, and unambiguous on every admitted platform. There is
no implicit namespace and no namespace inferred from ambient state.

Changing a namespace identifier creates a distinct retention authority.
Namespaces do not inherit, alias, or shadow one another. A namespace is a
partition for compare-and-swap updates; it does not participate in content
identity.

The version-2 format defines one fixed maximum admitted namespace count. The
count includes empty generations because namespace identity is never deleted
or reused. Admission of a previously absent namespace computes the attempted
count with checked arithmetic and refuses above the maximum before any
namespace-generation or manifest bytes are staged. The typed refusal preserves
the maximum and attempted counts. Transitions of already admitted namespaces
remain available at capacity; increasing the ceiling or reclaiming tombstones
requires a successor protocol with an ABA-safe migration.

File existence, catalog membership, and recent access are not retention
evidence. Keep does not infer semantic liveness from Echo, Git, Graft, paths,
timestamps, or caller identity.

### Roots and canonical closure

One retained root is a reconstruction anchor containing an exact `BlobId` and
`LayoutId`. The pair states which logical bytes must remain reconstructable
and which admitted reconstruction plan proves that result. Keep verifies that
the named layout reconstructs the named blob before admitting the anchor.

A root does not name a physical path, segment coordinate, catalog entry, or
all possible representations of a blob. Retention does not alter the logical
identity of a blob, layout, chunk, representation, or catalog record.

Each namespace generation contains one exact realization-profile coordinate
and one complete, sorted, duplicate-free set of reconstruction anchors. The
transition boundary rejects noncanonical ordering and duplicates instead of
silently normalizing caller input.

Keep derives a canonical closure for every anchor against one pinned,
completely verified catalog generation. The closure contains the anchor, its
layout record, every chunk record required by that layout, and the exact
supporting representation and verification metadata required to reconstruct
and authenticate the blob. Shared members occur once in canonical typed-ID
order.

The anchor defines a logical closure, not one permanent physical realization.
When a pinned catalog offers multiple admissible physical representations or
locations for a closure member, Keep resolves them through one registered,
versioned `RetentionRealizationProfile`. The profile defines a canonical total
order and an exact nonzero witness count that must remain available for every
logical closure member. It is explicit input to closure admission, never a
serializer default, filesystem order, or caller callback.

An admitted `RetentionRealizationProfileId` binds the exact profile identity,
version, and digest of its canonical definition bytes. Namespace generations,
liveness snapshots, and receipts store that complete coordinate. Admission and
revalidation resolve only the exact coordinate; an unavailable or mismatched
profile refuses before traversal or materialization. A registry entry with the
same human-readable name but different version, digest, or definition cannot
substitute for the retained profile.

Every candidate considered by the profile is admitted before selection. An
unsupported profile, missing witness, conflicting catalog claim, corrupt
candidate, or unresolved ordering tie refuses the snapshot. A valid candidate
cannot hide corrupt or ambiguous evidence beside it. The selected physical
realization is recorded in transition or snapshot evidence; it does not become
part of the root, `BlobId`, `LayoutId`, or other content identity. Compaction
may therefore change the selected realization only through a newly verified
catalog and liveness snapshot.

Closure traversal is deterministic, explicitly bounded, cycle-safe, and
fail-closed. Each supported protocol version defines implementation-enforced
hard ceilings for roots, nodes, depth, encoded bytes, and physical bytes
inspected. These ceilings are reviewed constants, not ambient configuration,
serializer defaults, or caller policy.

The caller supplies typed limits no greater than those ceilings and may choose
lower bounds. Keep validates every limit and their cross-field relationships
before traversal or materialization begins. A request above a ceiling is a
typed refusal that preserves the field, maximum, and observed value.

A visited set prevents repeated traversal and cycles; checked counters enforce
the admitted limits throughout the operation. Exceeding a bound, observing an
unknown mandatory edge, or finding a missing or corrupt closure member refuses
the complete transition or GC plan. Keep never drops the unproved member and
continues with a smaller live set.

### Generation-checked transitions

Every namespace has an immutable `RootGeneration`. A transition supplies:

1. the exact `RetentionNamespace`;
2. an expected state of either absent or one exact `RootGeneration`;
3. the complete candidate anchor set; and
4. one exact `RetentionRealizationProfileId`; and
5. explicit closure-admission limits.

Retention publication holds the same exclusive store writer authority used by
catalog publication and recovery. It acquires that authority before pinning
the catalog and retains it through closure verification, immutable generation
publication, retention-head replacement, durability synchronization, and
receipt construction. The bounded closure work is deliberately inside this
critical section so catalog evidence cannot change between proof and commit;
no application callback or external policy evaluation occurs while the
authority is held.

The executor reads one verified retention head, compares the namespace state,
computes and verifies the candidate closure against one pinned catalog
generation, and only then stages publication. A stale update fails with the
expected and observed generations. The initial `RootGeneration` is one. Every
later generation uses checked arithmetic and is exactly one greater than its
observed predecessor.

Publishing an empty anchor set remains a namespace transition: the global
manifest retains the namespace and names its new empty `RootGeneration`.
Version 1 does not delete or reuse namespace identity. This preserves
generation continuity and prevents an old absent-state request from becoming
valid again after release.

Retention publication follows the durable store law: immutable generation
bytes are written, verified, synchronized, linked without replacement, and
made durable before a head can name them. Publication returns an explicit
fallible receipt and never relies on `Drop`.

One immutable global retention manifest names the complete sorted map from
every admitted `RetentionNamespace` to its exact `RootGeneration`. Each
manifest carries a `LivenessGeneration`. The first manifest generation is one,
and every successor is exactly one greater under checked arithmetic. A
namespace transition publishes a new namespace generation and a new global
manifest, then atomically replaces the retention head under writer authority.
This top-level generation prevents a GC scan from missing a namespace created
concurrently with enumeration.

Retrying an already committed byte-identical transition may return a typed
already-committed outcome with the original evidence. Any different candidate
against the old expected generation is stale and must not be merged
implicitly.

### State meanings

The protocol has two disjoint state families. Logical retention states apply
to candidate bytes, namespace generations, and reconstruction anchors:

- **Staged candidate:** immutable candidate generation bytes exist, but no
  retention head names them. They are not retention authority.
- **Retained generation or anchor:** the current global manifest names the
  namespace generation, and that generation contains the reconstruction
  anchor with a completely verified closure.
- **Released anchor:** a successfully published successor generation omits an
  anchor that its predecessor contained. Release is a logical transition, not
  a physical deletion claim.

Physical GC states apply only to physical material:

- **Orphaned material:** the material is unreachable from the selected immutable
  liveness snapshot. It may still be protected by another current snapshot,
  an active reader, recovery evidence, or a later publication.
- **Collectible material:** a deterministic GC plan proves the material
  unreachable from its immutable liveness snapshot and catalog snapshot, and
  execution has revalidated every required generation and reader, recovery,
  and publication safety fence.

These states are explicit evidence postures, not path properties. Every state
transition requires the evidence defined above and cannot be inferred from
existence, age, recent access, or absence from one catalog.

### Recovery evidence disposition

A completely verified orphan left by interrupted publication remains
recovery-protected until an explicit finalize-or-retire transition disposes of
that authority. Root absence, catalog absence, age, and successful
reconstruction do not release it.

Finalization consumes the evidence through the existing recovery completion or
publication protocol and preserves it as visible or recoverable state.
Retirement is a separate writer-authorized transition that proves the exact
artifact is not named by `HEAD`, a fixed recovery stage, a pending publication,
a retained closure, or an active reader. Ambiguous, changed, or corrupt
evidence cannot receive a retirement disposition.

A successful retirement durably publishes a `RecoveryDispositionReceipt`
before GC planning may classify the artifact as unreachable. The receipt binds
the artifact identity and digest, its admitted recovery classification, the
observed head, catalog, retention, and reader-safety coordinates, and the exact
retirement decision. Process death before durable receipt publication leaves
the artifact recovery-protected; an exact retry either finishes that
publication or reports the changed evidence.

GC cannot infer recovery release. It admits only the exact durable disposition
receipt and revalidates every bound coordinate under writer authority. An
absent, stale, mismatched, or superseded receipt keeps the material protected.

### Release and grace

Release removes an anchor from a newly published namespace generation. It
does not promise immediate physical erasure, secure deletion, or deletion at
all. Shared closure members remain live while any retained anchor reaches
them.

Keep does not implement clock-based grace. A grace policy that must preserve
bytes is represented by an explicit retained anchor in a dedicated namespace
or generation. External policy may later publish its release, but wall-clock
expiry, lease-owner death, and process disappearance never mutate retention
state by inference.

### Immutable liveness snapshots

A GC planner first admits one immutable liveness snapshot. The snapshot binds:

- the exact global retention-manifest generation and digest;
- the complete sorted namespace-to-root-generation map;
- every verified reconstruction anchor and canonical closure;
- every exact realization profile identity, version, and digest;
- the exact catalog generation used to resolve physical records; and
- the explicit traversal and materialization limits used to establish it.

Planning is a pure deterministic comparison between that snapshot and one
bounded physical inventory. It classifies material as live, unreachable,
corrupt, ambiguous, recovery-protected, reader-protected, or already retired.
Only unreachable material may become a collectible candidate.

The physical retirement unit is one whole immutable segment, never an
individual record span. A segment is live whenever any contained record is
live, reader-protected, recovery-protected, corrupt, or ambiguous. An
unreachable record inside that segment cannot authorize partial deletion or
make the container collectible.

Reclaiming unreachable records from a mixed segment requires compaction to
copy and verify every live record into new immutable segments, publish and
synchronize a catalog successor that names those replacements, and revalidate
the resulting view. Only after that durably published compaction successor and
the required reader and recovery fences may the complete old segment enter a
retirement plan.

No segment is an executable retirement candidate until the current catalog
names no candidate retirement segment. If the current catalog still contains
any record in a planned segment, GC first constructs a canonical successor
that either omits unreachable records or names verified replacements for live
records. It publishes that successor through the complete catalog protocol.
The catalog successor is durable before any old segment unlink.

Execution then reopens the exact current snapshot and proves every retirement
segment absent from it. A catalog-generation check alone is insufficient.
`HEAD` never names a catalog with a missing segment; process death before
successor publication leaves every old segment reachable, and process death
after publication leaves the old segments as intact retired candidates.

Planning is observational and carries no mutation authority. Execution must
acquire the same exclusive writer authority used by catalog publication and
recovery, then retain it from revalidation through physical mutation,
durability synchronization, and receipt construction. No application callback
or external policy evaluation occurs while that authority is held.

After acquiring writer authority, execution revalidates the retention head,
catalog generation, candidate identity, exact realization-profile coordinate,
and all safety fences before physical mutation. Any changed generation,
unavailable or mismatched profile, missing evidence, corrupt member, ambiguous
alias, or active protection invalidates the plan. A plan never carries
authority across a changed liveness or catalog view.

### GC execution and crash recovery

Before the first physical retirement, the executor durably publishes one
canonical, bounded `GcRetirementIntent`. The intent binds:

- a checked GC generation and the complete plan digest;
- every retention, root, realization-profile, catalog, reader-safety, and
  recovery-disposition coordinate used for planning;
- the current catalog successor that names none of the candidates;
- the admitted segment-pool identity; and
- the exact candidate segment digests, lengths, and verification evidence in
  canonical segment-digest order.

The executor writes, flushes, and synchronizes the intent, then synchronizes
its parent before any unlink. An intent is immutable. A second plan cannot
replace, merge with, or execute beside it.

While retaining writer authority, execution visits candidates in the intent's
canonical segment-digest order. It reopens each present candidate without
following links, verifies its exact identity and digest, unlinks only that
entry, and synchronizes the segment-pool directory after every unlink before
advancing. Unlinks are never batched behind one final synchronization.

After every candidate is verified absent and the segment pool is synchronized,
the executor durably publishes a canonical `GcRetirementReceipt`, advances the
GC state from the exact intent to that receipt, synchronizes the GC namespace
and store root, and only then returns success. The receipt binds the intent
digest, retired segment set, observed post-retirement pool state, and every
completed synchronization.

Store admission treats a retained intent as recovery-required and excludes
catalog publication, retention transitions, and another GC execution until it
is resolved. Restart admits only these crash states:

- no durable intent and every candidate still present: no retirement
  authority exists;
- a durable intent with every candidate present: execution may begin;
- a durable intent with one exact candidate prefix absent: authorized partial
  retirement, including an unlink whose directory synchronization was
  interrupted;
- a durable intent with every candidate absent but no receipt: completion may
  be published; or
- the exact durable receipt: execution is complete.

An absent out-of-order candidate, substituted candidate, changed pool,
unadmitted intent, conflicting receipt, or changed bound coordinate is
unrecoverable ambiguity. Recovery does not guess which deletion occurred.

Idempotent recovery reacquires writer authority, re-admits the exact intent,
revalidates every bound coordinate, accepts only the absent canonical prefix,
continues at the first present candidate, repeats required directory
synchronization, and publishes or returns the exact completion receipt. It
never treats unexplained absence as successful retirement.

### Evidence and nonclaims

A successful retention transition returns a consequential, `#[must_use]`
receipt containing at least:

- the namespace and expected and observed generations;
- the committed root generation and global manifest generation;
- the exact realization profile identity, version, and digest;
- canonical anchor-set and closure digests;
- the catalog generation used for closure verification; and
- the durable publication outcome.

Its physical claim is narrow: at commit time, the named retained anchors had
complete verified closures reachable through the named catalog generation,
and the named retention generation was durably published.

The receipt does not prove application-level meaning, causal ownership,
exclusive ownership, future retention after another generation commits,
power-loss behavior beyond the admitted platform contract, or secure erasure.

A release receipt proves only the committed successor state. A liveness
snapshot proves only the exact bounded view it names. A GC receipt must state
which snapshot and revalidation evidence authorized each retirement; it does
not reinterpret storage evidence as application policy.

## Alternatives considered

### Git refs

Rejected. Git refs import Git naming, repository, process, and update policy
into Keep's lower storage boundary. They do not prove a canonical closure,
catalog generation, or durable Keep publication, and they make liveness depend
on an integration technology.

### Leases

Rejected as Keep authority. Expiry depends on clocks, renewal scheduling,
owner identity, and failure-detector policy. An external lease system may
decide when to request an explicit generation transition, but Keep does not
infer release from lease expiry or owner disappearance.

### Reference counts

Rejected. Counts conflate policy roots with derived reachability, are
difficult to update atomically across shared or cyclic graphs, and can drift
after interruption. A count of zero is not proof that a complete verified
root set reaches no material.

### Tracing from explicit roots

Accepted only with immutable versioned roots and fail-closed closure
verification. Unversioned tracing over mutable namespaces or catalogs was
rejected because one scan could combine incompatible generations or miss a
concurrent namespace.

### Caller-supplied physical closure

Rejected. A caller may supply reconstruction anchors and limits, but Keep
derives and verifies the closure. Trusting paths, segment offsets, or an
incomplete caller list would allow missing or corrupt material to disappear
from the live set.

### Retain every representation of a blob

Rejected. It would make physical optimization and compaction impossible
without changing retention policy. The root retains one verified
reconstruction closure; separately retained representations require explicit
anchors or a future typed policy extension.

## Consequences

- Issue #19 must implement canonical namespace and manifest formats,
  generation-checked transitions, typed evidence, golden fixtures, and crash
  recovery consistent with this decision.
- Issue #20 must report the exact verification depth established for roots,
  closures, manifests, catalogs, and physical members.
- Issue #21 must plan from an immutable liveness snapshot and revalidate every
  generation and safety fence before retiring material.
- Application integrations remain responsible for deciding which
  reconstruction anchors matter. Keep accepts explicit policy but does not
  invent it.
- Retention metadata and closure verification add storage, I/O, and
  synchronization costs. Implementations must bound and measure them rather
  than weakening verification.
- Released and orphaned material may occupy space indefinitely until a
  separately verified GC execution proves it collectible.
- This accepted decision is not implementation evidence. Living product
  documentation must continue to describe retention and GC as planned until
  their executable contracts ship.
