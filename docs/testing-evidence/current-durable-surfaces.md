# Current durable surface documentation (#130)

Change kind: documentation correction; no runtime, public signature, identifier, codec, format byte, or test expectation changes.

## Source boundary

The initial reconciliation used main integration `6051abb25a9fd33ae7ee0de5614514b709a4d82a`, which merged PR #99. This landing updates the current claims to main `2efc131e8466b458088eaf5de0a5981e636d8f85`, including delivered authenticated reads (#164/#109), verification (#165/#114), migration restart/compatibility evidence (#161/#111 and #162/#112), and reader-fence process-death evidence (#160/#113). Prepared implementation on `feature/roadmap-m4-tasks` and unmerged audit PRs do not establish delivery on this baseline. The source links below locate each claim; this is a reviewed documentation record, not a source-string assertion masquerading as runtime evidence.

| Claim | Owning source or evidence | Documentation disposition |
| --- | --- | --- |
| Migration can resume admitted residue under writer authority | [Migration recovery](../../src/adapters/store_migration/filesystem_migration_recovery.rs), [requirements](../formats/segment-store-v2/requirements.md#migration) | Preserve the implemented migration claim and bound additional coverage by the [restart matrix](migration-restart-matrix.md) and [compatibility/fuzz evidence](migration-compatibility-fuzz.md), now merged through #161/#162. These finite receipts do not establish exhaustive hostile-state coverage. |
| Retention recovery executes complete-stage plans and preserves incomplete stages | [Filesystem recovery](../../src/adapters/retention/filesystem_retention_recovery.rs), [planner](../../src/adapters/retention/recovery_planner.rs), [accepted landing evidence](retention-landing.md) | Remove obsolete absence and pending-#99 claims; keep incomplete-stage disposition deferred to #155. |
| Execution failure is distinct from pre-effect refusal | [Execution](../../src/adapters/retention/recovery_execution.rs), [progress](../../src/adapters/retention/retention_storage_progress.rs), [contract](../formats/segment-store-v2/retention-recovery.md) | Preserve typed causes, known/uncertain effects, and durability distinctions. |
| Retention snapshots bind a fenced view and admit selected roots | [Snapshot](../../src/adapters/retention/filesystem_retention_snapshot.rs), [collector](../../src/adapters/retention/retention_view_collector.rs) | Keep the snapshot contract distinct from the now-delivered `DurableStore` composition in #164. [Reader-fence process evidence](reader-fence-process.md) records the merged #160 lifecycle laws, not power-loss guarantees. |
| Forward retention publication and complete-root recovery reverify live closure | [Closure admission](../../src/adapters/retention/filesystem_retention_closure_admission.rs), [forward verification](../../src/adapters/retention/filesystem_retention_catalog.rs) | Remove the stale planned member-reverification claim. |
| General candidate-catalog admission must preserve every retained root | [Catalog preflight](../../src/adapters/filesystem_catalog_current.rs), [catalog storage](../../src/adapters/filesystem_catalog_storage.rs), [publication requirement](../formats/segment-store-v2/retention-publication.md#closure-admission) | Keep the normative requirement; explicitly identify its implementation gap in #125. Current-root verification against the current catalog does not satisfy this different requirement. |
| Durable authenticated reads are delivered | [Durable store](../../src/adapters/durable/store.rs), [snapshot](../../src/adapters/durable/snapshot.rs), [read contract](../invariants/authenticated-reconstruction/README.md), [evidence](durable-authenticated-reads.md) | Describe whole-blob and exact-range proof boundaries, stable locator, fence lifetime and separate allocation limits; #164 supplies mainline integration. |
| Explicit verification reports are delivered | [Report](../../src/verification/report.rs), [durable ingress](../../src/adapters/verification_ingress.rs), [subject/depth matrix](../invariants/verification/README.md), [evidence](durable-verification.md) | Describe named subject/depth evidence and precise non-success outcomes; #165 supplies mainline integration. Reports grant no retention authority or future SnapshotBinding proof. |
| Production durable ingestion, GC and compaction are not delivered | [Crate public surface](../../src/lib.rs), [GC ledger](../formats/segment-store-v2/requirements.md#garbage-collection-reservation) | Track #82 and #21; do not import PR #107's remaining prepared implementations. |

The managed-namespace concurrency contract remains the [accepted decision](../adr/retention-bounded-recovery-landing.md): cooperating writers under Keep authority, without a guarantee against arbitrary concurrent raw namespace mutation.

## Validation and limitations

The oracle is correspondence between public documentation and the source at the named integration, reviewed at each owning boundary above. Documentation integrity/refusal/link checks, Markdown lint, rustdoc generation and doctests validate document structure, references and compiling examples. Formatting and Clippy remain required. Their success does not prove storage behavior.

No runtime assertion was introduced or materially changed, so RED-on-parent, runtime mutation calibration, fault injection, generated-case reduction and new test resource ceilings are not applicable to this documentation-only change under the [enforcement profile](../testing/enforcement.md). Existing runtime evidence remains attributed to its original source and scope; this documentation change does not expand its behavioral claims. The final PR receipt records commands, source SHA and actual check results.

No acceptance criterion is reduced. No original ROADMAP task is newly checked. The #130 request for a source-based documentation regression is satisfied by this explicit claim/source reconciliation and existing documentation checks; a permanent test searching for prose or implementation tokens would assert document structure, not Keep runtime behavior.

## Landing integration

The normal merge retains both CHANGELOG histories, main's expanded reader-fence evidence, all public exports and runtime implementations, and #130's bounded recovery correction. Crate-level edits affect rustdoc only. The old assertions that #109/#114 and #111/#112/#113 remain undelivered are removed from current status; their historical receipts retain their original source and limits.

Final validation and independent review are recorded on the resulting PR head. Main's signed #165 integration tree is the runtime reference; documentation checks and compiling doctests cannot prove new storage behavior, and this PR introduces none.
