# GC Execution and Recovery

This page owns the protocol that retires released segments from a
`keep.segment-store/v2` root: the fixed phases, the residue each phase leaves,
the one lawful recovery of each residue, and the process-death matrix that
proves it. The record grammars and the planner are owned by the
[GC specification](gc.md); the design contract is ADR-0009.

> **Warning.** Execution unlinks immutable segments. It holds writer
> authority and the exclusive reader fence, writes and synchronizes
> `gc/intent` before the first unlink, acts only on a plan it has re-proven
> against the reopened store, and is verified afterwards by `recover`.
> `plan_gc` is the dry run: it changes nothing on disk.

## Authority and re-proof

`FilesystemGcAuthority::open` pins one admitted version-two root under the
writer lock. `prepare(&GcPlan)` reads `gc` and refuses unless the residue is
idle or complete; refuses a plan with no candidate (nothing to do is not an
intent); acquires `reader.lock` exclusively without waiting, refusing
`ReadersActive` while any reader holds the shared fence; re-observes liveness
under that fence and requires `plan_gc` over the reopened store to equal the
plan exactly, refusing `PlanStale` otherwise; then derives the one canonical
intent. Its generation succeeds the prior receipt's or is one; each candidate
carries the digest of the durable record that released it (the predecessor
catalog or the exact disposition receipt); the catalog-successor proof, pool
identity, and disposition-set digests are the derivations registered in
`definition.tsv` as `keep.gc-catalog-successor-proof/v2`,
`keep.gc-segment-pool/v2`, and `keep.gc-disposition-set/v2`. `execute` is
`prepare` followed by every phase. While `gc/intent` is durable, retention
publication refuses `GcIntentRetained` and another retirement refuses
`RecoveryRequired`.

## Phases

`GcExecutionPhase::ALL` fixes the order; `GcExecutionStorage` is the
blocking port, one capability per phase, and `execute_gc` and
`resume_gc_execution` drive it from the start or from any point.

<!-- markdownlint-disable MD013 -->

| Identifier | Phase | Effect |
| --- | --- | --- |
| `KEEP-CRASH-074` | `write-intent-stage` | exclusively create `gc/intent.next` with the complete intent |
| `KEEP-CRASH-075` | `sync-intent-stage` | synchronize and reverify the stage |
| `KEEP-CRASH-076` | `link-intent` | link the stage to `gc/intent` without replacement |
| `KEEP-CRASH-077` | `sync-gc-after-intent` | synchronize `gc`; the intent is now durable |
| `KEEP-CRASH-078` | `remove-intent-stage` | remove `gc/intent.next` after proving its link |
| `KEEP-CRASH-079` | `sync-gc-after-intent-cleanup` | synchronize `gc` |
| `KEEP-CRASH-080` | `unlink-candidate` | reopen one candidate without following links, verify kind, length, and admitted digest, unlink it (once per candidate, canonical order) |
| `KEEP-CRASH-081` | `sync-segment-pool` | synchronize `segments` (once per candidate) |
| `KEEP-CRASH-082` | `write-receipt-stage` | prove every candidate absent, read the pool, create `gc/receipt.next` with the receipt over the exact remaining inventory |
| `KEEP-CRASH-083` | `sync-receipt-stage` | synchronize and reverify the stage |
| `KEEP-CRASH-084` | `replace-receipt` | rename the stage onto `gc/receipt`, replacing the prior retirement's receipt atomically |
| `KEEP-CRASH-085` | `sync-gc-after-receipt` | synchronize `gc` |
| `KEEP-CRASH-086` | `remove-intent` | remove `gc/intent` after proving `gc/receipt` completes it |
| `KEEP-CRASH-087` | `sync-gc-after-intent-removal` | synchronize `gc`; the retirement is complete |

<!-- markdownlint-enable MD013 -->

The receipt replaces by rename, as `retention/HEAD` does, so `gc/receipt`
always holds the last completed retirement and the next intent's generation
is its successor. The receipt's synchronization count is one per candidate.

## State and recovery

`GcResidue` is what restart reads: every `gc` record as it is, bounded one
byte past its maximum length, and the presence of each candidate the durable
intent names. `plan_gc_recovery` is pure; `FilesystemGcAuthority::recover`
reads the residue, plans, and acts.

<!-- markdownlint-disable MD013 -->

| State | Evidence | Recovery |
| --- | --- | --- |
| idle | no `gc/intent`, `gc/receipt`, or stage | nothing; no retirement authority |
| complete | exact `gc/receipt` only | return the receipt |
| staged | complete `gc/intent.next`, no `gc/intent` | resume at `sync-intent-stage` |
| linked | `gc/intent` and byte-equal `gc/intent.next` | resume at `sync-gc-after-intent` |
| active | exact intent, every candidate present | resume at `sync-gc-after-intent-cleanup` |
| partial | exact intent, one canonical absent candidate prefix | resume at `unlink-candidate` for the first present candidate |
| completion pending | exact intent, every candidate absent, no receipt for it | resume at `write-receipt-stage` |
| receipt staged | completion pending beside a receipt stage that completes the intent | resume at `sync-receipt-stage` |
| receipt transition | exact intent and the receipt that completes it | resume at `sync-gc-after-receipt` |
| truncated stage | an intent stage before any authority, or a receipt stage after completion pending, shorter than its record | discard the stage, synchronize `gc`, replan |

<!-- markdownlint-enable MD013 -->

A `gc/receipt` whose generation the durable intent succeeds is the prior
retirement's and constrains nothing. Every other residue is a typed
`GcRecoveryAmbiguity` and nothing is touched: an undecodable intent, a
receipt that neither completes nor precedes the intent, a receipt stage
without an intent, a stage holding other bytes or longer than its record, an
absent candidate after a present one, or a receipt or receipt stage while a
candidate is still present. Recovery never guesses which deletion occurred.

## Process-death matrix

`cargo xtask durability-crash-matrix --sequence gc` runs 42 cases: the 14
boundaries above at before, during, and after positions, over a migrated
bundle store with retention generation one published and the one-zero
segment added as an orphan pool entry released by its exact retire
disposition. An isolated child plans, prepares, and executes with the
selected boundary gated and is killed by its process group. The parent then
requires the live bundle segment byte-identical, reopens the root for
recovery the way a restarted writer would, requires the production planner to
report exactly the row above (a truncated stage first plans its discard),
runs the recovery (or the forward retirement after an idle residue), and
requires one complete retirement: `gc` holds only the receipt, the orphan is
absent, the live segment is intact, a fresh plan names nothing and reports
the orphan already retired, and a second recovery reports `Complete`. The
[transitions ledger](../../../conformance/segment-store/v2/transitions.tsv)
records rows `KEEP-CRASH-074` through `-087`. In-process, the same law runs
over every prefix of the 14 points and both truncated stages in
`src/adapters/gc/filesystem_gc_tests.rs`. The matrix proves application
process death; host power loss remains outside its claim.

Not implemented: identity-preserving compaction (`named-unreachable`
material can only be released by a compaction successor), background
scheduling, and secure erasure; issue #21.
