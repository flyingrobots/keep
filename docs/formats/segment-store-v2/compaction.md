# Identity-Preserving Compaction

This page owns compaction for `keep.segment-store/v2`: reclaiming the
unreachable records of a mixed segment by copying its live records into one
new immutable segment and publishing a catalog successor that names the
copies and omits the rest. `BlobId`, `ChunkId`, and `LayoutId` never change:
the successor names the same logical identities at new physical locations,
which is the separation [ADR-0002](../../adr/0002-separate-identity-from-physical-storage.md)
draws (its compaction example moves a record between segments without moving
any identity). The design contract is
[ADR-0009](../../adr/0009-retention-roots-release-and-gc-liveness.md); the
records compaction releases are retired afterwards by
[GC execution](gc-execution.md).

> **Warning.** Compaction publishes a catalog successor and therefore
> changes what the next GC may retire. It holds writer authority, re-proves
> its plan against the reopened store, copies before it publishes, publishes
> through the complete catalog protocol, and revalidates afterwards.
> `plan_compaction` is the dry run: it changes nothing on disk.

## Observation and plan

`observe_compaction` reads one view: every record the current catalog names
with the segment holding it, every record some retained closure reaches
(the same closure walk GC planning uses), and the admitted segment pool.
`plan_compaction` is pure and deterministic over that observation. Every
named segment gets one disposition:

<!-- markdownlint-disable MD013 -->

| Disposition | Meaning | Successor |
| --- | --- | --- |
| `retained` | every named record is live | names the segment unchanged |
| `compacted` | live and unreachable records share it | copies its live records into the new segment, in canonical identity order, and omits the rest |
| `omitted` | no named record is live | omits the whole segment |

<!-- markdownlint-enable MD013 -->

The plan carries the coordinates it was computed under, the successor
generation, the copied record and byte counts, and the reclaimable bytes
(the complete length of every superseded segment). It refuses rather than
plans when no retention head is published (nothing is retained, so
compaction would omit everything), when every named record is live, when a
retained closure reaches an unnamed record, when a named segment is absent
from the pool, when the copies exceed one segment's ceilings, or when the
successor generation would overflow. One compaction fills at most one new
segment; a store with more to reclaim compacts again.

## Execution

`FilesystemCompactionAuthority::open` pins a version-two root through the
catalog publisher. `execute(&plan)`:

1. reopens the store under writer authority and requires
   `plan_compaction` to reproduce the plan exactly (`PlanStale` otherwise);
2. reads every retained segment from the pool and the current catalog;
3. when the plan copies records, creates `staging/current.seg`, appends the
   exact admitted records in canonical identity order, and seals it; the
   copies are byte-identical records, so the new segment's bytes are a
   deterministic function of the plan;
4. builds the successor catalog from the retained segments and the new one,
   at the successor generation over the current digest;
5. runs the complete version-one catalog publication protocol
   (`KEEP-CRASH-001` through `035` boundaries: segment link and pool sync,
   catalog stage, link, and pool sync, `head.next`, atomic replacement, root
   sync), refusing before any stage while a retained stage or `head.next`
   exists;
6. reopens the store, verifies every retained closure, and requires every
   superseded segment to plan as an `unreachable-superseded` GC candidate.

The closure *transcript digest* changes across a successor by design (it
binds the catalog generation and digest); closure membership, root digests,
and every live record's bytes do not.

## Recovery

A compactor may die at any publication boundary. Version-two admission
admits a retained `head.next` beside `staging` residue, readers keep
reading `HEAD`, and every publication refuses until recovered.
`recover_compaction` acquires writer authority and drives the version-one
recovery protocols over the three fixed stages, with the version-two root
entries admitted as inert by the recovery inventory:

<!-- markdownlint-disable MD013 -->

| Residue | Recovery |
| --- | --- |
| truncated `current.seg`, `current.cat`, or `head.next` | version-one evidence-bound discard |
| complete `current.seg` | compaction discard, only after every record proves byte-identical to what `HEAD`'s catalog names; a segment already linked into the pool stays as a valid orphan the next run relinks identically |
| reusable `current.seg` prefix | compaction discard: never-published staging |
| complete `current.cat` | compaction discard, only when it is exactly `HEAD`'s successor candidate |
| complete `head.next` | version-one finalization into `HEAD` |

<!-- markdownlint-enable MD013 -->

After recovery the store re-plans: identically when the successor was not
published, or `NothingToCompact` when it was; either way the same successor
is reached and the same segments are GC candidates. Any other residue is a
typed refusal and nothing is touched.

## Evidence

`src/adapters/compaction/filesystem_tests.rs` over a migrated store whose
second catalog generation adds a segment holding one anchored blob's chunk
and layout beside an unanchored chunk: the plan copies exactly the two live
records and omits the third; execution publishes generation three, keeps
every retained closure's root and members and every live record's bytes,
drops the unreachable chunk from the catalog, and hands the mixed segment to
GC, which retires it; refusals (no retention, nothing to compact, a plan
executed twice) happen before any stage; a death injected before each of
the 22 publication phases recovers to the documented residue outcome and
then to the same successor; recovery over an untouched store is idle. The
Golden File Worldline capability `keep.compaction.identity-stable/v1` is
`required` on that evidence.

Not implemented: a process-death matrix for compaction as its own sequence
(the boundaries are the version-one publication boundaries, proven by the
version-one matrix), amplification and latency benchmarks, and re-encoding
compaction (a representation change), issue #21.
