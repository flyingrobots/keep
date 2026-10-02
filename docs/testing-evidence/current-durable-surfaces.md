# Current durable surface documentation (#130)

Change kind: documentation correction; no runtime, public signature, identifier, codec, format byte, or test expectation changes.

## Source boundary

This reconciliation uses main integration `6051abb25a9fd33ae7ee0de5614514b709a4d82a`, which merged PR #99. Prepared implementation on `feature/roadmap-m4-tasks` and unmerged audit PRs do not establish delivery on this baseline. The source links below locate each claim; this is a reviewed documentation record, not a source-string assertion masquerading as runtime evidence.

| Claim | Owning source or evidence | Documentation disposition |
| --- | --- | --- |
| Migration can resume admitted residue under writer authority | [Migration recovery](../../src/adapters/store_migration/filesystem_migration_recovery.rs), [requirements](../formats/segment-store-v2/requirements.md#migration) | Preserve the implemented migration claim and the separate #111/#112 coverage gaps. |
| Retention recovery executes complete-stage plans and preserves incomplete stages | [Filesystem recovery](../../src/adapters/retention/filesystem_retention_recovery.rs), [planner](../../src/adapters/retention/recovery_planner.rs), [accepted landing evidence](retention-landing.md) | Remove obsolete absence and pending-#99 claims; keep incomplete-stage disposition deferred to #155. |
| Execution failure is distinct from pre-effect refusal | [Execution](../../src/adapters/retention/recovery_execution.rs), [progress](../../src/adapters/retention/retention_storage_progress.rs), [contract](../formats/segment-store-v2/retention-recovery.md) | Preserve typed causes, known/uncertain effects, and durability distinctions. |
| Retention snapshots bind a fenced view and admit selected roots | [Snapshot](../../src/adapters/retention/filesystem_retention_snapshot.rs), [collector](../../src/adapters/retention/retention_view_collector.rs) | Describe the delivered surface without promising the absent durable authenticated read API (#109). |
| Forward retention publication and complete-root recovery reverify live closure | [Closure admission](../../src/adapters/retention/filesystem_retention_closure_admission.rs), [forward verification](../../src/adapters/retention/filesystem_retention_catalog.rs) | Remove the stale planned member-reverification claim. |
| General candidate-catalog admission must preserve every retained root | [Catalog preflight](../../src/adapters/filesystem_catalog_current.rs), [catalog storage](../../src/adapters/filesystem_catalog_storage.rs), [publication requirement](../formats/segment-store-v2/retention-publication.md#closure-admission) | Keep the normative requirement; explicitly identify its implementation gap in #125. Current-root verification against the current catalog does not satisfy this different requirement. |
| Durable reads, reports, ingestion, GC, and compaction are not delivered on this baseline | [Crate public surface](../../src/lib.rs), [GC ledger](../formats/segment-store-v2/requirements.md#garbage-collection-reservation) | Track #109, #114, #82 and #21; do not import PR #107's prepared implementation claims. |

The managed-namespace concurrency contract remains the [accepted decision](../adr/retention-bounded-recovery-landing.md): cooperating writers under Keep authority, without a guarantee against arbitrary concurrent raw namespace mutation.

## Validation and limitations

The oracle is correspondence between public documentation and the source at the named integration, reviewed at each owning boundary above. Documentation integrity/refusal/link checks, Markdown lint, rustdoc generation and doctests validate document structure, references and compiling examples. Formatting and Clippy remain required. Their success does not prove storage behavior.

No runtime assertion was introduced or materially changed, so RED-on-parent, runtime mutation calibration, fault injection, generated-case reduction and new test resource ceilings are not applicable to this documentation-only change under the [enforcement profile](../testing/enforcement.md). Existing runtime evidence remains attributed to its original source and scope; this PR does not rerun or expand it. The final PR receipt records commands, source SHA and actual check results.

No acceptance criterion is reduced. No original ROADMAP task is newly checked. The #130 request for a source-based documentation regression is satisfied by this explicit claim/source reconciliation and existing documentation checks; a permanent test searching for prose or implementation tokens would assert document structure, not Keep runtime behavior.
