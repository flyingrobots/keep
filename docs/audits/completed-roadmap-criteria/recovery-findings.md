# Recovery audit findings in progress

This page records verified findings while auditing originally checked T-13.2 at main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. It is not a completed verdict for `KEEP-RECOVERY-005` through `-012`. The remaining inventory, classification, fingerprint and assessment criteria still require their complete accounting; the audit's completed-verdict count is unchanged.

## Corrupt partial seal can authorize discard

`KEEP-RECOVERY-010` requires every available fixed-framing byte to remain canonical before classifying a partial segment as truncation, with demonstrated corruption remaining a typed refusal. `recovery_segment_classifier.rs::classify_tail` recognizes complete seal magic and returns `RecoverySegmentTruncation::Seal` for a short seal without checking its available version, flags or other fixed fields.

A copied-Docker public API probe starts from the canonical `one-zero-segment.hex` fixture, changes the seal version's low byte from one to two, then retains only the segment through both version bytes. The complete records end at offset 209; the retained seal has 18 bytes. The classifier accepts it as seal truncation even though its available version contradicts the format. The regression compiles and fails with `unsupported seal version admitted as discardable truncation` on the inspected main revision.

A second probe fingerprints those exact contradictory bytes, admits their evidence, calls `assess_recovery_stage`, and obtains a successful `plan_recovery_stage_discard` with reason `Segment(Seal { offset: 209, required: 128, observed: 18 })`. Its failure output records that executable plan. This verifies propagation through public assessment/planning; no filesystem discard was executed and no actual evidence deletion is claimed by the experiment.

[Issue #171](https://github.com/flyingrobots/keep/issues/171) owns one bounded correction for partial-seal fixed framing, precise refusal, downstream discard prevention, canonical-prefix compatibility and parser corpus coverage. A coherent boundary/mutation sweep must cover the fixed-framing family; the fix must not become an open-ended investigation of every possible future completion. The version-one segment protocol is distinct from #99's approved preservation policy for incomplete version-two retention stages.

The existing complete-seal corruption and canonical short-seal examples do not exercise this contradictory short-seal branch. This finding is a runtime correctness defect, unlike #169's acquisition-test evidence gap. The original roadmap checkbox is preserved.

## Incomplete tails bypass known duplicate refusal

`KEEP-RECOVERY-010` also requires duplicate identities to remain typed refusals. On the same main revision, a canonical header followed by two copies of the one-zero record correctly refuses `DuplicateRecordIdentity` at indexes 0/1 and offsets 64/209 when no tail follows. Appending incomplete bytes bypasses `segment_identity_index::validate`: the partial-seal branch returns directly, while truncated record/header errors return through `classify_cursor_error` before the reusable-prefix identity check.

A copied-Docker public runtime probe preserves that no-tail control, then fingerprints, admits, assesses and plans discard for each of three tails after the same duplicate records. All three incorrectly yield executable discard plans at offset 354: a one-byte record header, a 120-byte partial record, and the 16-byte seal magic. The [actual parent RED](recovery-duplicate-tail-red.txt) records each exact plan reason. No filesystem removal was executed, so this evidence establishes erroneous authorization rather than observed deletion.

[Issue #173](https://github.com/flyingrobots/keep/issues/173) owns the bounded correction across all truncation-return paths and the corresponding runtime/generated regressions. It is independent of #171/#172's fixed-seal-framing correction: canonical incomplete seal magic is enough to reproduce this distinct duplicate invariant violation. Integration must preserve both corrections, but shared source alone does not establish a prerequisite. T-13.2 remains in progress and its checkbox and completed-verdict accounting remain unchanged.
