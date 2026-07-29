# Recovery and Platform Contract

This page owns recovery classification, deterministic refusal precedence, and
the minimum filesystem semantics for `keep.segment-store/v1`.

## Recovery

Opening is read-only. It returns an admitted reader snapshot, an uninitialized
state, a typed recovery inventory and plan, or a refusal. It never changes
durable bytes.

Recovery inventory reads only protocol-owned canonical names. Across the
store root, `staging`, `segments`, and `catalogs`, recovery counts entries
before retaining or sorting their names. The scan stops with a typed limit
refusal when the count exceeds `MAX_RECOVERY_INVENTORY_ENTRY_COUNT`; because
it stops on the first excess entry, the refusal reports an observed-at-least
count of `2,097,153`, not a host-order-dependent exact total. A configured
limit may be lower but never higher.

After the count is admitted, recovery sorts names by raw canonical bytes and
classifies the namespace grammar before opening artifact bytes. Unknown names,
symlinks, noncanonical pool coordinates, or multiple fixed-name stages are
unrecoverable ambiguity. Content verification then classifies each artifact;
a head that cannot be proven atomic is also unrecoverable ambiguity.

The current public `read_recovery_inventory` slice implements this fixed-order
count-before-retain orchestration through a read-only storage port. It enforces
the configured and protocol ceilings, count stability, duplicate refusal, and
namespace-plus-raw-byte ordering. `FilesystemRecoveryInventoryReader` pins the
root and three no-follow child directories, bounds each scan by the remaining
global budget, preserves raw Linux name bytes, and verifies child-directory
identity before and after inventory. `classify_recovery_names` requires the
four initialized root entries, types each fixed name and immutable-pool
coordinate, and refuses an unknown or conflicting name without artifact I/O.
`fingerprint_recovery_stage` then reads a fixed stage through a zero-allocation
bounded stream, refuses metadata or observed bytes above the name-selected
maximum, and returns its exact observed length and
`KEEP:RECOVERY:STAGE\0` fingerprint.

New stores obtain writer authority through
`FilesystemPlatformAdmission::initialize`. A stable published store reacquires
authority through `FilesystemPlatformAdmission::reopen`, which performs no
protocol mutation, admits the production platform, acquires the existing
writer lock, and requires exactly `writer.lock`, `staging`, `segments`,
`catalogs`, and a regular `HEAD` in the root. Missing or additional root
evidence remains a typed namespace refusal; content-level head, catalog, and
segment verification stays at the publisher and restart boundaries.

`FilesystemRecoveryInventoryReader::fingerprint_stage` binds that stream to
the pinned root or staging capability, opens without following links, admits
only regular files, and refuses entry replacement or length drift after
reading. `classify_recovery_segment_stage` classifies complete caller-supplied
stage bytes as a validated reusable prefix, a complete admitted segment, or an
exact truncation and preserves complete-looking corruption as a typed refusal.
Catalog- and next-head-stage classifiers likewise distinguish exact truncation
from complete canonical bytes. Transitive publication-view admission and
filesystem-streaming semantic classification remain unimplemented.
`admit_recovery_stage_bytes` first requires the canonical-name stage, exact
length, and recomputed stage fingerprint to match prior observation evidence;
only `assess_recovery_stage` may dispatch those admitted bytes to a semantic
classifier. Matching evidence does not convert corrupt bytes into lawful
content.
`plan_recovery_stage_discard` admits only an exact truncation assessment and
retains both its evidence and typed truncation reason. The semantic
`execute_recovery_stage_discard` port refuses evidence drift before mutation,
treats an absent canonical name as an idempotent input, synchronizes the
name-selected parent in either success case, and returns a receipt only after
that synchronization. `FilesystemRecoveryStageDiscarder` admits the supported
platform, retains the root and `writer.lock` locks, pins all protocol
directories, reopens the stage without following links, bounds and verifies
the complete fingerprint, refuses namespace or entry replacement, unlinks only
an exact evidence match, and synchronizes the typed `staging` or root parent.

The sole admissible duplicate digest is one fixed staging name and its exact
digest-derived pool name after a link transition. Recovery admits that pair
only after complete byte-for-byte verification proves the stage, pool entry,
declared digest, physical pool name, and artifact kind agree. Any third name,
wrong namespace, or disagreement is unrecoverable ambiguity.

The required classes are:

- **Reusable staged material:** a valid header followed by zero or more
  complete records and no seal. Explicit recovery may resume it without
  rewriting the admitted prefix.
- **Valid orphan:** a completely verified sealed segment or catalog not
  selected by the current head. Recovery preserves it and reports why it is
  invisible.
- **Truncated tail:** an empty stage, partial header, partial record, partial
  checksum, partial seal, or partial catalog/head. It is never truncated,
  padded, or treated as complete automatically.
- **Corrupt sealed state:** complete-looking immutable framing whose declared
  values, checksum, digest, identity, or physical name disagree. Recovery
  refuses it.
- **Stale generation:** a candidate head or catalog that does not extend the
  verified current generation and predecessor exactly. Recovery reports
  expected and observed coordinates.
- **Unrecoverable ambiguity:** evidence permits more than one incompatible
  history or cannot prove one lawful current state. Recovery refuses without
  choosing.

An explicit recovery executor may resume a reusable stage, complete a durable
stage into its immutable pool, preserve an orphan, explicitly discard a named
truncated stage, or finalize one fully verified next-generation head using the
same generation comparison and publication steps. Recovery itself uses the
same crash points and must be idempotent. It may not silently promote the
newest artifact, truncate to the last plausible boundary, rewrite a checksum,
delete a valid orphan, or select by timestamp.

## Process-death crash matrix

Run the repository-owned matrix:

```bash
cargo xtask durability-crash-matrix
```

The command executes the three ordered positions for each stable
`KEEP-CRASH-001`–`KEEP-CRASH-035` point: 105 canonical cases. Each case owns a
fresh filesystem store and an isolated child process group. The child retains
the writer lock and any open staged artifact while it executes the production
initialization, segment-writing, catalog-publication, or recovery-discard
protocol. A fault-injecting port decorator sends one readiness byte at the
selected semantic boundary and waits on the retained connection. The parent
keeps the accepted socket open, sends `SIGKILL` to the complete process group,
reaps the child, and only then begins restart inspection. A ten-second
deadline bounds failure handling; successful synchronization does not depend
on elapsed time, sleeps, or test ordering.

The production protocol driver and expected-state model are separate
implementations. The driver delegates every target mutation to the ordinary
filesystem adapters. Repository-only partial-write methods share the same
catalog and head stage writers and stop at deterministic proper prefixes;
they do not hand-construct a replacement namespace. Restart inspection proves
the complete path set and exact Golden File Worldline bytes. It additionally
verifies hard-link identity at link transitions, reacquires `writer.lock`
after process death, classifies `current.seg`, `current.cat`, and `head.next`
through the production recovery classifiers, admits immutable segment and
catalog bytes through the production decoders, and loads the exact
generation-1 snapshot when `HEAD` is present. That snapshot must expose the
one-zero chunk with the exact payload `00`.

For byte writes, a `during` case retains a deterministic proper prefix and its
open file handle at termination. Atomic create, hard-link, unlink, and rename
operations admit their completed namespace state because no torn namespace
operation is representable through the documented filesystem contract.
Flush and synchronization cases preserve the exact application-visible bytes
on both sides of the durability call. The matrix proves process-death
recovery; it does not simulate host power loss, torn media writes, or a
filesystem that violates the admitted atomicity contract.

CI runs the complete command once through the debug `xtask` binary and once
through an optimized `xtask` binary. To isolate one coordinate locally, run:

```bash
cargo xtask durability-crash-matrix --case KEEP-CRASH-006 during
```

## Resume a reusable segment

The public semantic boundary admits only
`RecoverySegmentStage::Reusable`. `plan_recovery_segment_resume` binds the
exact prior stage evidence, complete-record count, append boundary, and caller
resource policy into an owned request. Complete, truncated, catalog, and
candidate-head states remain ineligible. A selected policy below the already
admitted record count is refused before storage access.

`execute_recovery_segment_resume` consumes a
`RecoverySegmentResumeStorage` capability so exclusive writer authority can
remain owned by the returned stage. The storage port returns one
protocol-bounded materialization and a writable stage positioned immediately
after those exact bytes. Before returning that stage, the executor recomputes
the saved fingerprint, repeats semantic classification under the selected
policy, and rebuilds the incremental segment digest and duplicate-identity
index from the complete prefix.

Continuation returns the ordinary `StagedSegment` state machine. Subsequent
append and seal operations therefore retain the forward protocol's checked
record count, length bounds, duplicate refusal, flushes, synchronization, and
canonical seal. The admitted prefix is never rewritten. A storage adapter that
cannot prove the writable object, canonical entry, exact materialized bytes,
and end position agree must refuse before returning the stage.

`FilesystemRecoverySegmentResumer` binds this contract to the admitted
filesystem profile. It pins the root and all protocol directories, acquires
`writer.lock`, and is consumed by execution. The reopened `current.seg` is a
regular read-write file opened without following links or truncation. Its
complete protocol-bounded bytes are fingerprinted, materialized, and
re-admitted; the handle and canonical entry retain one file identity and exact
length, the pinned namespaces are reverified, and the handle is positioned at
the admitted append boundary before handoff.

The returned `FilesystemRecoverySegmentStage` owns the pinned authority and
writer lock. Zero-record and nonempty prefixes both enter the same append and
seal state machine. Missing stages, symbolic links, changed fingerprints,
entry replacement, namespace replacement, allocation refusal, read failure,
or position disagreement return typed failures without writing stage bytes.
Dropping an unsealed resumed stage preserves its current bytes for another
explicit recovery decision.

## Complete a durable stage

The recovery plan may bind one fully verified `current.seg` or `current.cat`
to its exact observed length, checksum, digest, artifact kind, and
digest-derived pool coordinate. Under the writer lock, the executor reopens
without following links, reverifies and resynchronizes the complete staged
artifact, and refuses any drift.

The public semantic boundary admits only
`RecoverySegmentStage::Complete` and `RecoveryCatalogStage::Complete`.
`plan_recovery_stage_completion` converts either borrowed assessment into a
bounded owned request containing exact stage evidence and the validated pool
coordinate. Reusable, truncated, and `head.next` states remain ineligible.
`execute_recovery_stage_completion` requires a storage port to perform the
ordered transition and returns `RecoveryStageCompletionReceipt` only after
pool and staging durability. The receipt proves a valid orphan; it does not
prove reachability or retention.

`FilesystemRecoveryStageCompleter` binds that port to the admitted filesystem
profile. It retains the pinned root and `writer.lock` authority, pins all
protocol directories, reopens stages and pool entries without following
links, bounds every complete read by the stage grammar, rechecks stage evidence
at the link boundary, and refuses entry replacement or fingerprint drift
without removing the fixed stage. An absent fixed stage can continue only when
the canonical pool coordinate exists and verifies to the exact request
evidence.

Segment completion reuses `KEEP-CRASH-008`–`012`; catalog completion reuses
`KEEP-CRASH-016`–`020`. The executor performs the same staged-file
synchronization, no-clobber link, post-link pool verification, pool-directory
synchronization, exact stage unlink, and staging-directory synchronization as
forward publication. An existing exact pool entry is an idempotent input, not
proof by name.

After a crash, retry accepts only the exact verified stage/pool pair, the
reappeared exact stage, or the already completed pool entry with an absent
stage. It repeats any required directory synchronization and returns a
valid-orphan receipt only after the fixed staging name is durably absent.
This action never creates or finalizes a publication head.

Discard uses one fingerprint protocol and one name-selected namespace
sequence. The recovery request binds the canonical stage name, observed byte
length, and a domain-separated digest of the complete observed truncated
bytes. Under the writer lock, the executor reopens the stage without following
links and refuses replacement or fingerprint drift.

The canonical name selects one protocol maximum:

- `current.seg` uses `MAX_SEGMENT_LENGTH`;
- `current.cat` uses `MAX_CATALOG_LENGTH`; and
- `head.next` uses `PUBLICATION_HEAD_LENGTH`.

Before hashing, recovery refuses metadata length above that maximum. The
bounded reader then reads at most the selected limit plus one byte across the
complete stream. The extra byte proves oversized or concurrently grown
evidence. Either observation produces a typed oversized-evidence refusal
before any discard fingerprint is admitted; recovery preserves the evidence.

```text
stage_fingerprint =
    framed_blake3_v1(ASCII("KEEP:RECOVERY:STAGE\0"), stage_bytes)
```

`stage_bytes` is the complete exact byte sequence observed through the
bounded streaming reader after this size admission. The request admits only
algorithm value `1`.

The canonical name also selects its actual parent and crash sequence:

- `current.seg` and `current.cat` select `staging`, using
  `KEEP-CRASH-027` and `KEEP-CRASH-028`; and
- `head.next` selects the store root, using `KEEP-CRASH-029` and
  `KEEP-CRASH-030`.

For admitted evidence, the executor unlinks only the fingerprint-bound stage
and then synchronizes the selected parent. Only synchronization of the
selected parent directory returns the discard receipt. On retry, the same
request either removes the reappeared exact stage, synchronizes an already
absent name in the correct parent, or refuses different evidence as
unrecoverable ambiguity.

## Leftover next head

Recovery opens an existing `head.next` without following links and validates
its exact 128-byte grammar, checksum, generation, catalog length and digest,
and complete transitive catalog view. Recovery finalizes it only when it
exactly extends the verified current head. A lawful generation-1 candidate may
instead extend a verified uninitialized root. Finalization reuses
`KEEP-CRASH-024`–`KEEP-CRASH-026` without rewriting the candidate.

The public semantic boundary requires both a complete
`RecoveryNextHeadStage` assessment and the exact complete `CatalogSnapshot`
named by that head. `plan_recovery_next_head_finalization` refuses a mismatched
snapshot, a noninitial generation over an uninitialized root, and any
generation or predecessor other than the expected exact successor. Its owned
request retains the prior stage evidence, current-state expectation, and
candidate generation, length, and digest.

`execute_recovery_next_head_finalization` revalidates durable current state and
the complete candidate view through its storage port. A ready candidate is
synchronized and reverified before it atomically replaces `HEAD`; an
already-finalized retry requires `head.next` to be absent and skips replacement.
Both paths synchronize the root before returning
`RecoveryNextHeadFinalizationReceipt`.

`FilesystemRecoveryNextHeadFinalizer` binds this port to pinned root, writer,
staging, segment-pool, and catalog-pool capabilities. It revalidates namespace
identity around bounded no-follow loads, binds `head.next` to the request
fingerprint before and after candidate synchronization, reconstructs the
complete transitive current and candidate views, refuses a reappeared candidate
after finalization, and revalidates the complete transition at the replacement
boundary.

A truncated, corrupt, stale, or otherwise unpublishable candidate remains
invisible and blocks new publication.
Recovery never rewrites a retained `head.next`.
An explicit discard request binds its canonical name, observed length, and
`stage_fingerprint`; different evidence is unrecoverable ambiguity. The
executor:

1. unlinks the fingerprint-bound `head.next` (`KEEP-CRASH-029`); and
2. synchronizes the store root (`KEEP-CRASH-030`).

Only step 2 returns the next-head discard receipt. Retrying the same request
either safely finalizes the exact publishable candidate, removes the exact
unpublishable candidate, synchronizes its already absent name, or refuses
changed evidence.

## Deterministic refusal order

Artifact decoders validate:

1. exact minimum fixed-header availability;
2. magic, version, flags, fixed lengths, algorithms, and reserved bytes;
3. count and length bounds with checked arithmetic;
4. declared, calculated, physical-name, and actual length agreement;
5. local checksum;
6. enclosing digest;
7. kind-specific identity and payload verification;
8. ordering, duplicate, offset, and generation laws; and
9. expected head or logical identity, when supplied.

The first failed law determines the typed error. A decoder never allocates
from an untrusted count before bounds and exact total length agree.

## Platform contract

The initial production adapter is supported only on Linux when it proves:

- capability-relative no-follow access to regular files and directories;
- an ext4 store root whose inode does not enable ext4 casefolding;
- `staging`, `segments`, and `catalogs` are independently writable,
  non-casefolded ext4 directories on the root's device and mount;
- a writable mount and successful file and directory synchronization calls;
- atomic same-filesystem no-clobber hard-link creation;
- atomic same-filesystem replacement of one regular file by another;
- directory synchronization that makes create, link, unlink, rename, and
  replacement durable;
- process-scoped exclusive advisory locking on the pinned root and writer file;
  and
- post-acquisition device-and-inode verification of the retained writer-lock
  handle.

The adapter refuses every non-ext4 filesystem, read-only mount, casefolded
store or protocol directory, protocol-directory mount point, foreign
protocol-directory device, symlinked selected path, or platform other than
Linux. A single local host is an explicit deployment precondition: filesystem
metadata cannot prove that an administrator has not exposed one block device
to another host. Shared ext4 is therefore unsupported even though the adapter
cannot distinguish it from a valid local mount. Windows support is deferred
until an adapter and crash harness prove equivalent semantics.

The protocol cannot compensate for hardware or an operating system that
acknowledges synchronization without honoring it. Documentation and receipts
state this assumption; they do not upgrade it into proof.
