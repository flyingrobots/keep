# Foundation acceptance audit

<!-- markdownlint-disable MD013 -->

This page owns criterion-level verdicts for the first remaining checked
tasks in issue #131. It does not certify the repository or close that issue.

## Basis and verdict rules

The binding task text is `ROADMAP.md` at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 202–230. The inspected
main implementation is `f49cff732cf7a6e1b472decba9e4c4130990559e`.
The task text delegates detailed evidence to its named documents; these
completed entries have no separate task-specific definition-of-done block.
Their delivery condition is the roadmap's definition of Done: shipped on
main, with evidence named in a ledger, changelog, or test file.

Pass below applies to the stated task, not every future operation governed
by its contract. A contract document can be complete while implementations
that it governs remain incomplete. Originally reopened T-01.2 belongs to
issue #110 and is excluded from these remaining-task verdicts.

## T-01.1 — State the law and its limits

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| State exact named bytes or refusal | `README.md` opening law; ADR-0001 Context and Verification | Pass |
| State identity's limits | ADR-0001 Consequences: identity proves neither retention, durability, location, nor semantic type | Pass |
| State authentication and output scope | `docs/invariants/authenticated-reconstruction/README.md`: Meaning of authenticated, Complete-object reconstruction, Exact-range reconstruction, Output visibility, and Receipt posture | Pass |
| Distinguish implemented reference behavior from future durable reads | Same contract's Durable reconstruction requirement and Current public evidence | Pass |
| Deliver the named documentation on main | All three named documents exist at the inspected main commit | Pass |

This verdict does not claim durable logical reconstruction is implemented;
that explicit gap is owned by #109. The mount-comparison documentation
correction in #137 is also separate from stating the logical-byte law.

## T-01.3 — Keep application semantics out of the core

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| No Echo, Git, Graft, WARP, or CLI types in `src/` | Reviewed crate dependency declarations and source imports; no application crates or named application types occur in `src/` | Pass |
| Protocol owns physical storage, not application policy | ADR-0005; `docs/formats/segment-store-v1/requirements.md` KEEP-STORE-016; record/catalog/head coordinate declarations and codec boundaries | Pass |
| Ship the application-free protocol | Domain modules, port declarations, and adapters are present on inspected main; `xtask` is a separate workspace crate | Pass |

KEEP-STORE-016 is marked Specified in the format's design ledger. That row
alone is not implementation evidence; this verdict also inspects the
implemented types and dependencies. This is a source audit, not an
automated guarantee that every future change will preserve the boundary.

## T-02.1 — Decide the identity contract

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Accepted identity decision | ADR-0001 Status: Accepted; exact logical bytes and explicit version/algorithm coordinates | Pass |
| Canonical preimage and representations | ADR-0001 byte tables: domain-separated prefix, payload, trailing big-endian length; strict text and 59-byte binary forms | Pass |
| Limits, failures, and alternatives | ADR-0001 Verification, Allocation and performance implications, Compatibility law, Alternatives considered, and Consequences | Pass |
| Deliver decision and independent fixtures | ADR and `conformance/golden-file-worldline/v1/identities.tsv` exist on main; ADR records independent b3sum fixture generation | Pass |

An accepted contract is not proof of every planned transformation. The
compatibility law remains binding on future encryption and compaction.

## T-02.2 — Implement identity types and codecs

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| `BlobId`, `BlobHasher`, `BlobLength` | `src/blob/id.rs`, `hasher.rs`, `length.rs`: private validated identity fields, checked accumulation, typed length | Pass |
| Both strict codecs with typed failures | `src/adapters/blob_id_binary.rs`, `blob_id_text.rs`, and their error enums; length/framing/version/algorithm/canonical text admission precedes trusted construction | Pass |
| Match independent corpus, not merely round trips | `public_blob_id_matches_every_golden_vector`: computed identity, exact text and exact binary compared to frozen corpus | Pass |
| Refuse malformed encodings and preserve operational sources | Worldline laws for noncanonical text, exact maximum text bounds, binary mutation classes, and reader failures | Pass |
| Deliver implementation and evidence | Named implementation, corpus, and public integration laws are on inspected main | Pass |

Text refusals are typed variants. This task does not assert every variant
carries expected/observed fields; that stronger reopened obligation belongs
to T-01.2/#110. Parser fuzz targets exist for text, binary, and hashing;
they were inspected but not executed in this audit run.

## T-02.3 — One-pass unknown-length streaming identity

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Unknown length needs no pre-scan or seek | `BlobHasher::hash_reader` consumes `Read` until EOF; no `Seek` bound or preliminary read; finish appends checked total length | Pass |
| Constant state, no content-sized spool | `BlobHasher` owns one BLAKE3 state and `BlobLength`; reader uses a fixed 8,192-byte stack buffer | Pass |
| Partition changes do not move identity | `input_partitioning_does_not_move_blob_identity` and `generated_bytes_and_partitions_preserve_identity`, including partitioned public readers | Pass |
| Overflow refuses before mutation | `length_overflow_refuses_before_mutating_identity_state`: exact typed failure, unchanged count and hash state | Pass |
| Deliver implementation and public laws | Named code and laws are on inspected main | Pass |

Bounded state follows the inspected implementation; this run does not claim
an allocation benchmark or multi-GiB soak measurement for blob hashing.

## Executed evidence

On 2026-10-01, a Git-bundle clone of audit commit `7981988` ran in Docker
with pinned Rust 1.96.0. Its Rust source and conformance corpus are unchanged
from inspected main; the extra commit adds only the audit scope documents.

- `cargo test -p keep --test golden_file_worldline`: 14 laws passed.
- The same integration target with `--release`: 14 laws passed.
- `cargo test -p keep --lib length_overflow_refuses_before_mutating_identity_state`:
  one law passed, in debug and release.
- `cargo fmt --check` and workspace/all-target/all-feature Clippy with
  `-D warnings`: passed.
- `cargo xtask source-structure-check`: passed. An initial invocation used
  the nonexistent `source-structure` command and failed before checking;
  the corrected command succeeded.

No full-workspace, reboot, power-loss, benchmark, or fuzz-run claim follows
from these targeted results. The next five verdicts are in
[identity layers and chunking](identity-layers-and-chunking.md). The other
54 checked tasks and 19 reopened tasks still need complete accounting
before issue #131 can close.

<!-- markdownlint-enable MD013 -->
