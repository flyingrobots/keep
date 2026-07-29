# Golden File Worldline v1

The Golden File Worldline is Keep's first implementation-independent
conformance story. It proves exact logical byte identity and the behavior of a
bounded reference model. It deliberately does not claim physical storage.

The authoritative machine-readable corpus is under
[`conformance/golden-file-worldline/v1/`](../../conformance/golden-file-worldline/v1/).

## Scope

Version 1 establishes these semantic laws:

1. exact bytes produce the ADR-0001 `BlobId`;
2. canonical text and binary forms name the same validated identity;
3. changing input read partitions does not move identity;
4. an exact, immutable `BlobId -> bytes` reference mapping returns the named
   bytes or refuses;
5. admitting nearby state B does not mutate previously admitted state A;
6. claimed A identity with B bytes is a content mismatch;
7. refusing that substitution has no model side effect, and A remains exactly
   readable afterward;
8. absence, malformed identity, unsupported rules, and content mismatch remain
   distinct outcomes.

The ordered semantic worldline is:

```text
empty model
  -> identify state A
  -> admit exact state A
  -> read exact state A
  -> identify nearby state B
  -> admit exact state B
  -> read exact state A again
  -> read exact state B
  -> refuse state B bytes claimed as state A
  -> read exact state A again after refusal
  -> report a well-formed but absent identity as absent
```

“Admit” and “read” in M1 are reference-model operations. They are not claims
that Keep has shipped a production store.

## Corpus protocol

All tab-separated files are UTF-8 with LF line endings. The first line is a
schema coordinate beginning with `#`. The second line is the exact column
header. Fields may not contain tabs or line breaks. Rows are processed in file
order. Blank lines, duplicate case identifiers, unknown schema coordinates,
unknown source kinds, unknown operations, extra columns, and missing columns
are invalid.

Lengths and offsets are canonical unsigned decimal. Hex is lowercase and has
even length.

Repository source paths use the named
`keep.golden-file-worldline.path/v1` profile:

- the field is a nonempty UTF-8 relative path;
- `/` is the only separator;
- leading, trailing, or repeated `/` bytes are invalid because they create
  empty path segments;
- `.` and `..` segments are invalid;
- a backslash, colon, or NUL is invalid anywhere in the field; and
- the checker rejects rather than normalizes any noncanonical spelling.

This lexical profile is independent of filesystem admission. A canonical path
can still be refused when its target is absent, inaccessible, outside the
corpus capability, or not a regular file. The scoped
[path-profile rationale](../../conformance/golden-file-worldline/v1/rationale.md)
records why version 1 deliberately uses the stricter portable spelling.

The corpus is declarative evidence, not part of a `BlobId` preimage.

### `identities.tsv`

Each row defines exact source bytes and their canonical expected identity.

Source kinds are:

- `empty-v1`: zero bytes; parameter is `-` and repetitions is `1`;
- `file-v1`: exact bytes of the repository-relative parameter path;
- `byte-ramp-v1`: bytes `00` through `ff` in ascending order, repeated the
  declared number of times; parameter is `-`.

No text decoding or newline normalization is performed for a `file-v1` source.
The maximum v1 source is 1,048,576 bytes and the exact total materialized
corpus is 1,048,911 bytes.

### `steps.tsv`

The M1 operation vocabulary is intentionally small:

- `identify`;
- `admit-exact`;
- `read-exact`;
- `verify-claimed-content`;
- `read-absent`.

Every result is a stable machine outcome, not prose.

### `invalid-text.tsv`

Invalid text inputs are encoded as lowercase hex so whitespace and the empty
input remain unambiguous. The expected result names a language-neutral outcome
class. Rust tests map each class to an exact error variant; they never compare
display strings.

### `mutations.tsv`

Mutations are deterministic operations over a named identity or content case.
They cover content bit changes, truncation, appended data, identity-frame magic,
version, algorithm, digest, truncation, and trailing bytes.

The mutation corpus does not claim universal collision freedom. It proves that
these named mutations cannot be accepted as the original identity and relies
on ADR-0001's cryptographic contract for the broader security argument.

### `capabilities.tsv`

Capabilities record the first milestone in which an assertion may become
`required`, together with its owning GitHub issues. A `declared-future` row is
not a skipped or passing test. Promotion to `required` needs separate
executable evidence at the named milestone; it does not enlarge M1's proof
boundary retroactively.

## Partition plans

Golden vectors use exact bytes, independent of input partitioning. The M1 Rust
harness must calculate each identity using at least:

- the complete source as one update;
- one-byte updates for sources no larger than 256 bytes;
- fixed 4,096-byte updates;
- the repeating sequence `1, 7, 64, 4093, 65536` until exhaustion;
- deterministic generated partition/property cases over varied bytes.

No random seed participates in a golden value.

## Reference model

The M1 model is a deterministic, single-threaded, capacity-bounded
`BTreeMap<BlobId, Vec<u8>>`. It is an oracle for logical behavior only.

By design, its explicitly named methods materialize bounded fixtures. This
test-only oracle is not a production streaming API and provides no:

- filesystem;
- chunking or physical deduplication;
- durable publication;
- retention policy;
- range-read optimization;
- crash recovery;
- corruption repair;
- compaction;
- concurrency or performance claim.

The model computes identity through Keep's public `BlobId` API, while the
checked-in vectors were generated independently with `b3sum`. Agreement is
therefore not a self-authored golden. The repository checker streams each
canonical preimage through the same deadline-bounded external-digest adapter
used by protocol conformance. The deadline clock starts before synchronous
spawn. After spawn returns, its remaining time bounds stdin transfer and output
collection; stdout and stderr have independent byte limits; and timeout or
collection failure kills the complete child process group. Child reaping and
reader retirement use fixed per-step cleanup grace periods and remain typed if
teardown cannot be proved. Digest-specific preimage construction remains owned
by this checker rather than the process adapter.

## Exact M1 acceptance boundary

M1 is complete when:

- ADR-0001 and ADR-0002 are accepted;
- the checked-in corpus passes its independent checker;
- public `BlobId` calculation matches every vector under every required
  partition plan;
- canonical text and binary codecs refuse the original identity or decode to a
  distinct supported identity exactly as each named mutation declares;
- the bounded reference model executes `steps.tsv` exactly;
- tests prove nearby state preservation, side-effect-free substitution refusal,
  and distinct absence/mismatch outcomes;
- debug, release, doctest, lint, fuzz-target compilation, dependency, and audit
  gates are green.

M1 does not establish chunk reuse, production ingest, exact range I/O,
durability, restart recovery, physical corruption refusal, retention,
compaction, encryption, Echo agreement, or Graft integration. Those assertions
remain `declared-future` until their owning milestones provide public-path
evidence.
