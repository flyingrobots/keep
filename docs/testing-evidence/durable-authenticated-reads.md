# Durable authenticated reads (#109)

Change kind: new read API with a shared-core extraction. The baseline is main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. No write protocol or on-disk format is changed; #125's candidate-catalog publication gate is not added or bypassed. This is an in-progress implementation ledger, not acceptance of #109.

## Review after draft readiness

The independent approval and successful checks on `186ab8a00796101d26640084f05960624d76f77d` predate the subsequent hosted Codex and CodeRabbit findings; they do not establish readiness of a revised candidate.

| Review obligation | Disposition and evidence | Exit condition |
| --- | --- | --- |
| CodeRabbit: selected root namespace binding (`discussion_r4170510757`). | Confirmed runtime defect on `186ab8a`: both direct selected-root read and durable snapshot admission accepted a canonical foreign-namespace root. The fix at `FilesystemRetentionSnapshot::retained_root` retains existing digest/generation checks and reports typed expected/observed namespace coordinates. | Focused debug/release laws, source-preservation calibration, final-head full validation and independent delta review. |
| Codex: inward ownership of shared authentication (`discussion_r4170495607`). | Open; durable code currently imports the shared policy and port through `reference`. | Move the shared behavior to an inward semantic boundary without changing authenticated output/refusal behavior; preserve public API compatibility and generated parity evidence. |
| Codex: relative store locator stability (`discussion_r4170495611`). | Open; `DurableStore::open` retains relative paths for later resolution. | A controlled working-directory change cannot silently retarget the same handle; locator/admission errors remain typed and documented. |
| Codex: reader production-platform admission (`discussion_r4170495617`). | Open; the current reader checks namespace and migration identity without the production filesystem-profile admission. | Reject unsupported filesystem semantics before exposing a durable snapshot, without taking writer authority; preserve positive production-profile reader evidence. |

### Namespace regression

Change kind: bug fix; the public fixture publishes real content, migrates, and retains it on the admitted ext4 profile, then deliberately installs a checksummed manifest selecting the original root under another namespace outside publication.

No raw mutation occurs concurrently with the read under test.

`a_selected_root_from_another_namespace_refuses_direct_read` and `a_foreign_retained_namespace_refuses_durable_output` were observed RED against production head `186ab8a` after successful compilation, reporting respectively `foreign namespace root was returned by the public reader` and `foreign namespace root admitted a durable snapshot`.

The final laws assert `FilesystemRetentionSnapshotError::Root`, `InvalidData`, and the exact `RetentionSelectedRootRefusal::Namespace` expected/observed digests; the convenience-read law also checks the caller's output sentinel remains unchanged.

The tests are medium, real-filesystem public-path evidence; no process-death, power-loss, or arbitrary concurrent namespace-isolation guarantee is inferred.

The new diagnostic is preserved inside the existing root error boundary, so unrelated checksum, generation, digest, and operational causes retain their existing paths.

Raw artifacts retain `namespace-red-on-186ab8a.log`, `namespace-green.log`, and `namespace-structure.log`; a replay initially included the new diagnostic type against the old library and failed compilation (`namespace-red-replay.log`), which is excluded from behavioral RED evidence.

The corrected parent-compatible regression was replayed RED in `namespace-red-replay-corrected.log` and committed separately as `0a43198` before the fix; that commit can reproduce the original acceptance defect without the new diagnostic type.

The fixed public laws pass in debug and release; the full Golden File Worldline integration binary, existing retention snapshot laws, doctests, all-target/all-feature Clippy, formatting, and structure checks also pass in the copied Docker candidate.

Swapping only the new diagnostic's expected and observed namespace digests in a separate source/target directory made both final public laws fail their exact-coordinate assertion (`namespace-coordinate-red.log`); the unchanged candidate remained green (`namespace-relevant-validation.log`).

Final full-chain checks and independent review remain pending while the other three review obligations are resolved.

## Delivered candidate behavior

`DurableStore` pins a fresh `DurableSnapshot` per convenience call. Snapshot admission keeps the existing shared reader fence, catalog, retention head and manifest view and verifies every selected retained closure against the catalog. Blob lookup scans selected roots one at a time and chooses the lowest retained layout identity; exact-layout reads use the catalog directly. Reconstruction and ranges use the existing immutable reference cores and return receipts with the view coordinates only after emission succeeds.

The allocation, blocking and failure contract is in the public API documentation and [rationale](../../src/adapters/durable/rationale.md). Catalog and selected segment bytes are materialized under explicit caller policy; the API does not claim lazy segment reads or constant total memory. It adds no whole-blob output buffer or aggregate anchor index.

## Closure ledger

| Obligation | Current evidence | Required exit condition |
| --- | --- | --- |
| Golden bytes and exact read coordinates | `durable_read_law_tests` reconstructs the independent one-zero corpus and compares catalog and manifest digests against frozen bytes. Public Worldline tests reconstruct all frozen identities after reopening. | Parity map below is populated; independent review must assess the stated scope and calibration gaps. |
| Stable retained view across publication | `durable_view_law_tests` releases the retained root while an older snapshot remains alive; the old reader emits its original byte and head generation, and a fresh view sees release. | Current supported retention-publication path is exercised. A production v2 catalog writer is absent on the baseline; review must not confuse that absent path with demonstrated catalog-successor execution. |
| Collector exclusion | The public snapshot holds the actual kernel shared fence; a deterministic exclusive try-lock refuses until drop. | Preserve this claim as fence evidence; actual GC execution remains absent on main under #21. |
| Operational failures versus evidenced absence | Removing the selected segment yields exact `OpenSegment` / `NotFound`; public whole/range reads of a catalogued layout missing its chunk yield exact logical `ChunkMissing` with untouched output. Prefix failures preserve layout, accepted bytes and `PermissionDenied`; corrupt input layouts preserve exact checksum coordinates. | Public writer, absence, physical whole/range corruption and false-profile witnesses are present; require final-head validation and review. |
| Reference-core equivalence | Existing reference algorithms are generalized only over a crate-private immutable source; their single-pass authentication tests are retained. | Reference and durable generated domains are exercised. Core authentication-pass evidence remains distinct from repeated durable admission; final validation and independent review remain. |
| Golden File Worldline | `tests/golden_file_worldline/durable_assertions.rs` publishes the corpus through production v1 authority, migrates it, retains it, reopens and checks frozen identities and exact bytes/ranges. `durable_range_properties.rs` adds exhaustive short intervals and multichunk boundary sweeps. | Retain precise scope: reopening after writer handles close, not a process-death test. Final-head validation remains required. |
| Documentation and final acceptance | API costs and design rationale are documented. | Requirements and compiled Linux example are updated for the candidate; final checks and exact-head independent review remain open. |

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
| Exactly one chunk hash per reconstruction/selected range (reference private instrumentation) | Existing shared-core reference tests retained | The normative single-hash paragraph explicitly describes ReferenceStore. Durable reads reuse that immutable core pass, while snapshot/catalog admission independently verifies persisted evidence. No durable end-to-end single-hash promise is made; independent review must assess this scope reconciliation. |

The issue itself states that collection exclusion is vacuous until #21 lands. Current evidence strengthens that floor with real shared/exclusive kernel fence exclusion and a live snapshot surviving retention publication. It does not claim an executable GC or unsupported version-two catalog publisher. The final requirement reconciliation must preserve these distinctions.

The next parity slice adds the false-profile-boundary twin using the frozen mutation record and production-published constituent chunks. It preserves the exact expected 262,143-byte first boundary versus the observed 262,144-byte boundary before output. Immediate writer errors, impossible write counts, absent range blobs and absent reconstruction layouts also have direct public durable witnesses. The complete public integration target passes debug/release, Clippy and source structure (`read-parity.log`).

Hosted validation of `cc189ff` caught a formatting-only error in the module declaration order: the earlier copy-back omitted the formatted `suite.rs`. Its Rust quality gate failed; the other required jobs passed. The author corrected the module list and records this as a validation-transfer mistake, not a runtime failure or a green final-head receipt. Subsequent validation checks the copied source against the committed files.

The writer-parity calibration uses an isolated source and target directory. It substitutes an impossible maximum of zero and wraps the immediate I/O cause as `Other`; the four whole/range assertions fail on those specific wrong public outcomes (`writer-parity-mutation-red.log`). The original source passed the same assertions in debug/release. This is assertion calibration for the new API, not a parent regression: the durable API does not exist on main.

## Range-domain and overlap closure

The range continuation adds a direct physical-corruption range witness, the reference suite's fixed patterned multichunk source/coordinate domain, and a selected-member-only catalog. The last case deliberately cannot reconstruct its whole layout: the absent first chunk produces an exact `ChunkMissing` with untouched output, while the interior range succeeds with exact source bytes and receipt coordinates. The comparison is one proof-scope behavior, not a count of harness cases. Publication and migration use production capabilities; no retained root falsely claims closure over the intentionally incomplete layout.

The complete Worldline target passes debug/release, all-feature Clippy and source structure (`range-overlap-parity.log`). Earlier generator-only and physical-corruption receipts are retained separately. The finite affine coordinate domain is replayable from source but does not claim a general shrinking framework; the exhaustive short-domain run remains ordered by increasing length. This narrows the remaining acceptance work without claiming arbitrary-input exhaustiveness.

An isolated source/build mutation made range verification start with all layout entries instead of the selected entries. The overlap law went RED at the read boundary with an exact missing first-chunk refusal (`range-overlap-mutation-red.log`); the original candidate succeeds on the interior range. This calibrates logical overlap independence without using internal lookup counters as its oracle.

## Acceptance documentation candidate

The Linux README example mirrors a compiled `DurableStore` doctest and names an explicit 16 MiB application admission budget. Normative documents now identify the available API, typed outcomes, view coordinates, managed-namespace fence scope and materialization costs. Requirement rows 009 and 010 identify the candidate implementation and explicitly retain final #109 acceptance as pending; they do not certify an unreviewed release.

Review queue inspection found no submitted review bodies or inline threads; the sole top-level comment reports that CodeRabbit skipped this draft. All retrieved connections were exhausted. Independent review must cover the whole diff, the raw receipts and the scope distinctions above; no absence of comments is treated as approval.

## Independent review and retained-closure correction

Independent Codex review of `dd42dcbede0316fc64487283b45a78e7a4a0cac6` returned REQUEST CHANGES for evidence gaps, with no demonstrated production defect. The [full findings and checklist](https://github.com/flyingrobots/keep/pull/164#issuecomment-5962524242) were posted before remediation. Agy exhausted its quota without a verdict; it supplies no approval. All required hosted checks passed on the reviewed head, which does not transfer to subsequent changes.

The first finding is addressed by `an_unsatisfied_retained_closure_refuses_snapshot_admission_before_output`. A production-admitted ext4 store contains a canonical layout but lacks its chunk. The fixture deliberately installs checksummed root/manifest/head evidence claiming that incomplete closure, outside publication; this is adversarial persisted state, not an assertion that production preflight permits it. Snapshot admission and the convenience reconstruction both preserve the exact namespace and `MissingMember::Chunk` coordinates. Caller output and selected root/manifest/head bytes remain unchanged.

Focused debug/release, all-feature Clippy and source structure pass (`closure-refusal-corrected.log`). The initial Clippy tuple-complexity failure is preserved separately as authoring evidence. An isolated source and target bypasses closure verification only when a manifest is present, leaving all earlier physical/canonical admission intact; the new test was observed RED with “incomplete retained closure admitted a snapshot” (`closure-admission-mutation-red.log`). This directly challenges the new owning boundary rather than only the shared closure verifier. The remaining review finding requires the consolidated refusal-calibration map and its missing observations before re-review.

## Refusal calibration requested by independent review

The remaining explicitly identified refusal claims were challenged in isolated source/build copies of `4d801e491eebc580866d6fa1c18d1365035fa3a9`. These mutations change production behavior or returned diagnostics, not test expectations. `refusal-calibration-green.log` records the unmodified layout laws passing in debug/release and the retained-closure law passing again. Each mutant compiles and reaches the named runtime check; build failures are not counted.

| Protected claim / assertion | Deliberate violation | Observed RED receipt |
| --- | --- | --- |
| Wrong whole-blob claims refuse before output | Skip complete-object verification for nonempty layouts | `proof-and-semantic-binding-red.log`: `false blob claim succeeded` |
| Content-correct false profile boundaries refuse | Same skipped complete-object verification pass | `proof-and-semantic-binding-red.log`: `false profile boundaries reconstructed` |
| Semantic range ingress requires exact catalogued binding | Execute the range core directly on the supplied layout | `proof-and-semantic-binding-red.log`: `uncatalogued layout succeeded` |
| Canonical record range ingress requires exact binding independently | Bypass binding only in record ingress; semantic ingress remains unchanged | `record-binding-red.log`: first semantic refusal passes, then `uncatalogued record succeeded` |
| Whole-record checksum refusal preserves expected/observed coordinates | Swap decoder checksum coordinates | `proof-and-semantic-binding-red.log`: whole-record checksum assertion fails |
| Range-record checksum refusal preserves coordinates independently | Swap checksum coordinates only at range ingress | `range-checksum-red.log`: whole-record assertion passes, then range-record checksum assertion fails |
| Snapshot admission proves selected retained closure | Skip closure verification only for a present manifest | `closure-admission-mutation-red.log`: `incomplete retained closure admitted a snapshot` |

The grouped proof mutation falsifies two distinct promised outcomes: complete identity and storage-profile admission. The record-only mutations deliberately leave the earlier semantic/whole assertions intact so their failures cannot mask the later entrypoint assertions. The successful ingress-equivalence law remains green in all three layout mutation copies, demonstrating that ordinary successful reads alone would not detect these omissions.

The earlier calibration receipts cover the other established claim families: emitted bytes and generated source-slice oracles (`emission-mutation-red.log`, `worldline-emission-mutation-red.log`); view generation (`coordinate-mutation-red.log`); actual collector exclusion (`fence-mutation-red.log`); accepted-prefix accounting (`output-prefix-mutation-red.log`, `range-zero-isolated-mutation-red.log`); immediate writer cause and maximum write count (`writer-parity-mutation-red.log`); additional whole-blob allocation (`read-memory-mutation-red.log`); and logical overlap independence (`range-overlap-mutation-red.log`). Historical witnesses retain their recorded source coordinates; the final independent review must assess whether the combined mapping satisfies the binding standard. This table does not turn unexecuted individual diagnostic-field mutations into evidence.
