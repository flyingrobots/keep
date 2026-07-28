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
verifies complete content before classification. Unknown names, symlinks,
conflicting canonical coordinates, duplicate digests, multiple fixed-name
stages, or a head that cannot be proven atomic are unrecoverable ambiguity.

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

An explicit recovery executor may resume a reusable stage, preserve an
orphan, explicitly discard a named truncated stage, or finalize one fully
verified next-generation head using the same generation comparison and
publication steps. Recovery itself uses the same crash points and must be
idempotent. It may not silently promote the newest artifact, truncate to the
last plausible boundary, rewrite a checksum, delete a valid orphan, or select
by timestamp.

Discard is one explicit two-step protocol. The recovery request binds the
canonical stage name, observed byte length, and a domain-separated digest of
the complete observed truncated bytes. Under the writer lock, the executor
reopens the stage without following links and refuses replacement or
fingerprint drift.

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

For admitted evidence, the executor:

1. unlinks only that verified stage (`KEEP-CRASH-027`); and
2. synchronizes `staging` (`KEEP-CRASH-028`).

Only step 2 returns the discard receipt. On retry, the same request either
removes the reappeared exact stage, synchronizes an already absent name, or
refuses different evidence as unrecoverable ambiguity.

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

The initial adapter is supported only when it can prove:

- capability-relative no-follow access to regular files and directories;
- case-sensitive, byte-preserving directory names without path aliases;
- atomic same-filesystem no-clobber hard-link creation;
- atomic same-filesystem replacement of one regular file by another;
- file synchronization that covers required data and metadata;
- directory synchronization that makes create, link, unlink, rename, and
  replacement durable;
- process-scoped exclusive advisory locking; and
- one host with one writer.

The adapter refuses network filesystems, shared multi-host mounts, filesystem
types with unknown rename or synchronization semantics, symlinked protocol
paths, and platforms whose directory durability cannot be established.
Windows support is deferred until an adapter and crash harness prove
equivalent semantics.

The protocol cannot compensate for hardware or an operating system that
acknowledges synchronization without honoring it. Documentation and receipts
state this assumption; they do not upgrade it into proof.
