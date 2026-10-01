# Completed roadmap audit, 2026-09-30

Scope: the 83 task entries checked at the start of this audit, in roadmap
order. Initially unchecked tasks were excluded. Passing entries retain their
checks. Acceptance criteria and definitions of done remain authoritative;
this audit does not substitute a smaller scope for either.

This pass is incomplete. The corrective changes below have regression
evidence, but the remaining obligations prevent certifying every checked
task. Unlisted tasks are not certified by omission or by a green workspace
test run. External issue closure, PR integration, and consumer cutover are
separate evidence from local implementation.

## Corrective changes

| Task | Gap found and correction |
| --- | --- |
| T-01.2 | Durable admission, selected-root reads, and compaction recovery erased typed sources. Preserve the original errors; downcast regressions cover durable admission and substituted root records. Other string-based refusals still need review. |
| T-06.3 | Benchmark counters still charged two hashes per chunk. Correct whole and range counters and freeze a clean-source Rust 1.96.0 baseline; differing hosts preclude a CPU speedup claim. |
| T-10.2 | Reference code imported adapter-owned transfer ports. Move the semantic port inward; preserve external exports and add an architecture regression. |
| T-19.1 | Missing subprocess fence-release and exclusive-fence exclusion laws remain. Experimental tests passed, but violated the existing prohibition on spawning processes under `src`; they were removed. Permanent evidence belongs in the external crash harness. |
| T-20.1 | The 125-sequence model omitted release. Expand to 343 sequences including release and restore, with fenced observations after each operation. |
| T-24.2 | Mutable sealed stages were read without a length bound. Admit the recorded exact length before allocation; externally growing a stage now produces the typed stage-read failure and preserves the head. |
| T-38.2 | Crate documentation incorrectly called implemented recovery, reads, and GC absent. Correct the claims and add a documentation regression. |
| T-38.3 | Version-two pages contradicted implemented publication and recovery. Reconcile them and explicitly document the outstanding general retained-closure publication gate. |

## Known acceptance and definition-of-done failures

| Task | Remaining obligation |
| --- | --- |
| T-01.2 | Audit remaining string-created operational/refusal errors against the universal typed-refusal requirement. |
| T-06.3 | The literal definition of done requires merged regression work; local source and benchmark evidence alone do not prove integration. |
| T-14.1 | Definition of done requires issue #69 closed; it was open when checked. |
| T-17.1 | Definition of done requires issue #97 closed; it was open when checked. |
| T-17.2 | KEEP-MIGRATION-005 remains In progress: restart corruption and mutation matrix. |
| T-17.3 | KEEP-MIGRATION-008 remains In progress: remaining compatibility and fuzz matrix. |
| T-18.1 | Definition of done requires PR #99 merged; it is open. |
| T-18.2 | The 51-case retention crash matrix exists; update its stale ledger row. PR #99 integration remains unproven. |
| T-19.1 | Permanent process laws are missing, and PR #99 must be merged. |
| T-20.1 | Expanded model coverage does not satisfy the requirement that PR #99 be merged. |
| T-21.1 | Durable segment/catalog/retention verification report interfaces and depths remain planned as KEEP-VERIFY-006. |
| T-21.2 | Decoder mutation evidence does not establish each required durable report variant and achieved depth through the report interface. |
| T-22.3 | Compaction amplification, sync, reclaimed-byte, latency, and temporary-space benchmarks and dedicated process-death evidence are missing. Re-encoding is outside this task's original scope. |
| T-22.4 | Explicit 65,536-candidate stress evidence is missing; retirement crash evidence does not replace that acceptance check. |
| T-22.5 | Dedicated disposition process-death evidence across stage/link/sync/remove transitions is missing. In-process residue tests are different evidence. |
| T-23.1 | Complete durable Worldline restart/range backend witnesses are missing. |
| T-24.1 | Full required ingestion-law and Worldline execution through both adapters remains missing. |
| T-24.2 | Required rollover, multi-GiB stress, ceiling soak, ingestion-driven crash evidence, benchmark evidence, and Worldline capability coverage remain missing. |
| T-24.3 | Required CPU advantage is not demonstrated; Worldline copy evidence remains missing. Multithreading is outside this task's scope. |
| T-25.2 | Echo issue #722 is open and its plan explicitly says production replacement is not approved. A merged boundary document does not establish adapter cutover. |

## Integration evidence

Read-only tracker checks found Keep issues #69, #71, #72, #74, #82, #97,
issues #108 and #109 open, and PR #99 open with no merge timestamp. No issue was
closed and no PR was merged by this audit. Echo #722 is an external blocker
for the claimed production cutover.

The retained-closure check for arbitrary low-level version-two catalog
publication is still a documented gap. Durable ingestion preserves current
catalog records; compaction verifies retained closures. Neither narrower
guarantee establishes the missing general publication gate.

No format bytes or public export names change in the corrective work.
The new benchmark source is preserved on the local
`audit/roadmap-benchmark-source` branch. Its artifact records full source,
compiler, and host coordinates.

## Validation

Rust 1.96.0 checks passed for formatting, workspace Clippy with warnings
denied, the release workspace suite with all features, source structure,
documentation integrity, stable fuzz-target compilation, dependency audit,
and dependency policy. The Linux ext4 durability crash matrix passed in
debug and release: 105 version-one cases, 51 retention cases, 68 migration
cases, and 42 GC cases. These existing sequences do not establish the
missing ingestion, disposition, or compaction process-death criteria.

The debug workspace suite also passed. Host tools must include `sysctl`
and `b3sum` on PATH for source-bound benchmark and conformance tests.
Linux ARM host capture encountered an unsupported CPU-model coordinate;
that environment failure is not evidence that the entire Linux workspace
suite passed.

## GitHub follow-up ownership

Tracking container: [completed-roadmap audit follow-ups](https://github.com/flyingrobots/keep/issues/132).
It coordinates work and integration gates; it is not another executable PR.
The original acceptance criteria remain authoritative.

| Follow-up | Roadmap ownership |
| --- | --- |
| [Finish typed refusal audit across durable boundaries](https://github.com/flyingrobots/keep/issues/110) | T-01.2 |
| [Complete migration restart corruption and ambiguity matrix](https://github.com/flyingrobots/keep/issues/111) | T-17.2; KEEP-MIGRATION-005 |
| [Complete migration compatibility and fuzz evidence](https://github.com/flyingrobots/keep/issues/112) | T-17.3; KEEP-MIGRATION-008 |
| [Prove reader-fence process death and collector exclusion in external harness](https://github.com/flyingrobots/keep/issues/113) | T-19.1 |
| [Implement durable verification reports at explicit achieved depths](https://github.com/flyingrobots/keep/issues/114) | T-21.1; KEEP-VERIFY-006 |
| [Assert durable mutation failures through verification reports](https://github.com/flyingrobots/keep/issues/115) | T-21.2 |
| [Add dedicated compaction process-death recovery matrix](https://github.com/flyingrobots/keep/issues/116) | T-22.3 |
| [Freeze reproducible compaction amplification and temporary-space baseline](https://github.com/flyingrobots/keep/issues/117) | T-22.3 |
| [Prove GC planning at the 65,536-candidate acceptance bound](https://github.com/flyingrobots/keep/issues/118) | T-22.4 |
| [Add explicit orphan-disposition process-death recovery matrix](https://github.com/flyingrobots/keep/issues/119) | T-22.5 |
| [Run complete ingestion contract and Worldline against both store adapters](https://github.com/flyingrobots/keep/issues/120) | T-24.1; T-24.2 |
| [Implement atomic durable ingestion across segment rollover](https://github.com/flyingrobots/keep/issues/121) | T-24.2 |
| [Prove durable ingestion memory bounds with multi-GiB and ceiling soak evidence](https://github.com/flyingrobots/keep/issues/122) | T-24.2 |
| [Add ingestion-driven process-death and restart matrix](https://github.com/flyingrobots/keep/issues/123) | T-24.2 |
| [Publish source-bound durable ingestion throughput and resource baseline](https://github.com/flyingrobots/keep/issues/124) | T-24.2 |
| [Enforce retained-closure admission for low-level version-two catalog publication](https://github.com/flyingrobots/keep/issues/125) | T-38.3 audit finding; #82 |
| [Add backend-neutral Worldline copy witnesses for transfer pipelines](https://github.com/flyingrobots/keep/issues/126) | T-24.3 |
| [Integrate transfer-source port ownership regression fix](https://github.com/flyingrobots/keep/issues/127) | T-10.2 |
| [Integrate retention release/restore model coverage](https://github.com/flyingrobots/keep/issues/128) | T-20.1 |
| [Integrate exact-length admission before durable stage allocation](https://github.com/flyingrobots/keep/issues/129) | T-24.2 |
| [Integrate corrected crate and version-two implementation documentation](https://github.com/flyingrobots/keep/issues/130) | T-38.2; T-38.3 |
| [Complete acceptance and definition-of-done audit of the remaining 64 checked tasks](https://github.com/flyingrobots/keep/issues/131) | Original completed-task audit remainder |

Existing ownership is retained in [#69](https://github.com/flyingrobots/keep/issues/69), [#71](https://github.com/flyingrobots/keep/issues/71), [#74](https://github.com/flyingrobots/keep/issues/74), [#97](https://github.com/flyingrobots/keep/issues/97), [#108](https://github.com/flyingrobots/keep/issues/108), [#109](https://github.com/flyingrobots/keep/issues/109), [#72](https://github.com/flyingrobots/keep/issues/72), [#20](https://github.com/flyingrobots/keep/issues/20), [#21](https://github.com/flyingrobots/keep/issues/21), [#82](https://github.com/flyingrobots/keep/issues/82).
The tracker separately records [PR #99](https://github.com/flyingrobots/keep/pull/99)
and [Echo #722](https://github.com/flyingrobots/echo/issues/722) as integration
and external gates. No duplicate external issue was created.
