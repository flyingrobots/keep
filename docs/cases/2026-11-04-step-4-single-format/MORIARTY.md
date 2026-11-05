# Testimony of MORIARTY: PR #220 at `fb355d66`

Such a lovely large gift: 461 files, 13,564 lines in, 19,947 lines out. I came to make the store lie, and I regret to report it mostly refused to. The store's defenses are good. Its evidence for them is much softer, and that is where I feasted.

**Method and limits:** this was read-only. I ran no cargo, and I changed nothing in the repository. Three helper lists went to my scratchpad. I read the admission, initialization, expectation and platform code in full, and compared the merge by test-name diffs across both parents plus the key source diffs. I did not read all 461 files. Anything marked CONJECTURE has not been executed.

## Strikes landed

### S1. MAJOR (vector 2): a public read route with no expectation at all

`FilesystemCatalogSnapshot::load(root, policy)` is public, exported and not feature-gated. It takes no `StoreExpectation`, no fence and no platform admission:
- `src/adapters/filesystem_catalog_snapshot.rs#42@fb355d66`
- `src/lib.rs#95@fb355d66`
- `src/adapters/catalog_restart_loader.rs#24@fb355d66`, which opens ambiently at line 28

It serves content through `src/adapters/catalog_snapshot.rs#48@fb355d66` and `src/adapters/admitted_segment_record.rs#56@fb355d66`.

The repository contradicts itself on this route:
- `src/adapters/store_expectation.rs#39@fb355d66` says "Every open therefore requires this choice."
- `tests/single_format/expectation_laws.rs#1@fb355d66` says the laws hold "at every public open."
- `docs/adr/0010-one-storage-format.md#152@fb355d66` exempts only recovery and verification (line 157).
- `docs/formats/store/publication.md#156@fb355d66` calls this route "the observational reader boundary," which is neither.
- `docs/formats/store/requirements.md#47@fb355d66` lists catalog restart among "every open path."

The PR's failure modes name recovery and verification and stay silent here. A rolled-back or wrong store answers this route without refusing. Bytes stay authenticated, so the Core Law survives, but the "no caller loses the check by omission" promise does not.

### S2. MAJOR (vectors 2 and 6): one crash forces the downgrade

`src/adapters/store_expectation.rs#10@fb355d66` says a receipt or pinned view supplies the expected catalog. Process death after `HEAD` replacement leaves generation two published and no receipt (`docs/formats/store/catalog-publication.md#172@fb355d66`). Gated finalization also moves `HEAD`.

The old `Catalog(E)` then refuses forever. The only other variant is `NoExpectation`, which accepts "a different store" (`src/adapters/store_expectation.rs#49@fb355d66`). A successor of this store and an unrelated store at that generation refuse with the same mismatch shape.

No law or document covers this:
- `StoreExpectation::Catalog` appears in tests only at `tests/single_format/expectation_laws.rs#53@fb355d66`, plus lines 123, 179 and 207. That file never interrupts a publication.
- Every crash-matrix open uses `NoExpectation`: `xtask/src/durability_crash_matrix/restart/initialization.rs#282@fb355d66`, `xtask/src/durability_crash_matrix/restart/retained_catalog.rs#212@fb355d66`, `xtask/src/durability_crash_matrix/restart/retention_snapshot.rs#29@fb355d66`.
- The only format page describing the expectation is silent on this (`docs/formats/store/recovery.md#127@fb355d66`, through line 139).

The PR says the check "adds no write protocol, so it needs no crash rows." That is true of the protocol and useless to the caller. It does fail closed, which is why this is not FATAL. A caller could record both catalogs before publishing, but nothing documents that, and `ExpectedCatalog` holds only one.

CONJECTURE, on severity: this strands real callers. Experiment: kill at `KEEP-CRASH-077` after replacement, holding the prior catalog. Then show that no public API separates "my successor" from "another store at that generation" without the caller's own write-ahead record.

### S3. MAJOR (vector 5): the commit point's synchronization is unobservable

`a_failed_commit_synchronization_leaves_an_unadmitted_resumable_store` fails no synchronization. Its observer errors at `Boundary::Before`, so the sync never runs (`src/adapters/store_initialization/initialization_resume_tests.rs#117@fb355d66`, lines 119 to 125).

`Execution::phase` notifies around whatever closure it is given (`src/adapters/store_initialization/initialization_execution.rs#58@fb355d66`). A phase label is not a syscall.

`docs/formats/store/requirements.md#45@fb355d66` marks "returns authority only after the commit and cleanup synchronizations" as Implemented. Its first cited law checks inventory and bytes only (`src/adapters/store_initialization/initialization_resume_tests.rs#39@fb355d66`). In the files I read, `RecordingStorage` was a port-level witness on main (`tests/store_initialization.rs#83@27d5c38c`), and it was deleted. The document honestly disclaims power loss (`docs/formats/store/initialization.md#189@fb355d66`). The disclaimer does not excuse the test's name.

CONJECTURE: stub `entries::synchronize_root` to `Ok(())` at `src/adapters/store_initialization/initialization_execution.rs#131@fb355d66`, then run everything. I predict all green. A label-only witness can't distinguish the two, and AGENTS.md demands that sync behavior be "crash-tested."

### S4. MAJOR, disclosed (vector 6): deletion lands with recovery rows unkilled

`KEEP-CRASH-079` and `-080` are Planned (`docs/formats/store/catalog-publication.md#174@fb355d66`, line 175). `docs/formats/store/requirements.md#25@fb355d66` says so honestly.

Finalization and completion run only after a writer dies (`xtask/src/durability_crash_matrix/restart/retained_catalog.rs#89@fb355d66`, line 100). Nothing kills them while they run, so these remain unproven:
- death during recovery's stage-completion link
- death during gated `HEAD` replacement
- the "gate reruns on retry" claim

Per AGENTS.md, this is a write transition that must be crash-tested.

The PR admits "the agreed plan wanted them before version 1 is deleted," then deletes anyway. An issue search for the identifier surfaces only #198; I found no dedicated tracker.

In fairness, I found no killed-finalization row on main either, so nothing was lost. The owed evidence was simply not delivered. The confession is honest, but merging it unaccepted makes it an unexecuted IOU.

### S5. MAJOR (vectors 5 and 7): the torn-copy laws claim more than they test

- **The anti-vacuity guards certify nothing.** `tests/single_format/torn_copy_laws.rs#27@fb355d66` promises the laws "cannot pass vacuously." Mask 0 alone satisfies both guards at lines 137 and 138: it is the untorn before-state, where the first blob reads and the second refuses. Every genuinely torn mix could refuse for any reason and still pass.
- **Any failure counts as a lawful refusal.** `Err(_refusal)` accepts every failure, operational ones included (line 106). AGENTS.md requires exact typed failures.
- **In-flight state is excluded by construction.** The law's own header describes a copy taken "while a writer publishes" (line 3), yet both states are quiescent (lines 52 to 62). `head.next`, `current.cat` and `current.seg` never appear. Readers admit a `head.next` (`src/adapters/retention/filesystem_retention_snapshot.rs#122@fb355d66`), and recovery takes it with no expectation.
- **The wording is inflated.** The law is named "every_partly_copied_file" (line 143), and `docs/formats/store/catalog-protocol-requirements.md#36@fb355d66` marks that wording Implemented. It tests one half-length tear per file, always over the after-state (lines 151 to 159). The PR body admits half-length; the ledger and the law's name do not. The PR's "covers the outcomes deterministically" goes further still.

CONJECTURE, that an in-flight mix is exploitable: I have no bypass. Experiment: capture the tree at each `CatalogPublicationPhase` boundary, overlay those trees into the mixes, then run the reader and the recovery tools on each.

### S6. MINOR (vector 7): "CI is the check" cannot check what it claims

The PR reports two `process_group` failures on macOS, "not diagnosed," under "CI is the check for them because the Linux job runs them."

- Linux cannot clear a macOS-only failure.
- The macOS job runs `cargo test` without `--workspace` (`.github/workflows/ci.yml#123@fb355d66`).
- `docs/testing-evidence/filesystem-platform-adapters.md#30@fb355d66` confirms that xtask tests never run on macOS.
- The crash matrix terminates children through this same `ProcessGroup` (`xtask/src/durability_crash_matrix/process.rs#11@fb355d66`, line 167).

CONJECTURE, on consequence: the failures may be sandbox artifacts. Experiment: name the two laws and run them natively outside the sandbox.

### S7. MINOR (vector 7): stale claims in present tense

- `README.md#56@fb355d66` claims 156 coordinates for `KEEP-CRASH-001`–`052`. Five of those are retired, so 47 × 3 = 141. The README also still says "Durable version-1 segment store" (line 48).
- `docs/formats/store/publication.md#29@fb355d66` says the store requires case-sensitive names and that a probe refuses case folding. Default APFS is admitted, and `src/adapters/darwin_filesystem.rs#37@fb355d66` has no such probe.
- `docs/testing-evidence/filesystem-platform-adapters.md#22@fb355d66` says 963 passed. The PR says 937 "at this head." The section was edited in this stack and the table was left behind.
- `docs/testing-evidence/filesystem-platform-adapters.md#71@fb355d66` lists `a_writer_lock_cannot_mint_...` as a law that "remains." It was deleted in `579e64fe`.

### S8. MINOR (vector 5): half of `KEEP-PORTABILITY-003` cannot fail on its stated cause

`tests/single_format/portability_laws.rs#92@fb355d66` replaces `writer.lock` (lines 93 and 94) and expects `Busy`. The held root flock returns `Busy` before `writer.lock` is ever consulted (`src/adapters/filesystem_writer_lock.rs#82@fb355d66`).

CONJECTURE: delete lines 93 and 94, and the law still passes. Lock-identity coverage exists separately, at acquisition.

### S9. MINOR (vector 5): weak witnesses

- `tests/store_admission_contract.rs#24@fb355d66` greps source text, so it would pass with `require_expectation`'s body removed. Its own comment concedes the markers only keep writers on the admission side.
- `tests/darwin_store_spelling.rs#53@fb355d66` uses bare `is_err()`, against AGENTS.md.
- `docs/testing-evidence/filesystem-platform-adapters.md#49@fb355d66` says those laws do not witness the lookup check, yet line 57 lists them as covering the public routes. In the Darwin tests I read, per-lookup evidence stops at the unit laws for `FORMAT`, `HEAD` and one directory, plus the existing aliases. I saw no public-route law for pool-entry names, such as uppercase hex. The wrapper still guards them.

### S10. MINOR (vector 3): `AlreadyInitialized` is inferred from existence

`src/adapters/store_initialization/initialization_observation.rs#32@fb355d66` returns it for any `FORMAT` beside no `FORMAT.next`, with no validation. A root holding only a junk `FORMAT` is told it is initialized. `docs/adr/0010-one-storage-format.md#107@fb355d66` says existence proves nothing. It refuses without mutation, so the harm is a false diagnostic only.

### S11. MINOR, CONJECTURE (vector 3): a power-loss stage may never resume

`src/adapters/store_initialization.rs#4@fb355d66` promises that every interrupted state is a prefix a later call resumes. A marker stage whose bytes are garbage after power loss refuses as `MarkerStageMismatch` (`src/adapters/store_initialization/initialization_observation.rs#191@fb355d66`, the check, with refusal at 194) and never resumes. It fails closed, but the promise is broader than the behavior. Experiment: block-level crash replay of `WriteMarkerStage` on APFS and ext4. The documents disclaim power loss, so this is minor.

## Attacks attempted and failed (certification)

**Vector 1: surrender, no strike.**
- Non-test source touches raw `cap_std::fs::Dir` only in the wrapper, the spelling check and the platform adapters, in the files my search covered.
- `.raw()` in production appears only in platform admission.
- The two `openat` families use `"."` or the root path, never a protocol name.
- `entries()` returns exact stored names.
- `require_exact` refuses multi-component paths.
- Both `hard_link` and `rename` check source and target.
- `FilesystemCatalogSnapshot::load` skips the platform profile but keeps the wrapper.
- The only residue is a check-then-use race between two syscalls, in a design that disclaims out-of-band mutation. I could not make a foreign spelling answer.

**Vector 2: these partial attacks failed.**
- The reader compares the collected view it serves, after collection, so there is no time-of-check gap (`src/adapters/retention/filesystem_retention_snapshot.rs#174@fb355d66`).
- The writer compares under the lock.
- A retained `head.next` refuses the writer at namespace admission, so recovery cannot move `HEAD` beneath an admitted writer.
- The rollback, different-store and no-catalog laws are real.
- The retention-only limit is disclosed and pinned.
- Non-test downgrade to `NoExpectation` occurs only in verification, which the ADR exempts.
- The rollback bypass through recovery is disclosed in the PR.

**Vector 3: the commit point holds.**
- `FORMAT.next` is durable first, the link never replaces, the root sync follows, and only then is the stage removed.
- Every admission refuses `FORMAT.next`.
- A reader racing the two lookups in `admit_committed` still hits the membership check.
- Foreign, out-of-order and stray-child roots all refuse.
- Concurrent attempts get `Busy` or `AlreadyInitialized`.
- An empty lone `writer.lock` is adopted, but that is documented and harms nothing.

**Vector 4: surrender, no merge damage proven.**
- The three-way test-name diff shows nothing dropped from both parents.
- The one stack-only loss, `every_protocol_child_must_share_...`, is covered by `src/adapters/filesystem_mount_admission_tests.rs#74@fb355d66` and `src/adapters/filesystem_platform_profile_tests.rs#19@fb355d66`.
- The O_PATH law survives as `src/adapters/filesystem_catalog_artifact.rs#156@fb355d66`.
- #216's spelling laws, FIFO law and reattach law survive and are extended.
- The nested-lookup fix in the merge is correct.
- Limit: a rename-blind diff plus selected source diffs is not a line-by-line audit of 76 resolved files.

**Vector 5: these held.**
- The fold-case tests' other branch is guarded by `KEEP_TEST_EXPECT_APFS_VARIANT` (`src/adapters/darwin_filesystem_tests.rs#31@fb355d66`).
- CI runs both APFS variants with `--all-features`.
- Nothing in the new laws is gated off macOS except the Linux tmpfs mechanisms.

**Vector 6: the ledger is clean.**
- No identifier from main vanished without a ledger row.
- No retired ID is live in `xtask/src/durability_crash_point_identity.rs#8@fb355d66`.
- Eighteen of nineteen cited hashes are on main, and the other (`e3930f0`) is stated as in-PR.

**Vector 7: these claims held.**
- Nothing persists a device, inode, mount or path in the code I read.
- All required CI checks show success at this head. CodeRabbit was skipped, so no bot review counts here.

## Verdict

```text
VERDICT
Target:          PR #220, fb355d66 onto 27d5c38c
FATAL:           0  (no route returned wrong bytes; no incomplete store admitted)
MAJOR:           5  S1 catalog-load bypass of the expectation
                    S2 crash forces NoExpectation, no law or doc
                    S3 commit-point sync unobservable, test misnamed
                    S4 KEEP-CRASH-079/080 unimplemented at deletion (disclosed)
                    S5 torn-copy guards vacuous, in-flight state excluded
MINOR:           6  S6 macOS xtask failures "covered" by Linux CI
                    S7 stale README, spec and evidence claims
                    S8 replaced-lock half of the law cannot fail on its cause
                    S9 string-grep and is_err() witnesses
                    S10 AlreadyInitialized inferred from existence
                    S11 power-loss stage does not resume (CONJECTURE)
Failed attacks:  vectors 1 and 4 surrendered; commit point, reader/writer
                 compare timing, and retired-ID ledger certified
Resilience:      C+   Runtime refusal behavior is strong; the proof and the
                 prose claim more than the tests establish.
Before merge:    Decide S1 and S2, or narrow "every open" to what is true.
                 Run the S3 stub experiment. Fix the S5 guards and add
                 in-flight mixes. Have the owner accept S4 explicitly, with
                 a tracking issue. Correct S7.
```

Run the S3 stub experiment first. If I am wrong, you lose one test run. If I am right, your one durable commit point has been certified by a label.
