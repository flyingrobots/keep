# Conformance-oracle acceptance audit

This page owns the four originally completed F-08 task verdicts for
issue #131. Binding task text is the original roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 423–430.
Inspected main is `f49cff732cf7a6e1b472decba9e4c4130990559e`.
These completed entries have no separate task-specific definition-of-done
block: the named artifact, implementation and evidence must be on main.

## T-08.1 — Worldline scenario and reference model

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected and executed evidence | Verdict |
| --- | --- | --- |
| State eight semantic laws and ordered A/B scenario | `docs/conformance/golden-file-worldline.md`: Scope, ordered worldline, canonical corpus grammar and Reference model | Pass |
| Independent identity and scenario oracle | `xtask/src/golden_file_worldline/`: typed preimage construction, external b3sum identity witness, canonical coordinate/mutation checks and ordered scenario model | Pass |
| Admit B without changing A; reject B claimed as A with no side effect | Scenario oracle checks exact identity/input fixture relations and ordered admitted-set membership through the required step sequence; public Worldline laws separately execute the production reference store | Pass |
| Malformed, unsupported, absent and mismatched outcomes stay distinct | Invalid-text and mutation oracles; corpus steps; typed checker errors and canonical table admission | Pass |
| Executable checker and delivery | `cargo xtask golden-file-worldline-check` passed; documentation, corpus and checker exist on main | Pass |

The independent model is corpus-bounded logical evidence, not physical
storage or crash evidence. Required capability rows and declared-future rows
remain distinct; a future row is not an executable witness. Oracle modules
do not import Keep production identity/store types. The separate xtask
workspace crate does depend on Keep for other repository tasks.

## T-08.2 — Rust CDC and ChunkId oracles

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected and executed evidence | Verdict |
| --- | --- | --- |
| Rust independent CDC recipe and scalar boundary oracle | `xtask/src/protocol_conformance/cdc_profile/`: regenerates Gear table and 96-byte profile record; reconstructs sources/mutations and scalar boundary expectations | Pass |
| Independent ChunkId preimages | `xtask/src/protocol_conformance/chunk_identity.rs` builds typed domain/version/algorithm/content/length recipes and checks external b3sum results | Pass |
| Canonical bounded fixture admission | Corpus capability-relative no-follow reads; table byte/row limits; canonical decimal/hex/path profiles; final-LF/CR/blank-line refusal | Pass |
| Executable command and delivery | Isolated `cargo xtask conformance-check` passed; both corpora and checker modules exist on main | Pass |

Independent means independent of production Keep codecs/detectors. Hash
witnesses still rely on BLAKE3 implementations; no collision-freedom or
second-language implementation claim follows.

## T-08.3 — Segment-store v1 and v2 fixture oracles

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected and executed evidence | Verdict |
| --- | --- | --- |
| Construct frozen v1 bytes independently | `xtask/tests/segment_store_protocol_contract/fixture_oracle/`: segment, bundle, catalog and head field encodings, checksums, identity and artifact comparisons | Pass |
| Construct frozen v2 bytes independently | `xtask/tests/retention_store_v2_format_oracle/`: registered definitions, inventory, intent, receipt, retention roots/manifests/head and exact v1 predecessor artifacts | Pass |
| Oracles avoid production codec imports | Both oracle directories use their own field/encoding/preimage construction; no Keep production imports | Pass |
| Executable exact fixture evidence | Both integration targets passed in debug/release: 25 v1 protocol laws and four v2 format laws | Pass |
| Delivery | Named directories, integration roots and v1/v2 frozen corpora exist on main | Pass |

Matching fixtures does not prove every durable write/recovery crash state.
Those obligations remain with their owning recovery and migration tasks.

## T-08.4 — Bounded external digest execution

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected and executed evidence | Verdict |
| --- | --- | --- |
| Written failure/retirement decision | Accepted ADR-0008 specifies process-group termination, direct-child reaping, bounded reader retirement and typed cleanup evidence | Pass |
| Bound input/execution/output work | `xtask/src/external_digest.rs`: streamed input parts, admitted PATH/C locale, raw single-thread b3sum, ten-second deadline and separate stdout/stderr caps | Pass |
| Preserve primary and cleanup failures | Bounded-process capture/reader modules retain original failure and additional retirement failure; detaching an already-failed reader never yields success | Pass |
| Executable failure and resource laws | Bounded-process tests cover output limits, process deadlines, input/reader failures, signaling and retirement; passed in debug/release | Pass |
| Delivery | ADR, process boundary, external witness and executable laws exist on main | Pass |

The deadline starts before synchronous spawn and governs remaining work
after spawn returns; it is not an OS guarantee that spawn itself cannot
block. Failed-reader retirement bounds caller waiting, not proof that an
arbitrarily blocked injected worker has stopped. These documented limits
are preserved, and uncertain cleanup remains failure.

## Executed evidence

On 2026-10-01, pinned Rust 1.96.0 in copy-isolated Docker used the
main-equivalent clone `7981988` with its dedicated target directory.

- Worldline command passed; 31 Worldline unit laws passed in debug/release.
- CDC/ChunkId conformance command passed with b3sum 1.8.5.
- v1 protocol and v2 format targets: 25 and four laws passed per profile.
- 22 bounded-process unit laws passed in debug/release; the filtered
  integration laws also passed. Targets selecting zero tests are not counted.

Twenty-six of 64 remaining checked tasks have verdicts. The two reference
corrections retain their acceptance/delivery limitations; 37 other checked
tasks and full accounting of 19 reopened entries remain. No checkbox changes.
