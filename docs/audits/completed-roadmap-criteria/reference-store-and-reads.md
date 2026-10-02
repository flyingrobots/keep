# Reference-store and authenticated-read acceptance audit

This page owns eight originally completed task verdicts for issue #131.
Binding text is the roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 317–412.
Inspected main is `f49cff732cf7a6e1b472decba9e4c4130990559e`.
PR implementations and mainline delivery are separate observations.

## T-06.1 — Bounded ingestion and reconstruction

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Evidence on inspected main | Verdict |
| --- | --- | --- |
| Bounded streaming input | `src/reference/ingestion.rs`: fixed read/chunk buffers, checked input accounting, entry limit; `chunk_staging.rs`: committed plus pending plus incoming payload capacity checked before copy | Pass |
| Invisible staging and explicit commit | `tests/streaming_cas/ingestion_laws.rs`: staged absence, dropped stage, capacity refusal, cross-store deduplication and commit laws | Pass |
| Exact bounded reconstruction | Committed reconstruction/range allocation laws; complete identity and profile verification before output; writer belongs to caller | Pass |
| 216 exhaustive three-step model sequences | `tests/streaming_cas/model_laws.rs`: six operations in three nested loops; after every step each of three blob cases is compared with a BTreeMap model | Pass |
| Delivery | Implementation and named public/model/allocation laws are on main | Pass |

Bounded does not mean constant total staging memory: unique pending payload
may grow to capacity and metadata to the configured entry limit. T-06.4
has its own stronger measurement/documentation obligation below.

## T-06.2 — Chunk deduplication is a storage fact

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Evidence on inspected main | Verdict |
| --- | --- | --- |
| Deduplicate by exact ChunkId | `ReferenceChunkStaging::stage_chunk` checks committed and pending ordered maps by ChunkId and compares actual bytes before reuse | Pass |
| Refuse conflicting bytes | `conflicting_existing_chunk_bytes_are_never_repaired` and ingestion conflict mapping preserve refusal rather than replacing content | Pass |
| Do not confer retention | Reference-store README Contract and Ingestion explicitly distinguish process-memory presence/deduplication from retention and durability | Pass |
| Observable reuse | `identical_chunks_are_deduplicated_without_a_retention_claim`: first stage owns payload, committed repeat has zero pending payload; cross-store commit checks missing deduplicated content | Pass |
| Delivery | Named architecture documentation, implementation and tests are on main | Pass |

## T-06.3 — Single-pass authenticated emit

**Acceptance: fail/pending reconciliation. Delivery: blocked.**

Current correction owner is #71 / PR #134 at `fbb3813`. The checked roadmap
entry is historical scope, not proof that this correction has landed.

| Original field / obligation | Evidence | Verdict |
| --- | --- | --- |
| One chunk authentication per selected entry | Main calls verified_chunk during verification and emission; PR #134 counts one ChunkId hash per selected entry | Fail on main; pass in PR |
| Complete BlobId/profile verification and range accounting | Existing refusal/receipt/property laws retained; PR's exact range counter regression covers one-byte, whole and empty requests | Pass in PR |
| Preserve failure surfaces and untrusted prefix posture | Typed failures and writer-prefix laws retained; no successful receipt after sink failure | Pass in PR |
| Counting adapter and no new public type | Private test-only observed hashes; exact whole/range counting laws; only private emission helper added | Pass in PR |
| Immediate output by default; strict proof only through an explicit new API | Original scope and issue expected behavior request immediate output. PR deliberately retains normative complete-proof-before-output behavior with no new mode | Unsatisfied; contract-owner clarification pending |
| Existing golden/refusal laws and boundary range | Full keep suite and exact selected-chunk counting laws in PR; main's selected-range law still observes two reads | Pass for preserved laws; optimization not on main |
| Stress / read amplification becomes 1 | Source-bound optimized TSV at 30ffe90 records whole-blob authenticated-byte accounting 1/1; range accounting counts complete selected chunks once | Pass in PR; no cross-host CPU speedup claim |
| Benchmark baseline regenerated | `benchmark/baselines/30ffe90-aarch64-apple-darwin.tsv`: clean source, compiler/host, 100 samples, five warmups; historical baseline retained | Pass in PR |
| CHANGELOG and rewritten two-pass rationale | PR updates both; fbb3813 also corrects the stale current metric definition | Pass in PR |
| Regression merged; unchanged interfaces; no prerequisites | Interfaces unchanged and branch based on main; regression remains unmerged | Delivery blocked |

Literal output-order scope conflicts with the current reconstruction
contract. One-hash evidence cannot resolve that conflict. PR #134 now refs
instead of closes #71, and its Code Lawyer report records the unresolved
requirement. Do not declare this task complete or silently alter its scope.

## T-06.4 — Bounded-memory staging

**Acceptance: pass in PR implementation. Delivery: blocked.**

Correction owner is #74 / PR #135 at `b5def4a`.

| Original field / obligation | Evidence in PR #135 | Verdict |
| --- | --- | --- |
| Bounded missing-chunk window, or explicit unavoidable-materialization rationale with enforced capacity refusal | Colocated rationale explains invisible unique chunks must remain owned by an in-memory stage; capacity is checked before copies | Permitted rationale alternative met |
| Explicit bound, checked accounting and invisible staged value | Named fixed scratch constant; payload capacity and independent entry/metadata cap documented; repeated-content stage remains absent until commit | Pass |
| Source larger than ceiling | Deterministic one-million-byte source refuses at 200,000-byte capacity within measured scratch/payload/empirical metadata allowance | Pass |
| Exactly at capacity and one byte over | Public ingestion laws assert admission at the configured payload capacity and exact attempted total on overflow | Pass |
| All chunks present and none present | Deduplicated stage owns no payload; over-capacity and repeated-content fixtures exercise new unique payload | Pass |
| Terminal source failure after partial staging | b5def4a consumes 300,000 bytes, including a staged full chunk and partial successor; exact source error, zero retained staging heap and original committed content preserved | Pass |
| Memory ceiling law and large-source behavior | Seven allocation laws; synthetic 4 GiB input refuses after first 256 KiB chunk; successful synthetic 4 MiB repeat stores one unique chunk | Pass; not successful 4 GiB ingestion |
| API scope and documentation | Stage parameters unchanged; additive scratch constant, README memory section, rationale and CHANGELOG; no durable spill or async fallback | Pass |
| Memory law merged; README example unchanged | Root README and existing examples unchanged by PR; law remains unmerged | Delivery blocked |

The metadata allowance is empirical fixture slack, not a universal allocator
bound. The source error is preconstructed outside measurement to preserve
its allocation without attributing it to staging. A cleanup-leak mutant
fails with 262,368 retained bytes; real code passes. No production leak is
claimed. Review gates remain; source/fixture evidence is not integration.

## T-07.1 — State the authenticated reconstruction contract

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Evidence on inspected main | Verdict |
| --- | --- | --- |
| Define complete and range proof scopes | Authenticated-reconstruction README: Complete-object reconstruction, Exact-range reconstruction, Receipt posture and Output visibility | Pass |
| State success/refusal/operational failure separately | Contract and Decisions, refusals, and operation failures; requirement ledger -005 and -006 | Pass |
| Bind coordinates, limits and future durable obligations | Coordinates and evidence; receipt types; durable requirement ledger -009/-010 remains Planned | Pass |
| Delivery | Contract, rationale and requirement ledger exist on main | Pass |

## T-07.2 — Complete-object and exact-layout reads

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Evidence on inspected main | Verdict |
| --- | --- | --- |
| Complete-object exact bytes and receipt | Committed reconstruction law verifies bytes, target, exact layout and byte count; mismatch/missing/profile refusal laws emit nothing | Pass |
| Exact requested committed layout, no fallback | `reconstruct_layout` looks up only its requested LayoutId; absent layout returns LayoutMissing; ordered automatic selection is a separate path | Pass |
| Receipt is complete-object scope | Private construction after complete verification and exact written length; ReconstructionReceipt differs from RangeReadReceipt | Pass |
| Delivery | Public API, implementation, receipt and named laws are on main | Pass |

## T-07.3 — Exact range reads select overlapping chunks only

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Evidence on inspected main | Verdict |
| --- | --- | --- |
| Load only minimal overlap | Flat-layout planner and instrumented private selected-chunk law; unselected chunks removed without failing the read | Pass |
| Exact range and admitted target binding | Boundary/property laws, admitted/canonical entrypoint equivalence, forged same-length target-layout refusal | Pass |
| Keep receipt scope narrow | RangeReadReceipt explicitly excludes whole blob, unselected chunks and profile-boundary verification | Pass |
| Delivery | Named public tests and private instrumented law are on main | Pass |

## T-07.4 — Distinct success, evidenced refusal and operational failure

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Evidence on inspected main | Verdict |
| --- | --- | --- |
| Content failures and operational failures differ | Boundary error variants distinguish missing/identity/profile evidence from source/sink I/O errors with preserved sources | Pass |
| Exact writer diagnostics and accepted prefix | `broken_range_writers_preserve_exact_failure_boundaries` and `range_failure_reports_the_prefix_already_accepted`: exact counts, layout, source kind and source chain | Pass |
| No success receipt on failure | Result Err paths cannot construct a success receipt; prefix failure remains explicitly untrusted | Pass |
| Delivery | Failure laws, receipt construction and contract evidence are on main | Pass |

## Validation and remaining accounting

Docker uses pinned Rust 1.96.0 and separate target directories per source
clone. Main, PR #134 and PR #135 source identities are distinct. Previous
shared-target executions could reuse stale binaries across clones and are
not used as evidence for these verdicts; isolated reruns replace them.

Full `cargo test -p keep` passed in debug/release for all three clones,
including public, unit, property, corruption, memory and doctest targets.
The isolated main memory target has two laws; PR #135 has seven. Each clone
also passed fmt, workspace/all-target/all-feature Clippy with warnings
denied, and source-structure policy. Main uses the unchanged Rust/corpus
source at audit clone `7981988`. The PR #134 clone uses `6a95b5f`; its later
fbb3813 change is documentation-only and was linted separately. PR #135
uses `e43ad0e` plus the final b5def4a test/documentation patches. No
full-workspace or dependency-policy rerun is claimed here.

Twenty-two of 64 remaining checked tasks have verdicts. Two reference-store
corrections have unmet mainline delivery, and #71 additionally has unresolved
acceptance scope. Forty-two tasks and the 19 reopened entries still need
complete accounting. No checkbox is changed by this audit document.
