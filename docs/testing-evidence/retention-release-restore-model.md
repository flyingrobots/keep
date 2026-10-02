# Retention release and restore model evidence

This change closes the missing release/restore exploration from #128 and original completed T-20.1. Change kind: correction of missing verification and an oracle improvement, with no production behavior change. Owner: `@flyingrobots`. The branch starts at main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`, which integrates the prerequisite #99. Original roadmap checkboxes and definitions of done remain unchanged; this document does not claim mainline integration before its PR merges.

## Contract and oracle

The model explores every length-three history over initial publication in namespaces A and B, a successor in A, release of A's anchors, restoration of A's original anchors, byte-identical retry, and stale initial publication. Seven choices at each position give 343 explored histories. This is an exploration bound, not a case-count assertion or proof of correctness. Operations without their prerequisite namespace or prior publication make no request; those precondition no-ops are part of the declared history space, not successful release witnesses.

Every history starts with fresh migrated filesystem storage. After each step, the fenced reader's complete namespace-to-generation map, liveness generation, selected root generations and anchor sets must equal the reference model. The reference model advances from the requested operation and its own prior state: release chooses an empty set, restore chooses the independent initial fixture's anchors, successors increment the model generation, and retries/refusals preserve state. It no longer accepts candidate output as its expected generation or anchor set. Existing exact repeated-initial and superseded diagnostic checks remain in place.

The space includes `Initial(A), Release, Restore`, which witnesses a nonempty anchor set becoming empty at generation two and returning at generation three. It also mixes release/restore with namespace B, retries and stale callers, so unrelated namespace preservation and exact failure outcomes remain observed. Diagnostics include the complete deterministic schedule. The fixture and codecs remain shared foundations; this model does not independently re-prove their binary specification, which has separate golden and corruption evidence.

## Calibration and observed results

The parent implementation already supports these operations; there is no claimed production bug fix or fabricated parent runtime RED. A copied production reader mutation instead hides selected roots whose anchor set is empty. The original parent model suite passed that mutant. The expanded suite failed it, including the concrete history `Initial(B), Initial(A), Release` with `model namespace has no root on disk`. This demonstrates the previous coverage gap through observed product output.

Two separate oracle calibrations supplied canonically valid but semantically wrong candidates while leaving the reference model unchanged. A release candidate retaining the old anchors failed with observed nonempty anchors versus expected empty. A restore candidate remaining empty failed with observed empty anchors versus the original fixture anchors. These are calibration of oracle independence, not claims that the unmodified production publisher should reject a valid empty-root request. All mutations used separate source/build directories and were excluded from the candidate.

The unmodified production implementation passes the expanded histories in debug and release. Formatting and warnings-denied Clippy pass. The PR records the immutable candidate SHA, full validation and final hosted checks; historical #99 evidence still describes its earlier, smaller operation alphabet and cannot be relabeled as this expanded campaign.

## Execution, replay and limits

Replay `cargo test --lib --all-features --locked retention_model_tests` and its `--release` variant in copied Docker source with pinned Rust 1.96.0. The tests are medium-size filesystem experiments using owned per-history scratch. They retain the existing model fixture's repository-only migration/admission setup, so they verify post-admission retention behavior and do not prove production platform eligibility. No host Rust execution or writable host checkout mount is used.

There is no random seed: the alphabet and depth are fixed source inputs. The reported three-operation schedule is directly replayable; removal of precondition no-ops reduces the calibration witnesses to `Initial(A), Release` and `Initial(A), Restore`. No random corpus or fuzz campaign is claimed. Histories longer than three, arbitrary anchor sets, namespace exhaustion, concurrency schedules, new fault injection and physical power loss remain outside this model change. Existing tests for those distinct contracts are preserved.

The sequence index is only a scratch-name coordinate and uses checked arithmetic. No assertion freezes a harness count. Ordinary per-test resource ceilings and suite-budget gaps remain as disclosed in the binding testing enforcement profile; no new enforcement or latency SLO is claimed. Retire this coverage only if the operation contract disappears or stronger model exploration demonstrably subsumes it. No identity, format, dependency, synchronization, recovery or performance behavior changes.
