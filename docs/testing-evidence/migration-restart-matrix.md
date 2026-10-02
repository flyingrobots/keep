# Migration restart ambiguity matrix

This closure ledger records #111 and KEEP-MIGRATION-005. Change kind: missing runtime verification; no production behavior changes. Owner: `@flyingrobots`. Base: main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. Implementation and verification in this branch do not constitute mainline integration; the PR records the exact candidate, required checks and integration status.

## Contract coverage

| Invariant | Filesystem restart evidence | Disposition |
| --- | --- | --- |
| Corrupt intent, marker and receipt refuse | `filesystem_migration_restart_record_tests` damages each staged/canonical record checksum and requires the exact record-specific decoder variant and expected/observed checksum bytes. | Implemented. Existing codec corruption/fuzz owners retain exhaustive field grammar coverage. |
| Overlong stages are not disposed of as partial records | Record laws require exact `StageOverlong` stage and length for all three stages. | Implemented; existing strict-prefix success tests remain. |
| A receipt must bind its own intent | A valid checksummed receipt from another migrated root refuses with `ReceiptUndecodable` / `IntentDigestMismatch` and both digest coordinates. | Implemented. |
| Canonical/stage pairs bind one inode and identical bytes | `filesystem_migration_restart_pair_tests` replaces each linked stage with equal bytes at another inode, requiring `Adoption` / `ExactRecordRefusal::KindLengthOrIdentity`; contradictory bytes require exact `StageDiffers`. | Implemented; removing inode comparison incorrectly admits the substituted stage and fails the law. |
| Effects require durable intent and prior stage cleanup | `filesystem_migration_restart_order_tests` pins `EffectBeforeIntent` and `StageAfterEffect` with exact stage/effect coordinates through reopened authority. | Implemented. |
| Namespace holes, marker before namespace and receipt before marker refuse | Ordering laws cover filesystem-realizable holes with exact absent/present coordinates and both record-order refusals. | Implemented. An absent parent with a present child is not a realizable filesystem case. |
| Persisted root identity is binding | `filesystem_migration_restart_root_tests` copies the complete interrupted store into a different root and requires `IntentDiffers`. | Implemented; removing the root-file comparison incorrectly recovers the copy. |
| Immutable pools and current HEAD remain authoritative | `filesystem_migration_restart_pool_tests` damages segment/catalog/HEAD magic with exact decoder coordinates, substitutes a valid segment under the wrong digest name, and adds a valid orphan that changes the inventory bound by intent. | Implemented, including precise segment digest mismatch and `IntentDiffers`. |
| Unknown names refuse | Pool laws require exact pool, raw name and canonical-width refusal; namespace laws cover root and every nested migration directory. | Implemented; original root `Namespace` / `InvalidData` and nested `Adoption` / `NamespacePreflight` boundaries remain. |
| Wrong kinds and payload-bearing fences refuse | Namespace laws replace each fixed record/fence with directories and symbolic links, replace protocol directories with files, and require the exact nonempty fence kind/length refusal. | Implemented; existing diagnostic shapes are preserved. |
| Refusal preserves evidence | Every new scenario records root and descendants before reopening and after refusal: names, device/inode, file bytes, directory presence and link targets. Unknown witness shapes fail. | Implemented; deleting HEAD during observation fails the preservation assertion. |
| Success coverage and definition of done remain intact | Existing forward-prefix recovery, strict stage truncation, remount and process-death owners remain. The requirements ledger and normative recovery page link this matrix. | Full validation and exact-head hosted results belong to the PR receipt. Original roadmap checkboxes remain unchanged; merge is separately required. |

Root namespace admission currently exposes a typed `FilesystemMigrationAuthorityError::Namespace` boundary with `InvalidData`, while its deepest namespace diagnostic is a textual I/O payload. Nested preflight exposes the existing typed `NamespacePreflight` wrapper. These tests pin those existing public failure shapes; this change does not claim to add finer typed namespace coordinates or complete the separate #110 diagnostic audit.

## Calibration and observed results

The covered runtime matrix passes in debug and release against the unmodified product. Separately copied source/build mutations demonstrate meaningful RED outcomes: omitted root-file comparison admits a copied root; omitted exact-record inode comparison admits a substituted stage; deleting HEAD during observation fails the preservation witness; skipping recovery namespace preflight changes an unknown-entry refusal from `Adoption` to the later `Resumption` boundary; omitting inventory and derived store-identifier comparison incorrectly recovers changed inventory. None of these mutations is included in the branch.

Removing only the direct inventory comparison survives because the independently checked store identifier also binds that inventory. This survivor is retained as evidence of redundant protection, not reported as a weak test or hidden behind a mutation score. Removing both parts of that binding makes the orphan law fail with `ambiguous migration restarted successfully`.

These are calibrations of missing verification, not claimed production bug fixes with fabricated parent RED. The existing single corrupt-intent test was removed under the recorded deletion criterion “subsumed by stronger evidence”: the new record matrix preserves its exact checksum refusal, adds checksum coordinates, and replaces its entry-count-only witness with all names, identities and bytes. Remaining risk lives in the new record law and the complete witness, not in a count assertion.

## Execution and limits

Replay `cargo test --lib --all-features --locked filesystem_migration_restart` and its `--release` variant. Tests run in copied Docker sources with pinned Rust 1.96.0 on Linux aarch64. The laws are medium-size real-filesystem experiments using repository-only platform admission to isolate migration semantics. They execute a named forward prefix, drop writer authority, alter owned evidence, then reopen and derive current intent before recovery. This is deterministic restart semantics, not a process-death or physical power-loss campaign; existing crash campaigns retain those claims.

Schedules are the named forward phase and scenario tuple in each law; no random seed is involved. Each scenario owns fresh storage. Raw execution and calibration logs are retained in the author's issue-111 scratch evidence. The PR records immutable candidate coordinates and actual full validation results; compiler/setup failures are not behavioral RED evidence.

Before/after equality does not independently prove absence of a transient write restored before observation. The tests combine preserved-state witnesses with the exact refusal boundary; source inspection places observation/planning/adoption refusal before resumed mutation. They do not establish isolation against arbitrary concurrent out-of-band writers. Ordinary-test resource ceilings and exhaustive mutation adequacy are not claimed; the enforcement profile's existing gaps remain. Retire these laws only if the corresponding contract disappears or stronger restart evidence subsumes it.
