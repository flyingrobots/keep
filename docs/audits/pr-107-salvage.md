# PR #107 salvage ledger

PR #107 (`feature/roadmap-m4-tasks`) was closed without merging on
2026-10-20. In the meantime main had re-implemented several of its features
in separate PRs, and merging main into the branch conflicted in 93 files whose
two sides implement the same features differently. This page records, for
every change the branch carried that main did not, whether it is kept and how.
It is a decision record, not a completion claim: a row marked *ported* names a
salvage branch, and the work counts as delivered only once that branch merges.

## Preservation

Nothing on the branch is deleted. Two annotated tags pin the source material
independently of branch lifetime:

| Tag | Commit | Contents |
| --- | --- | --- |
| `archive/pr-107-feature` | `57b6c37` | The PR head at closure, including #133 merged into it. |
| `archive/pr-107-landing` | `f21589f` | The `landing/107-integration` branch: later review fixes and the unfinished disposition RED checkpoint. |

The branch merge base with main is `c9277ea` (2026-09-09). At closure the
branch had 54 commits not on main; main had 360 not on the branch.

## Verdicts

*Superseded* means main delivered the outcome independently. Main's design
wins and the branch version is not ported. *Ported* means a salvage branch
re-implements the change on current main. *Deferred* means the change is worth
keeping but cannot land without a named prerequisite or decision. *Dropped*
means it has no remaining value.

### Superseded on main

| Branch commits | Outcome | Delivered on main by |
| --- | --- | --- |
| `68b81ee` | Restart-stable root comparison on v2 reopen | #137 |
| `2bce29d`, `ce3bc67`, `65ed69e`, `925d592` | Migration recovery planning, prefix recovery, crash matrix | #138, #161, #162 |
| `4b32c02`, `03dc989` | One hash per chunk per reference read; benchmark counters | #134 |
| `132b1c7` | Reference staging memory ceiling | #135 |
| `34732b2` | Merge of #99 into the branch | #99 merged directly |
| `0bafaa7` | Durable authenticated reads over a fenced snapshot | #164 (different module layout) |
| `420964e` | Retention release/restore model sequences | #159 |
| `a5c6701`, `fed314e`, `1a586d8` | Living-page and status reconciliation | #130, #136 |
| `5a23238` | Explicit-depth verification reports | #165 for durable views; the branch's reference-view report conflicts with #165's model (ordinal depths, range of supported depths) and is deferred below |

### Ported to salvage branches

| Branch commits | Salvage branch | Changes from the branch |
| --- | --- | --- |
| `0d300db`, part of `ee4ef54` | `salvage/107-content-store-port` | `contains_blob` is fallible to match main's durable snapshot; generic laws assert typed refusals through per-backend oracles; `DurableSnapshot` implements the read half; the `ChunkSink` generalization waits for durable ingestion. |
| `1dbbf7f`, `80c8042`, `652059e` (#127) | `salvage/107-transfer-pipeline` (stacked on the port) | Starts from the #127 inward shape; `ChunkReader` lives in the shared read core; the pipeline lives at `src/pipeline` because it imports only ports; `DurableSnapshot` is a transfer source; the source-text architecture test is not carried over (#168). |
| `29690fc`, `6a84324`, `b5ecd97` | `salvage/107-retention-mutation-laws` | Root, manifest, and head field-mutation matrices for KEEP-RETENTION-003; helpers added to main's newer `tests/support/byte_patches.rs`. |
| landing `494e193`, `691e454`, `66384de` | `salvage/107-recovery-sync-outcomes` | Shared v1 recovery: directory-sync errors keep the removal or finalization outcome. Main's affected files were byte-identical to the fix's parent, so the defect is live on main. The landing evidence receipts are replaced by fresh evidence on main. |
| `e4fb098` and codec review fixes `fb9f129`, `b1eaa59`, `f00e780`, `a62bfb8`, `5b533b0` | `salvage/107-gc-retirement-codecs` | GC retirement intent and receipt codecs only; no planning, execution, writer, or change to the v2 format-definition digest. Execution-only additions from `9a10b22` (`decode_unbound`, `from_fields`, `candidate_set_digest`) and every disposition part are stripped. Review finding: `docs/formats/segment-store-v2/gc.md` does not state that the candidate-set digest hashes a big-endian `u32` candidate count before the entries; the codec and the independent oracle agree, so the page needs that sentence. |
| `66c0e46` (#133) | `salvage/107-typed-refusals` | Re-derived against main's rewritten retention and migration code for the erasures still present on main; runtime downcast laws replace the branch's source-text contract test. Same-pattern sites outside the audited list remain: `sync_capable_directory.rs`, `filesystem_initialization_namespace.rs`, `filesystem_recovery_stage_materialization.rs`, `observed_segment_stage.rs`, `filesystem_migration_authority.rs`, and the lossy `read_exact_optional` collapse into `RecordKindOrLength`. |
| `24c2d93`, `cec0559` and later roadmap edits | `salvage/107-roadmap` | `ROADMAP.md` with stable F/T identifiers and full task fields, statuses reconciled to main's ledgers. |

### Deferred, with the blocking prerequisite or decision

| Branch commits | Outcome | Blocked by |
| --- | --- | --- |
| `668fd5f`, `5bba27e` | Deterministic GC planning from a fenced liveness snapshot | Main lacks `CatalogSnapshot::record_segments` and retained-closure member verification; main's snapshot load adds a root-identity check the planner must adopt. Lands after the codecs. |
| `9a10b22`, landing `8828db1`, `20dec6d`, `f6aca41` | GC execution, retirement, recovery, crash rows `KEEP-CRASH-074`–`087`, failing-capability reporting | Exclusive reader-fence API absent on main; typed `RetentionStorageError`; KEEP-GC-001 namespace admission must change from "empty `gc/`"; the `gc/intent.next` and `gc/receipt.next` stage names and rename-replacement of the receipt need a protocol ruling against main's "fixed names are never replaced" rule. |
| `2859f68` | Disposition enumerations and receipt codec | **Format decision.** The branch adds seven rows to `docs/formats/segment-store-v2/definition.tsv`, changing `StoreFormatDefinitionDigest::VERSION_TWO` from main's `32381f1a…` to `a4a010ce…`. Stores migrated by either side would refuse the other. v2 is unreleased, so a bump is possible, but only as an explicit decision record. |
| `aa371ae`, landing `f21589f` | Explicit finalize-or-retire disposition of protected retention orphans | The format decision above; main's retention authority and storage port changed; overlaps #155. The landing RED test is reusable as a test idea only. |
| `393accb`, landing `43e20c4`, `5a67c24`, `5f37c7e`, `c5b3e34`, `e49fde5`, `9731d50`, `92aafcd` | Identity-preserving compaction and its later recovery fixes | GC planning (compaction imports `plan_gc` and the liveness observation); v2 publisher and stage-discarder entry points absent on main. |
| `ee4ef54`, `d0998b3` (#129) | Durable staged ingestion with deduplication; bounded sealed-stage reads | #125: the writer publishes through low-level `publish_catalog_generation`, which has no retained-closure gate on main. Also needs `FilesystemCatalogPublisher::open_version_two` and the split `close_sealed`/`select_closed`. #129 is moot on main (main's reads are bounded) and travels with this port. |
| `3aebec4` | Durable, replayable verification receipts (`KEEP:VERIFY:RCPT` v1) | Its 384-byte record encodes the branch's report vocabulary, which #165 rejected. It needs a decision record and a redesign against main's report types; main's #165 scope excludes report serialization. |
| `5a23238` (reference-view part) | Explicit-depth verification of `ReferenceStore` | Rewrite onto main's `VerificationReport`, subjects, and exact supported-depth sets; it would widen public non-exhaustive enums. |
| `5f49800` | Permanent corruption ledgers over every durable structural field | The v1 ledger and the 99 v2 rows for records main has can be ported as tests only. The 51 GC and disposition rows wait for those codecs. The expected outcomes are derived from `Debug` names and must be re-derived on main. This is not evidence for #115, which needs report-level assertions. |
| landing `acfb0e9`, `cdc7b64` | Shared stage creation and receipt-effect reporting | GC storage; re-derive against main's typed retention errors rather than cherry-pick. |
| `92f3eb4` | Repinned v2 fixture digests after the `definition.tsv` additions, plus crash-point table fixes | Travels with the format decision above; it is not a lint-only change. |
| `c7f9f57` | Clippy fixes to the corruption-ledger and receipt test crates | Travels with the corruption-ledger and receipt ports. |
| `1de4f66`, `666922a` | Crash-point table and GC/compaction doc-link fixes | Travel with GC execution and compaction. |
| `05b804b`, `7af834a`, `9a5fb8f`, `65c0707`, `e8ae339` | Review evidence for the GC codecs, reader-lock coordinates, fixture provenance, and GC retention oracles | Travel with the GC codec and planning ports; each records calibration against branch-only code and must be re-established on main rather than copied. |
| `7cdc2cf`, `4158001` | Requirement-ledger wording for KEEP-RETENTION-003 and KEEP-GC-001 that cites the branch-only `mutations.tsv` ledger | Travel with the corruption-ledger port; the mutation-law salvage writes its own KEEP-RETENTION-003 evidence. |
| `4b9c389` | The 2026-09-30 completed-roadmap audit and its GitHub ownership index | Its findings are the source of #110–#131. Where it lives depends on the open decision about #131 and #141; it remains readable in the tag. |

### Dropped

| Commits or files | Reason |
| --- | --- |
| landing `43eaeb3`, `34db571`, `864f804` | Re-integration merges of #133 and an older main; superseded by the typed-refusal salvage and by main itself. |
| `docs/dependencies/{blake3-1.8.5,cap-std-and-cap-fs-ext-4.0.2,yaml-rust2-0.11.0}.md` | Main admits newer versions. |
| `docs/architecture/durable-store/{README,rationale}.md` | Describe the branch's durable read API; main documents its own in `src/adapters/durable/rationale.md` and the reconstruction ledger. |
| `docs/audits/pr-landing-*` and `pr-landing-evidence/107-integration/*` on the audit branch | Session process logs for a merge queue that no longer exists. They are kept in the tags and the audit branch only. |
| Branch edits to `README.md`, `CHANGELOG.md`, and requirement ledgers | They would revert main's delivered state (for example KEEP-VERIFY-006 back to Planned). Only the entries belonging to a ported change travel with that change. |

## Ordering

The deferred rows form this prerequisite graph. Each edge answers the
question: which PR must already be merged for this one to be correct?

- GC retirement codecs → GC planning → GC execution and recovery.
- GC planning → compaction.
- Format decision on `definition.tsv` → disposition codec → explicit disposition.
- #125 retained-closure gate → durable ingestion → ingestion-dependent issues
  (#120 durable half, #121–#124).
- Content-store port → transfer pipeline; durable ingestion → copy into a
  durable destination.

The ported branches are independent of each other except that the transfer
pipeline is stacked on the content-store port.
