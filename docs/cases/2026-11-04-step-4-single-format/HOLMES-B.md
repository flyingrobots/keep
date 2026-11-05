# Testimony of Holmes, scope B: PR #220 at `fb355d66`

Watson, I read and did not build. Every "not compiled" below is literal. Worktree HEAD and the PR head both resolve to `fb355d66`, base `27d5c38c`.

The PR body is an unusually honest document. The trouble is in the repository docs, which state universals that the code and laws do not reach.

## 1. ADR-0010 conformance

| Requirement | Verdict | Evidence |
|---|---|---|
| Recoverable state machine, each prefix has one result | CONFIRMED | `src/adapters/store_initialization/initialization_execution.rs#83@fb355d66`, `docs/formats/store/initialization.md#94@fb355d66` |
| Non-prefix root refused, never adopted | CONFIRMED | `src/adapters/store_initialization/initialization_prefix_tests.rs#49@fb355d66`, `#63`, `#82` of the same file |
| One durable commit point, no admission before it | CONFIRMED | `initialization_execution.rs#130@fb355d66`, `src/adapters/filesystem_store_namespace.rs#197@fb355d66` (refuses `FORMAT.next`), `src/adapters/store_initialization/initialization_resume_tests.rs#117@fb355d66` |
| Concurrent attempts | CONFIRMED | `initialization_resume_tests.rs#152@fb355d66` |
| Missing retention never means nothing retained | CONFIRMED | `docs/formats/store/initialization.md#142@fb355d66`, `initialization_prefix_tests.rs#82@fb355d66` |
| Initialized store with no catalog is a defined state | CONFIRMED | `src/adapters/retention/filesystem_retention_snapshot.rs#333@fb355d66`, `tests/single_format/expectation_laws.rs#147@fb355d66` |
| 41-case crash matrix exists | CONFIRMED as written, UNVERIFIABLE as passing | `xtask/tests/durability_crash_case_contract.rs#102@fb355d66`, `docs/formats/store/initialization.md#189@fb355d66` |
| Step 1: gate on every publish route | CONFIRMED for in-tree routes, see §4 | |
| Step 2: writer and stage recovery; #121 not claimed | CONFIRMED | `docs/formats/store/catalog-publication.md#13@fb355d66`, `docs/formats/store/requirements.md#23@fb355d66` |
| Step 3: fixtures built through the writer | CONFIRMED, with qualification | `tests/support/populated_store.rs#165@fb355d66`. Hand-placed artifacts remain only in refusal laws, such as `tests/catalog_writer/finalization_laws.rs#60@fb355d66` |
| Step 4: integrated sequence | CONFIRMED, with two qualifications | Below |
| Step 4: older snapshot refuses newer expected state | CONFIRMED | `tests/single_format/expectation_laws.rs#114@fb355d66` |
| Step 4: torn copy law | CONFIRMED narrowly, see §3 | |
| Step 5: version 1 deleted, pages consolidated | CONFIRMED | `docs/formats/segment-store-v1` and `-v2` are absent at this tree, `docs/formats/store/recovery.md#43@fb355d66` |
| Definition digest changed, migration rows removed | CONFIRMED | `git diff --stat 27d5c38c fb355d66` shows `definition.tsv` -20 and `format-marker.hex` changed |
| #205, frozen decoders, upgrade tool | PLANNED, honestly framed | `docs/adr/0010-one-storage-format.md#89@fb355d66`, `#198`, `#245` |

**Step 4 sequence.** Each step maps in `tests/single_format/acceptance_laws.rs@fb355d66`:
- initialize `#93`
- ingest `#97`
- reopen `#101`
- exact bytes `#113`
- retain `#116`
- second catalog `#118`
- interrupt `#123`
- recover `#129`
- both reads `#130`
- lossy refusal with intact witness `#135` to `#150`

**Step 4 qualifications.**
- The interruption is an injected observer error before `HEAD` replacement, not process death. The law says so at `acceptance_laws.rs#3@fb355d66`. The hook is repository-task API, not production public API.
- The law exists only under `#[cfg(feature = "repository-tasks")]` (`tests/single_format/suite.rs#3@fb355d66`). CI passes `--all-features` (`.github/workflows/ci.yml#70@fb355d66`), but a plain `cargo test` silently omits it.

**Portability laws.**

| Law | Verdict | Evidence |
|---|---|---|
| Quiescent copy opens | CONFIRMED | `tests/single_format/portability_laws.rs#35@fb355d66` |
| Corrupt copy refuses as original | CONFIRMED for one marker byte only | `portability_laws.rs#50@fb355d66` |
| Incomplete copy refuses "exactly" | CONFIRMED weakly: compares `io::ErrorKind` only, for one removed directory | `portability_laws.rs#75@fb355d66` |
| Replaced lock or protocol directory refuses | CONFIRMED | `portability_laws.rs#88@fb355d66` |
| Independent divergence | CONFIRMED | `portability_laws.rs#128@fb355d66` |
| Backup round trip | CONFIRMED, with qualification | `acceptance_laws.rs#158@fb355d66`. "Verify" means reopen plus reconstruct (`tests/catalog_writer/writer_fixture.rs#43@fb355d66`). It never calls `load_for_verification` |

Conjecture: the `Busy` at `portability_laws.rs#101@fb355d66` comes from the root directory lock described at `docs/formats/store/initialization.md#151@fb355d66`, not lock-file identity. I did not read the lock code.

**No persisted identifier or binding.** CONFIRMED.
- The `FORMAT` byte table has no identifying field (`docs/formats/store/recovery.md#50@fb355d66`).
- `FilesystemDirectoryCoordinate` holds device and mount and is documented as never persisted (`src/adapters/filesystem_directory_coordinate.rs#5@fb355d66`).
- A grep found `RootIdentityChanged`, `require_root_identity` and `StoreIdentifier` nowhere in `src`, `tests`, `xtask`, `fuzz` or `conformance`.
- The remaining `.dev()` and `.ino()` reads are in-memory comparisons in the lock, stage, record and platform-check code (`src/adapters/filesystem_writer_lock.rs#41@fb355d66`, `src/adapters/retention/reader_fence.rs#74@fb355d66`). I saw no write path for them, though I did not audit each one line by line.

## 2. StoreExpectation

| Claim | Verdict | Evidence |
|---|---|---|
| `reopen` requires it | CONFIRMED | `src/adapters/filesystem_store_admission.rs#118@fb355d66` |
| `load` requires it | CONFIRMED | `src/adapters/retention/filesystem_retention_snapshot.rs#132@fb355d66` |
| `DurableSnapshot::open` requires it | CONFIRMED | `src/adapters/durable/snapshot.rs#96@fb355d66` |
| `DurableStore::open` requires it | CONFIRMED, but it refuses nothing itself | `src/adapters/durable/store.rs#83@fb355d66`. It returns `Ok` on a mismatch and refuses at first `snapshot()` (`store.rs#111@fb355d66`, `expectation_laws.rs#61@fb355d66`). Documented at `store.rs#71@fb355d66` |
| No default | CONFIRMED | The enum derives no `Default` (`src/adapters/store_expectation.rs#41@fb355d66`) |
| Mismatch carries expected and observed | CONFIRMED, always wrapped in the boundary error | `store_expectation.rs#70@fb355d66`, `src/adapters/filesystem_store_admission_error.rs#41@fb355d66`, `src/adapters/retention/filesystem_retention_snapshot_error.rs#42@fb355d66` |
| Writer compares under lock, reader under fence | CONFIRMED | `filesystem_store_admission.rs#125@fb355d66`, `filesystem_retention_snapshot.rs#175@fb355d66` |
| `NoExpectation` adds no I/O; `Catalog` adds one 128-byte `HEAD` read | CONFIRMED (plus a stat and a directory clone) | `filesystem_store_admission.rs#152@fb355d66`, `src/adapters/catalog_restart_loader.rs#42@fb355d66` |

**Routes that bypass it.**

| Route | Status |
|---|---|
| `FilesystemCatalogSnapshot::load` | **Unexempted, undisclosed: drift D1.** `src/adapters/filesystem_catalog_snapshot.rs#42@fb355d66`, publicly exported at `src/lib.rs#95@fb355d66` |
| `FilesystemRecoveryAdmission::reopen`, inventory reader, retention recovery | Exempt and disclosed (`docs/adr/0010-one-storage-format.md#157@fb355d66`). `src/adapters/filesystem_recovery_admission.rs#41@fb355d66` |
| `load_for_verification` (two definitions) | Exempt and disclosed. `src/adapters/retention/filesystem_retention_verification.rs#34@fb355d66`, `src/adapters/verification_ingress.rs#59@fb355d66` |
| `reopen_unchecked_for_repository_tasks` | Feature-gated, not mentioned in the exemptions. `src/adapters/filesystem_store_admission.rs#140@fb355d66` |
| `initialize` on a marker-linked populated root | Narrow, and not named as a bypass. It returns writer authority with no expectation choice. `filesystem_store_admission.rs#59@fb355d66`, `docs/formats/store/initialization.md#133@fb355d66`, and `observe_linked` at `src/adapters/store_initialization/initialization_observation.rs#199@fb355d66` |
| Publisher and retention authority | No bypass. They consume an admission (`src/adapters/retention/filesystem_retention_authority.rs#63@fb355d66`) |

## 3. The laws

**`expectation_laws.rs`.** Every named law does what its name says. That includes the pinned limit at `expectation_laws.rs#190@fb355d66` and the damaged-head boundary at `#171`. `DurableSnapshot::open` is reached through `store.snapshot()`, which is adequate. The header "every public open" (`#1`) is false, per D1.

**`torn_copy_laws.rs`.** Its assertions are sound, but they cover less than its documents say:
- Partial copies are half-length only, and only over the complete after-state (`tests/single_format/torn_copy_laws.rs#146@fb355d66`, `#154`).
- No mix contains an in-flight stage (`head.next`, `current.cat`, `current.seg`, retention stages), an absent unchanged file, or an empty new directory. `changed` is derived from final regular files (`#63`).
- Refusal is an untyped `Err(_refusal)` (`#106`), against the `AGENTS.md` rule to assert exact typed failures.
- The non-vacuity guard is weak (D3).

**KEEP-STORE-022 and -023.** Both rows exist and cite the right files (`docs/formats/store/catalog-protocol-requirements.md#35@fb355d66`, `#36`). Both overclaim, per D1 and D2.

## 4. #125 gate

**Forward route: CONFIRMED.** The order matches PR #200: admit candidate, admit evidence, stages, revalidate head, replace (`src/adapters/retained_catalog_publication.rs#73@fb355d66` through `#100`).

**Recovery route: CONFIRMED.**
- `verify` runs the closure gate (`src/adapters/filesystem_recovery_next_head_candidate.rs#41@fb355d66`).
- `replace_head` re-enters it immediately before the rename (`src/adapters/filesystem_recovery_next_head_finalization_storage.rs#53@fb355d66`).
- A law pins the refusal (`tests/catalog_writer/finalization_laws.rs#85@fb355d66`).

**Other routes.** Stage discard, completion and resume do not change `HEAD` (`docs/formats/store/catalog-publication.md#127@fb355d66`). `FilesystemCatalogStages` is not exported.

## 5. Retired identifiers

| Claim | Verdict | Evidence |
|---|---|---|
| Every retired ID is in the ledger | CONFIRMED | Every ID live on base under `docs/formats`, `conformance`, `xtask`, `src`, `tests` is either still live at head or in the ledger. `comm` residual is empty |
| 21 migration rows, 5 v1 rows, -078, 8 MIGRATION, STORE-020, RECOVERY-002/003, RETENTION-014 | CONFIRMED | `docs/formats/store/retired-identifiers.md#26@fb355d66`, `#54`, `#129`, `#152`, `#164` |
| Not reused in code | CONFIRMED | Outside the ledger they appear only in history and in "retired" assertions (`xtask/tests/segment_store_protocol_contract/transition_laws.rs#56@fb355d66`) |
| Cited last-evidence commits are real | CONFIRMED except one path (D6) | 18 of 19 hashes are on base. `e3930f0` is not, and the row admits that |
| -079/-080 honestly described | PLANNED, CONFIRMED honest | PR body, `docs/formats/store/catalog-publication.md#174@fb355d66`, `docs/formats/store/requirements.md#25@fb355d66`, `retired-identifiers.md#167@fb355d66` |
| Code implements -079/-080 | No. CONFIRMED absent | Crash IDs named in `xtask`, `src`, `tests` skip 078 to 080. The PR says this and says the agreed plan is unsatisfied |

## 6. Remaining PR-body claims

| Claim | Verdict |
|---|---|
| 937 passed, crash matrix exit 0, Clippy, 417/9 xtask results, RED calibration | UNVERIFIABLE |
| "Owner rejected", "owner confirmed", "agreed plan" | UNVERIFIABLE from the code. ADR records the no-stores statement (`docs/adr/0010-one-storage-format.md#40@fb355d66`) |
| Long-lived `DurableStore` refuses after new publication | CONFIRMED (`expectation_laws.rs#124@fb355d66`) |
| Retention-only rollback undetected, pinned by law | CONFIRMED (`expectation_laws.rs#190@fb355d66`) |
| Recovery can finalize against a rolled-back store | CONFIRMED by signature (`src/adapters/filesystem_recovery_admission.rs#41@fb355d66`), not exercised |
| Invariant "Opening a store states what the caller expects" | DRIFT (D1) |

## 7. Drift ledger

- **D1, worst.**
  - Claim: "Every open therefore requires this choice" (`src/adapters/store_expectation.rs#39@fb355d66`).
  - Also: `docs/formats/store/requirements.md#107@fb355d66`, KEEP-STORE-022, the law's own header, and the PR invariant.
  - The ADR exempts only recovery and verification (`docs/adr/0010-one-storage-format.md#157@fb355d66`).
  - Evidence: `FilesystemCatalogSnapshot::load(root, policy)` admits a store and returns a pinned view with no expectation (`src/adapters/filesystem_catalog_snapshot.rs#30@fb355d66`, `#42`). The acceptance law uses it (`tests/single_format/acceptance_laws.rs#76@fb355d66`).
  - A caller can therefore lose the check by omission, the exact outcome the type says it prevents.
- **D2.**
  - Claim: "Every ... partly copied file" (`docs/formats/store/catalog-protocol-requirements.md#36@fb355d66`, `docs/formats/store/recovery.md#118@fb355d66`, and the test name at `torn_copy_laws.rs#143@fb355d66`).
  - Evidence: one half-length truncation per file, over one state (`torn_copy_laws.rs#154@fb355d66`).
  - The PR body discloses this; the normative docs do not.
- **D3.**
  - Claim: counts mean "the laws cannot pass vacuously" (`torn_copy_laws.rs#27@fb355d66`).
  - Deduction: mask 0 is the untorn before-state. It reads the first blob and refuses the second, which alone satisfies both guards at `#137` and `#138`. No torn mix need do anything.
- **D4.**
  - The ledger says -078 to -080 "are reserved" (`docs/formats/store/retired-identifiers.md#83@fb355d66`).
  - Then it retires -078 (`#164`, and `docs/formats/store/catalog-publication.md#173@fb355d66`).
- **D5.**
  - The ledger lists as deleted `classify_recovery_names` (`retired-identifiers.md#109@fb355d66`) and `filesystem_catalog_publisher.rs`, `filesystem_catalog_publisher_storage.rs`, `store_initialization*.rs` (`#173`).
  - All are present at head, for example `src/adapters/recovery/recovery_name_classification.rs#26@fb355d66`.
  - They are renamed version-2 successors (`CHANGELOG.md#30@fb355d66`). The ledger never says so.
- **D6.**
  - Claim: `tests/catalog_writer.rs` at `e3930f0` (`retired-identifiers.md#132@fb355d66`).
  - Evidence: that path is absent at `e3930f0`. The law is `tests/version_two_catalog_writer/version_laws.rs#79@e3930f0`.
  - Conjecture: "Never on main" goes stale in a loose sense, since a real merge commit will bring `e3930f0` into main's ancestry.
- **D7.**
  - Claim: "no `KEEP-CRASH-074` or later point exists there", with 074 to 087 assigned to GC (`ROADMAP.md#1416@fb355d66`, `#1422`, `#1249`).
  - Evidence: head assigns 074 to 077 and 081 to 091 elsewhere (`docs/formats/store/catalog-publication.md#169@fb355d66`, `docs/formats/store/initialization.md#177@fb355d66`).
  - The #107 GC salvage must now renumber, and nothing says so.
  - Also, `ROADMAP.md#2409@fb355d66` still gives a retired ID a future acceptance criterion: "`KEEP-RECOVERY-003` gains a macOS row".
- **D8.**
  - Minor. The ADR says the format has no `FORMAT.next` (`docs/adr/0010-one-storage-format.md#76@fb355d66`).
  - Initialization depends on it as its pre-commit stage (`docs/formats/store/initialization.md#70@fb355d66`).
  - True only of committed stores, a carve-out the ADR does not state.
- **D9, conjecture, deduced from signatures, not compiled.**
  - Claim: the ungated entry "can therefore never drive gated stages" (`src/adapters/filesystem_retained_catalog_stages.rs#8@fb355d66`).
  - Evidence: `RetainedCatalogPublicationStorage` is public and unsealed (`src/adapters/retained_catalog_publication/retained_catalog_publication_storage.rs#23@fb355d66`), and an external test implements it (`tests/retained_catalog_publication/model_storage.rs#86@fb355d66`).
  - The publisher's impl ignores the admission token (`src/adapters/filesystem_catalog_publisher_storage.rs#26@fb355d66`).
  - `read_retention_head -> None` yields an unretained admission (`src/adapters/retained_catalog_publication/retained_evidence_admission.rs#26@fb355d66`).
  - So a caller-written wrapper over the publisher could report no retention head, or hand the stages to `publish_catalog_generation` (`src/adapters/catalog_publication.rs#25@fb355d66`).
  - It requires deliberate misuse. No in-tree route does this.

## 8. Reverse drift

- `FilesystemCatalogSnapshot::load` is absent from the PR's failure modes and exemptions.
- `publish_catalog_generation` survives version 1's deletion as a public export (`src/adapters/exports.rs#16@fb355d66`). The only implementors I found are a test double (`tests/catalog_publication/recording_storage.rs#87@fb355d66`) and the gated stages. PR #200's "unchanged for version 1" no longer has a referent.
- The integrated acceptance sequence is feature-gated, which the PR does not mention.
- `initialize` can adopt a populated marker-linked store with no expectation. It is documented in the format page but not as an expectation bypass.

## VERDICT

| Area | Result |
|---|---|
| Direct initialization, commit point | CONFIRMED |
| Steps 1 to 5 | CONFIRMED with qualifications; #205 and later PLANNED |
| Step 4 integrated sequence | CONFIRMED (injected interruption, feature-gated) |
| Portability laws | CONFIRMED, thin on damage variety |
| No store identifier or binding | CONFIRMED |
| Four named opens require `StoreExpectation` | CONFIRMED |
| "Every open" requires it | **DRIFT (D1)** |
| Mismatch expected and observed | CONFIRMED |
| Torn-copy law | CONFIRMED narrowly; **DRIFT (D2, D3)** |
| Retired ID accounting | CONFIRMED; **DRIFT in wording and one path (D4, D5, D6)** |
| Roadmap and ADR ID and format text | **DRIFT (D7, D8)** |
| KEEP-CRASH-079/080 | PLANNED, honest, unimplemented |
| #125 gate, in-tree routes | CONFIRMED; **conjectural bypass by wrapper (D9)** |
| Test counts, RED runs, CI results | UNVERIFIABLE |

**Drift count:** 8 established, 1 conjectural.

**Worst drift:** D1. `FilesystemCatalogSnapshot::load` opens a store with no expectation, while the type docs, `docs/formats/store/requirements.md`, KEEP-STORE-022 and the law header all claim every open requires one.

**Fidelity grade: B.** The PR body is close to scrupulous, and it volunteers its own unmet plan on -079/-080. The grade falls because the normative docs state universals that the code and laws do not reach, and because one public open escapes the guarantee this PR exists to establish.
