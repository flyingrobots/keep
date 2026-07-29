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
that synchronization. The pinned-filesystem implementation remains
unimplemented.

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

## Complete a durable stage

The recovery plan may bind one fully verified `current.seg` or `current.cat`
to its exact observed length, checksum, digest, artifact kind, and
digest-derived pool coordinate. Under the writer lock, the executor reopens
without following links, reverifies and resynchronizes the complete staged
artifact, and refuses any drift.

Segment completion reuses `KEEP-CRASH-009`–`012`; catalog completion reuses
`KEEP-CRASH-017`–`020`. The executor performs the same no-clobber link,
post-link pool verification, pool-directory synchronization, exact stage
unlink, and staging-directory synchronization as forward publication. An
existing exact pool entry is an idempotent input, not proof by name.

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
`KEEP-CRASH-025` and `KEEP-CRASH-026` without rewriting the candidate.

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
- a writable mount and successful file and directory synchronization calls;
- atomic same-filesystem no-clobber hard-link creation;
- atomic same-filesystem replacement of one regular file by another;
- directory synchronization that makes create, link, unlink, rename, and
  replacement durable;
- process-scoped exclusive advisory locking on the pinned root and writer file;
  and
- post-acquisition device-and-inode verification of the retained writer-lock
  handle.

The adapter refuses every non-ext4 filesystem, read-only mount, casefolded store
root, symlinked selected path, or platform other than Linux. A single local
host is an explicit deployment precondition: filesystem metadata cannot prove
that an administrator has not exposed one block device to another host. Shared
or multiply mounted ext4 is therefore unsupported even though the adapter
cannot distinguish it from a valid local mount. Windows support is deferred
until an adapter and crash harness prove equivalent semantics.

The protocol cannot compensate for hardware or an operating system that
acknowledges synchronization without honoring it. Documentation and receipts
state this assumption; they do not upgrade it into proof.
