# Durable authenticated reads (#109)

Change kind: new read API with a shared-core extraction. The baseline is main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. No write protocol or on-disk format is changed; #125's candidate-catalog publication gate is not added or bypassed. This is an in-progress implementation ledger, not acceptance of #109.

## Delivered candidate behavior

`DurableStore` pins a fresh `DurableSnapshot` per convenience call. Snapshot admission keeps the existing shared reader fence, catalog, retention head and manifest view and verifies every selected retained closure against the catalog. Blob lookup scans selected roots one at a time and chooses the lowest retained layout identity; exact-layout reads use the catalog directly. Reconstruction and ranges use the existing immutable reference cores and return receipts with the view coordinates only after emission succeeds.

The allocation, blocking and failure contract is in the public API documentation and [rationale](../../src/adapters/durable/rationale.md). Catalog and selected segment bytes are materialized under explicit caller policy; the API does not claim lazy segment reads or constant total memory. It adds no whole-blob output buffer or aggregate anchor index.

## Closure ledger

| Obligation | Current evidence | Required exit condition |
| --- | --- | --- |
| Golden bytes and exact read coordinates | `durable_read_law_tests` reconstructs the independent one-zero corpus and compares catalog and manifest digests against frozen bytes. Public Worldline tests reconstruct all frozen identities after reopening. | Finish the explicit reference-law parity map and remaining calibration. |
| Stable retained view across publication | `durable_view_law_tests` releases the retained root while an older snapshot remains alive; the old reader emits its original byte and head generation, and a fresh view sees release. | Add catalog-generation successor/restart evidence without introducing unsupported v2 publication. |
| Collector exclusion | The public snapshot holds the actual kernel shared fence; a deterministic exclusive try-lock refuses until drop. | Preserve this claim as fence evidence; actual GC execution remains absent on main under #21. |
| Operational failures versus evidenced absence | Removing the selected segment yields exact `OpenSegment` / `NotFound`; public whole/range reads of a catalogued layout missing its chunk yield exact logical `ChunkMissing` with untouched output. Prefix failures preserve layout, accepted bytes and `PermissionDenied`; corrupt input layouts preserve exact checksum coordinates. | Complete remaining broken-writer, absent-layout and physical-corruption parity checks. |
| Reference-core equivalence | Existing reference algorithms are generalized only over a crate-private immutable source; their single-pass authentication tests are retained. | Run generated reference properties plus complete debug/release validation, and calibrate distinct new load-bearing assertions. |
| Golden File Worldline | `tests/golden_file_worldline/durable_assertions.rs` publishes the corpus through production v1 authority, migrates it, retains it, reopens and checks frozen identities and exact bytes/ranges. `durable_range_properties.rs` adds exhaustive short intervals and multichunk boundary sweeps. | Retain precise scope: reopening after writer handles close, not a process-death test. Final-head validation remains required. |
| Documentation and final acceptance | API costs and design rationale are documented. | Update reconstruction requirements and Linux example only after complete evidence; run required final checks and reconcile exact-head review. |

## Evidence limits

The initial unit filesystem laws use owned test directories and the existing repository migration fixture with platform admission bypassed. The newer Linux public Worldline laws use production platform admission, publication, migration and retention in owned ext4 scratch. No sleep or probabilistic race is used. The collector test uses a kernel try-lock in one process, not a full GC integration or process-death claim.

New laws are medium-size because they own filesystem state. Per-test resource enforcement and suite SLOs remain gaps described in the repository [enforcement profile](../testing/enforcement.md); no compliance waiver or new resource ceiling is claimed here. Deletion criteria and oracles appear beside the laws.

Initial debug/release golden-read checks and all-feature Clippy passed; expanded view laws passed in debug. Initial authoring/compilation and Clippy failures are retained in the local evidence logs and are not product RED evidence. Final source SHAs, complete check results and actual mutation observations belong in the final PR receipt. This new API did not exist on the parent; parent compilation failure would not prove a runtime regression.

## First implementation receipt

Implementation commit `d6c38a1` passed formatting, source structure, both workspace Clippy feature profiles, complete all-feature workspace debug/release tests and doctests, and rustdoc generation on pinned Rust 1.96.0 in a copied Docker source tree. These runs include the existing generated range-to-source-slice properties and reference single-hash-pass laws. Subsequent documentation and visibility narrowing require their own final-head checks; this receipt does not transfer to future semantic changes.

Three isolated source/build mutations were observed RED on that implementation: substituting `[1]` during emission after verification made whole-read, exact-layout and nonempty-range assertions report expected `[0]` versus observed `[1]`; advancing the receipt's catalog generation made its coordinate assertion report expected 1 versus observed 2; replacing shared fence acquisition with unlock made the collector-exclusion assertion report expected `WouldBlock` versus observed successful acquisition. These are runtime calibration, not parent-bug reproductions or an assertion that every new check has already been calibrated.

At that first receipt, generated durable laws, Worldline reopen/range witnesses, missing-member/output failures and independent digest expectations remained open; the subsequent expansion below addresses those items. The issue stays open and its PR remains a draft until the full closure ledger is satisfied.

## Worldline and layout-ingress expansion

Commit `ab518b2` added production-admitted Worldline reopen/range reads, bounded range sweeps, exact logical-member absence, output-prefix failures and independent digest coordinates. It passed complete all-feature workspace debug/release tests, both Clippy profiles, formatting and source structure. This evidence does not transfer to later semantic changes.

Isolated mutations made the new whole/range prefix assertions report a false zero accepted count instead of five, and made Worldline and range source-slice assertions detect an altered emitted byte after verification. Both were observed RED at the intended runtime assertions. The mutation that changes emitted bytes correctly leaves refusal-only laws green; those protect different claims.

The next candidate adds the four caller-supplied layout entry points so reference whole-blob and catalogued-range binding laws have durable equivalents. Exact whole-blob mismatch and corrupt-layout checksum coordinates are asserted before output. These entry points and the public memory law pass focused debug/release tests and all-feature Clippy; final source-bound validation remains pending.

`durable_read_memory` measures incremental allocation during reconstruction of the frozen 1 MiB Worldline source into a nonallocating sink. It requires peak incremental allocation below one whole blob, after explicit snapshot admission; it does not claim that snapshot construction avoids materializing selected segments or prove a universal resident-memory ceiling.

The short-range sweep enumerates its finite domain in increasing length, making the first failing interval minimal within that domain. The multichunk sweep uses fixed documented boundary coordinates over the frozen source. Neither uses ambient randomness or claims exhaustive coverage of arbitrary large layouts. Replay is the named public test on the recorded source; any discovered counterexample must be retained before extending the domain.

Memory calibration inserted a deliberately unnecessary 1,048,576-byte allocation into reconstruction in an isolated source/build copy. The new public memory assertion went RED with an observed peak of 1,048,576 bytes against its strictly smaller-than-blob requirement; the unmodified candidate passed. This validates that concrete additional-buffer detector, not the entire process memory budget.

## Physical corruption and range output boundaries

Candidate `e0a540c1e0913097329b209c3ab06be40e017e56` passed the complete local validation chain and all four required hosted checks: Rust quality gates, documentation and workflow integrity, runtime fuzz smoke and dependency policy. Later test additions require their own checks; this receipt does not transfer to them.

The next test slice corrupts an actual selected segment payload in a production-admitted ext4 store. Its independent checksum preimage follows the normative v1 framing; reconstruction must preserve the typed record-index, offset and expected/observed checksum refusal before touching caller output. This is physical filesystem corruption evidence, not simulated port failure or process-death evidence.

Additional range-output laws verify exact bytes through short/interrupted writes and exact zero-progress refusal before any accepted output. The complete public Worldline integration target passes in debug and release with these additions, and all-feature Clippy passes. The local receipts are `physical-corruption.log` and `corruption-output-validation.log`; no new production behavior is introduced by this test slice.

Calibration changed the zero-progress writer's reported accepted count from zero to one in an isolated source copy. The new range assertion was observed RED on the exact `WriteZero` outcome with `ByteLength(1)` (`range-zero-mutation-red.log`), rather than a setup or compilation failure. The candidate source retains zero and is rechecked separately. The broader calibration ledger remains open; this witness is specific to accepted-byte accounting.

An initial post-mutation candidate run reused mutant build output because both copies shared a Cargo target directory. That run is invalid candidate evidence and is retained as `corruption-final-slice-corrected.log`; the earlier mistyped xtask command is retained separately. The corrected protocol gives the mutant its own target directory and invalidates the candidate's compiled source before rerunning the full public integration target in debug/release, formatting, source structure and Clippy. Only `range-zero-isolated-mutation-red.log` and `corruption-final-candidate.log` are admissible for that corrected RED/GREEN pair.

## Reference read-law reconciliation

This map covers runtime read claims, not source-string assertions in `range_read_contract.rs` or write/staging laws. Durable twins may preserve a stronger pre-output admission boundary; that difference must be named instead of describing every reference error as identical.

| Reference claim / source | Durable evidence | Remaining difference or action |
| --- | --- | --- |
| Exact reconstructed bytes, empty blobs, frozen identities (`streaming_cas/reconstruction_laws`, Worldline) | `durable_assertions`, `durable_read_law_tests` | Complete frozen corpus exercised after writer handles close and store reopen. |
| Short/interrupted writes, zero progress, accepted prefix (`reconstruction_laws`, `refusal_laws`, `range_read`, `range_read_failures`) | `durable_output_laws` | Whole and range outputs covered; calibration receipts are above. |
| Immediate output error and impossible returned write count (`refusal_laws`, `range_read_failures`) | `durable_writer_failures` | Exact layout, accepted prefix, cause or supplied/observed count checked. New validation recorded separately. |
| Whole-blob identity mismatch (`reconstruction_laws`) | `durable_layout_laws` | Caller-supplied layout reconstructs from catalogued chunks and refuses the wrong target before output. |
| False chunk-profile boundaries (`refusal_laws`) | `durable_layout_laws` | Frozen false-boundary record against production-published constituent chunks; exact expected/observed boundary and untouched output. |
| Absent blob/layout, bounds (`range_read`) | `durable_refusal_laws`, unretained-blob unit law | Exact identities and unchanged caller output; debug/release validation passes. |
| Malformed canonical layout (`refusal_laws`, `range_read`) | `durable_layout_laws` | Whole and range ingress preserve exact checksum coordinates. |
| Committed target binding / semantic and record ingress (`range_read_entrypoints`) | `durable_layout_laws` | Both durable ingress paths are exercised. |
| All short intervals / generated multichunk ranges (`range_read_properties`) | `durable_range_properties` | Short finite domain exhaustive; the durable suite also runs the reference's fixed 786,432-byte patterned source and affine coordinate domain, alongside the Worldline boundary grid. |
| Missing selected chunk (`reconstruction_laws`, reference private range laws) | `durable_refusal_laws` | Catalogued unretained layout exercises evidenced absence; retained missing closure refuses earlier at snapshot admission. |
| Corrupt selected chunk (reference private reconstruction/range laws) | `durable_corruption_laws` | Physical payload corruption refuses during segment admission before whole or range output, preserving exact record/checksum coordinates. |
| Only overlapping chunks read (reference private range law) | `a_durable_range_needs_no_nonoverlapping_chunk_records` | An unretained catalogued layout with only its interior chunk present serves an exact interior range; whole reconstruction refuses the missing first chunk. This establishes logical member independence, not minimal physical I/O during snapshot admission. |
| Exactly one chunk hash per reconstruction/selected range (reference private instrumentation) | Existing shared-core reference tests retained | Durable admission also verifies persisted evidence. No durable end-to-end single-hash claim established; instrumentation alone cannot substitute for the promised runtime contract. |

The issue itself states that collection exclusion is vacuous until #21 lands. Current evidence strengthens that floor with real shared/exclusive kernel fence exclusion and a live snapshot surviving retention publication. It does not claim an executable GC or unsupported version-two catalog publisher. The final requirement reconciliation must preserve these distinctions.

The next parity slice adds the false-profile-boundary twin using the frozen mutation record and production-published constituent chunks. It preserves the exact expected 262,143-byte first boundary versus the observed 262,144-byte boundary before output. Immediate writer errors, impossible write counts, absent range blobs and absent reconstruction layouts also have direct public durable witnesses. The complete public integration target passes debug/release, Clippy and source structure (`read-parity.log`).

Hosted validation of `cc189ff` caught a formatting-only error in the module declaration order: the earlier copy-back omitted the formatted `suite.rs`. Its Rust quality gate failed; the other required jobs passed. The author corrected the module list and records this as a validation-transfer mistake, not a runtime failure or a green final-head receipt. Subsequent validation checks the copied source against the committed files.

The writer-parity calibration uses an isolated source and target directory. It substitutes an impossible maximum of zero and wraps the immediate I/O cause as `Other`; the four whole/range assertions fail on those specific wrong public outcomes (`writer-parity-mutation-red.log`). The original source passed the same assertions in debug/release. This is assertion calibration for the new API, not a parent regression: the durable API does not exist on main.

## Range-domain and overlap closure

The range continuation adds a direct physical-corruption range witness, the reference suite's fixed patterned multichunk source/coordinate domain, and a selected-member-only catalog. The last case deliberately cannot reconstruct its whole layout: the absent first chunk produces an exact `ChunkMissing` with untouched output, while the interior range succeeds with exact source bytes and receipt coordinates. The comparison is one proof-scope behavior, not a count of harness cases. Publication and migration use production capabilities; no retained root falsely claims closure over the intentionally incomplete layout.

The complete Worldline target passes debug/release, all-feature Clippy and source structure (`range-overlap-parity.log`). Earlier generator-only and physical-corruption receipts are retained separately. The finite affine coordinate domain is replayable from source but does not claim a general shrinking framework; the exhaustive short-domain run remains ordered by increasing length. This narrows the remaining acceptance work without claiming arbitrary-input exhaustiveness.

An isolated source/build mutation made range verification start with all layout entries instead of the selected entries. The overlap law went RED at the read boundary with an exact missing first-chunk refusal (`range-overlap-mutation-red.log`); the original candidate succeeds on the interior range. This calibrates logical overlap independence without using internal lookup counters as its oracle.
