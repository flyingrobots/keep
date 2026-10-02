# Flat-layout acceptance audit

This page owns the four originally completed F-05 task verdicts for
issue #131. It uses the source and delivery rules in [foundations.md](foundations.md).
Binding task text is the roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 287–294.
Inspected main is `f49cff732cf7a6e1b472decba9e4c4130990559e`.
These completed entries have no separate task-specific definition-of-done
block. Delivery requires the named specification, implementation and evidence
on main. Originally unchecked T-05.5 is excluded.

## T-05.1 — Specify the format

**Acceptance: pass. Delivery: pass. Leave checked.**

The binding scope is KEEP-LAYOUT-001 through -012. Every row below is an
inspection of the frozen specification in
`docs/formats/flat-chunk-layout-v1/README.md`, not an implementation verdict.

| Requirement | Exact specification evidence | Verdict |
| --- | --- | --- |
| 001: versioned fixed-width grammar | Canonical plan record: magic, version, codec, big-endian header and 44-byte entry tables | Pass |
| 002: domain and exact plan length | Canonical LayoutId: ADR-0002 envelope, domain prefix and trailing checked length | Pass |
| 003: target identity and length | Fixed header: embedded canonical 59-byte BlobId; Structural laws: final aggregate equals embedded logical length | Pass |
| 004: one registered profile | Fixed header binds a typed profile coordinate; admission recognizes registered identities only | Pass |
| 005: typed chunk coordinates | Header fixes chunk version and algorithm; Entry binds positive length and exact digest | Pass |
| 006: ordered contiguous spans | Structural laws: offset zero, exact predecessor end, no gaps or overlaps | Pass |
| 007: checked arithmetic | Record/entry formulas, Bounds, and Structural laws require checked arithmetic before allocation or cursor movement | Pass |
| 008: exact empty/nonempty cardinality | Structural laws distinguish zero entries from required positive count | Pass |
| 009: depth and allocation bounds | Bounds: depth one, 1,048,576 entries, 46,137,520-byte record maximum, configured admission cap before materialization | Pass |
| 010: unsupported mandatory fields | Header zero flags/reserved bytes; deterministic refusal order rejects unsupported coordinates | Pass |
| 011: canonical framing | Positional fields exclude duplicate fields; declared/calculated/actual length equality rejects extra or missing bytes | Pass |
| 012: typed domain-separated checksum | Record checksum defines exact BLAKE3 preimage and separates checksum from identity, authority and retention | Pass |
| Delivery | Frozen specification, rationale, requirement ledger and golden/mutation corpus are on inspected main | Pass |

The specification also distinguishes parsing, admission and verified
reconstruction, and documents compatibility/security limits. Its future
hierarchical codec is not permission to reinterpret flat codec 1.

## T-05.2 — Implement codec, admission and fuzz target

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected implementation and executed evidence | Verdict |
| --- | --- | --- |
| Admitted semantic state | `src/layout/admitted.rs` and `validation.rs`: private fields, registered profile, contiguous checked spans, cardinality and configured cap | Pass |
| Canonical record and explicit policy | `src/adapters/layout_record.rs`, `layout_decode_policy.rs`, encoder, decoder and framing modules; trusted types follow bounded raw-header parsing | Pass |
| Bounds before allocation | Decoder framing checks protocol/configured counts, checked lengths and exact input length before entry materialization; configured-cap and mutation laws exercise refusals | Pass |
| Checksum and expected identity | Decoder verifies checksum before nested semantic admission, then expected LayoutId; mutation laws pin first-failure precedence | Pass |
| Exact independent golden bytes | `tests/layout_record.rs`, `layout_decode.rs`, and `layout_oracle.rs`: compare semantic encoding, admitted decoding and independent field/checksum/identity oracle against frozen records | Pass |
| Generated canonicality and corruption | `tests/layout_properties.rs` and `layout_mutations.rs`: generated canonical records and every frozen mutation's typed first-failure class | Pass |
| Supply bounded fuzz target | `fuzz/fuzz_targets/layout_record.rs`: protocol-capped decoding, exact canonical reencoding and expected-identity readmission for accepted input | Pass |
| Delivery | Implementation, public tests, fuzz target and `conformance/layout/v1/` are on inspected main | Pass |

The record oracle uses independently assembled field/preimage calculations
and the BLAKE3 library; it is not an independent hash implementation.
Decoding materializes bounded entry metadata, as documented, and does not
claim constant memory. This audit inspects the fuzz target but does not
claim a locally executed fuzz campaign or a maximum-entry allocation soak.

## T-05.3 — Verified reconstruction replays the profile

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected implementation and executed evidence | Verdict |
| --- | --- | --- |
| KEEP-LAYOUT-016: reproduce declared spans | `src/profile/verification.rs` replays the registered detector and compares each emitted span with layout entries, including EOF completion | Pass |
| Apply replay to the exact reconstructed bytes | `src/reference/reconstruction.rs::verify_complete_blob` authenticates each chunk, feeds profile verifier and blob hasher, finishes both before output | Pass |
| Refuse false boundaries despite correct chunk bytes | `content_correct_false_profile_boundaries_are_refused_before_output`: exact first index and expected/observed span lengths, with no emitted bytes | Pass |
| Preserve complete-object identity scope | Whole-blob mismatch and committed reconstruction laws in `tests/streaming_cas/` verify complete identity before output | Pass |
| Delivery | Domain verifier, reference mapping and named public laws are on inspected main; requirement ledger marks -016 implemented | Pass |

This does not certify a durable reconstruction API. The extra verification
pass on main is the separate performance defect owned by #71/PR #134;
it does not remove profile replay or weaken this task's acceptance law.

## T-05.4 — Exact range planning

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected implementation and executed evidence | Verdict |
| --- | --- | --- |
| KEEP-LAYOUT-017: minimal ordered overlap | `src/layout/range_plan.rs`: checks bounds; skips entries ending at/before start; stops at entries starting at/after end; returns the first/end interval with checked arithmetic | Pass |
| Empty ranges select no entries | Planner returns no first entry and zero count; public zero-length laws cover coordinates through EOF with no output | Pass |
| Exact boundary and generated range bytes | `tests/range_read.rs` and `range_read_properties.rs`: boundary slices, every short valid range and generated multichunk ranges match reference slices | Pass |
| Instrumented lookup scope | `an_interior_range_loads_only_its_single_overlapping_chunk`: removes every unselected chunk, succeeds on an interior range, and observes only the selected identity | Pass |
| Precise absent/out-of-bounds/malformed failures | Public range entrypoint/failure laws refuse before output; admitted and canonical entrypoints agree | Pass |
| Delivery | Public planner, range APIs and instrumented laws are on inspected main; ledger marks -017 implemented | Pass |

The instrumented law observes two reads of the same selected chunk on main;
it proves minimal selected identities, not a single authentication pass.
An exact range does not authenticate the whole BlobId or replay all profile
boundaries; the public range receipt documents that narrower scope.

## Executed evidence and limitations

The earlier targeted runs below used a shared target directory. Fresh
source-isolated full keep debug/release suites now replace them as
source-specific evidence; see the [validation correction](README.md).

On 2026-10-01, copy-isolated Docker and pinned Rust 1.96.0 used the
main-equivalent source clone identified in [foundations.md](foundations.md).
The following twelve public targets passed in debug and release, totaling
52 laws in each profile:

- `adapters_layout_contract`, `layout_decode`, `layout_mutations`;
- `layout_oracle`, `layout_properties`, `layout_record`;
- `range_read`, `range_read_contract`, `range_read_entrypoints`;
- `range_read_failures`, `range_read_properties`, `streaming_cas`.

The exact private interior-range law passed once in each profile. An initial
module filter selected zero tests and supplies no evidence. A subsequent
test-list command failed because ripgrep was absent inside the container;
the corrected exact-name execution ran and passed the intended law.

No durable crash, benchmark execution, full-workspace or fuzz-campaign
claim follows from these checks. The subsequent
[reference-store audit](reference-store-and-reads.md) brings accounting to
22 verdicts, followed by four [conformance verdicts](conformance-oracles.md).
The other 38 and full accounting of 19 reopened tasks remain.
