# Retention stage prefix refusal evidence

This receipt owns the focused fixed-field recovery correction in PR #99, production fix `ffb5074`, and the policy-only integration of reviewed PR #152 in `813df95`. The change kind is a bug fix with an additional public error diagnostic; it does not certify the entire retention PR.

## Runtime claim and oracle

When an interrupted root, manifest, or head stage contains an available fixed byte that contradicts the version-two grammar, recovery must return the stage-specific `StageCorrupt` refusal and preserve every retained file's bytes. Canonical strict prefixes must remain classified as interrupted writes. The oracle is the format grammar and checked-in canonical conformance records, independently of the case generator.

The public assessment laws are in [retention_stage_prefix.rs](../../tests/retention_stage_prefix.rs). The filesystem regression is `noncanonical_short_stages_refuse_recovery_without_changing_retained_bytes` in [filesystem_retention_recovery_prefix_tests.rs](../../src/adapters/retention/filesystem_retention_recovery_prefix_tests.rs). These assert production classification, typed diagnostics, and persistent bytes. They do not assert test-harness case totals.

## Observed RED

On unfixed production `ee0e585864d8b17ee9b2b2e52950c5f6f5fad4a5` with the filesystem regression copied into the isolated Docker checkout, `cargo test -p keep --lib noncanonical_short_stages_refuse_recovery_without_changing_retained_bytes -- --nocapture` exited 101 at the intended assertion: malformed `root.next` returned `Ok` with `DiscardRootStage` and `Clean`. Test commit `8b3557d` records that reproduction.

On exact unfixed commit `4513a776d1c901c659d20a75bcf436c96eedf43a`, `cargo test -p keep --test retention_stage_prefix` exited 101. Each corrupt-prefix law failed at fixed byte zero and prefix length one; canonical-prefix laws passed. This was repeated in a dedicated build directory, independent of the fixed checkout's artifacts.

## Calibration

Changing the production prefix comparator to report the wrong expected byte made each `short_*_refusal_reports_only_observed_bytes` assertion fail. Changing production admission to reject canonical prefixes made each `canonical_*_prefixes_remain_recoverable_interrupted_writes` assertion fail at its named check.

Deleting the retained root stage after observing it but before planning left the correct typed corruption refusal intact and made the exact-byte preservation assertion fail: `refusal must preserve every retained byte for root.next`. This demonstrates the preservation assertion independently of the refusal assertion.

One early calibration selected zero matching tests after independent checkouts shared a build directory. That run is invalid and is excluded from this evidence. Parent, fixed, and mutated checkouts were re-executed with separate Cargo target directories; mutations affected only disposable Docker copies.

## Observed GREEN

All execution used copy-based Docker isolation on Linux with pinned Rust 1.96.0. `cargo test -p keep` passed, including the library, public integrations and doctests. `cargo test -p keep --release --test retention_stage_prefix` and `cargo test -p keep --release --lib retention` passed. Full release workspace validation is not claimed.

Both workspace Clippy configurations passed with all targets and `-D warnings`, with and without all features. Rust formatting, fuzz formatting, the source structure check, whitespace validation, and Docker Markdown lint passed. The duplicate cross-sequence match arms encountered during integration were corrected separately in `5b9fc4d` without changing their typed refusal.

The old successful interrupted-write fixture used literal garbage. It now uses a strict prefix of the actual canonical manifest and retains its existing publication/head-byte assertions. The new refusal regression separately protects malformed short bytes; this preserves the distinction between interrupted canonical writes and corruption.

## Bounded fuzz exploration

The existing retention format target now calls all three stage assessments as well as complete-record decoders. The focused Docker command was `cargo +nightly-2026-07-24 fuzz run retention_format --debug-assertions --sanitizer address -- -seed=99 -max_total_time=15 -timeout=5 -max_len=1048576 -rss_limit_mb=1024 -print_final_stats=1`, using cargo-fuzz 0.13.2 and the prepared canonical corpus. Seed 99 was fixed in the invocation outside the fuzz process; the resource arguments match [campaign.env](../../fuzz/campaign.env). The command completed successfully without a reported finding.

An earlier exploratory run used different input/RSS limits and is not the policy smoke receipt. Fuzz completion is robustness evidence for the exercised target, not proof that every semantic corruption is refused or every trust-boundary parser is covered.

## Remaining blind spots

The focused fix checks fixed fields. Incomplete semantic and variable fields, sync-before-publication recovery ordering, current-generation and namespace admission, error boundaries, and stronger persistent-view oracles remain review obligations. The runtime fixtures bypass production platform admission deliberately to isolate post-admission storage behavior; they do not establish platform eligibility or physical power-loss durability. PR #99 remains gated on its complete review queue and current-head validation.

## Complete generation field correction

Test commit `9431fc0` demonstrated that the published unfixed production implementation at `ce54ae5` classified complete zero-generation fields as discardable truncation. The public root and manifest laws failed at prefix length 40, and the head law failed at prefix length 32; the filesystem law returned `DiscardRootStage` instead of the required typed corruption refusal. Production fix `b365f6b` admits complete fields through the existing checked generation constructors, without fabricating missing bytes or changing complete-record decoding precedence.

The public generation and fixed-prefix laws, filesystem preservation law, release retention library suite, both workspace Clippy configurations, formatting, source structure check, and the same bounded seeded ASan retention campaign passed. A disposable production mutation deleting `root.next` before planning made the filesystem preservation assertion fail by name even though the typed refusal remained correct. This is focused recovery evidence, not a complete workspace or power-loss claim.

## Tool verification and test curation

Commit `69def44` replaces the crash CLI test's implementation-derived round trip and obsolete refusal of `retention` with independent documented accepted and rejected spellings. These assertions verify repository-tool runtime behavior, not Keep durability. Changing the production retention spelling to `retentions` made both the admission and refusal laws fail at their intended assertions. The first calibration attempt failed during compilation because a source-copy exclusion accidentally removed a legitimate directory named `target`; it is invalid evidence. The corrected complete source copy produced the two assertion failures in a separate build directory.

The deletion criterion for `requirement_ledger_names_planned_and_executable_evidence` is an uninterpretable class-5 change detector under Testing Standards Rule 18: substring presence neither establishes the listed claims nor validates their evidence, while the frozen phrase `Planned in #19` rejects legitimate progress to implementation. The controlled full workspace run exposed that obsolete phrase assertion; changing it to a new status would preserve the same defective oracle. Removing it does not waive any ledger requirement. Runtime contract laws and requirement-by-requirement acceptance review retain responsibility for those claims; Markdown and link validation retain responsibility for document integrity. Neighboring documentation guards are not promoted to runtime evidence by this deletion.

After that focused deletion, `cargo test --workspace --all-features` and Rust formatting passed in the controlled Docker checkout. The external BLAKE3 oracle was supplied through `/tools/bin` in the test environment. Earlier runs with a transport-generated AppleDouble file or a missing oracle executable were setup failures, not product regressions or retry-to-green evidence. Full release workspace validation and an independent current-head review remain outstanding.

## Recovery file synchronization

The OS-boundary regressions in test commit `97b7b96` were observed red on unfixed production `fb34603`: all three complete stages were published by the surviving restart process without a preceding successful `fsync` on the exact staged file. Linux strace 6.13 observed production syscalls while the existing crash command killed the writer immediately after writing each stage. The oracle distinguishes the killed writer from the surviving restart process, requires an actual publication witness, and checks the stage pathname rather than descriptor numbers or syscall counts. This tests a contractual interaction with the operating system, not internal choreography or physical power-loss survival. The Linux test environment requires strace and permission to trace its own children; CI installs and reports the system tool version.

Adding stage synchronization before root namespace creation/link, manifest link, and head replacement made all three laws pass in debug and release. The retention library suite and all-feature workspace Clippy also passed. Rust formatting initially flagged the new test layout and was corrected; this is not a runtime failure.

Hosted validation of `fb34603` exposed a stale-document-copy gap in the earlier local full-workspace claim: the Docker checkout did not contain the latest recovery document. The hosted documentation-size guard refused its 306 lines. The earlier pass therefore proves the executed Rust suite against that local document snapshot, not full current-tree validation. Current-tree verification must use a complete source copy and resolve the document ownership issue before claiming the broad gate is green.

The retention restart section now lives in [retention-recovery.md](../formats/segment-store-v2/retention-recovery.md), linked from the namespace page and protocol index; its grammar and boundaries are preserved. The existing documentation guards read the new owner explicitly and remain documentation checks. Obsolete inline line-length overrides were removed from the touched pages so the accepted global line-length policy governs prose. A complete copy of the changed source and documents passed both `cargo test --workspace --all-features` and its release counterpart, including the OS-boundary regressions. All-feature Clippy, source structure, formatting, and Markdown passed. A preliminary complete-copy run failed Git's ownership check after archive extraction changed the owned sandbox directory's UID; correcting that sandbox ownership, without global Git exceptions or source changes, allowed the controlled run. Physical power loss, kernel I/O error injection, and the remaining PR review queue are still not certified by these results.

## Namespace refusal before recovery effects

Test commit `282c105` records RED on unfixed production `a038b1c`. Direct recovery returned `Clean` after discarding `root.next` despite an unknown retention entry; publication returned the expected typed unknown-entry refusal after the stage was deleted, failing the exact-byte witness. An initial parallel run reused a fixture name and encountered `AlreadyExists`; that setup failure is excluded. Each entry point and case now owns a distinct sandbox name.

Recovery now shares forward namespace admission while additionally permitting the three fixed stage names. The census checks entry names and pool kinds before stage observation or execution; bounded stage observation still checks the admitted names' contents. The real-filesystem laws cover each fixed stage prefix with an unknown retention entry, a non-namespace root entry, or a malformed manifest pool name, and require the exact typed source plus unchanged retained bytes. A disposable production mutation changing the unknown-entry refusal to a different namespace refusal made both diagnostic assertions fail by name. A separate mutation changing the preserved I/O kind to `InvalidInput` made the kind assertions fail. Deleting the root stage before admission while retaining the correct typed refusal made both exact-byte preservation assertions fail by name. These are filesystem outcomes, not census-count assertions.

The corrected code passed focused retention laws in debug and release, the existing syscall synchronization regressions, full all-feature debug and release workspace suites, both Clippy configurations, source structure, formatting, and Markdown. Initial compilation caught a discarded must-use census, and Clippy caught test-helper style violations; those were corrected and are not runtime RED evidence. The [namespace admission decision](../adr/retention-recovery-namespace-admission.md) records ordering, alternatives, and remaining obligations. This correction establishes names-and-kinds refusal before effects; complete pool-content admission, capacity limits across all recovery transitions, and independent final-head approval remain open.

## Pool stage identity before recovery publication

Test commit `7d45af2` records real-filesystem RED on unfixed production `aa71bc0`. A byte-equal root pool replacement let recovery execute `FinalizeHead` before refusing at `RemoveRootStage`. A manifest replacement let it finalize the head and remove the root stage before refusing at `RemoveManifestStage`. Both regressions failed the named requirement that `HEAD` remain absent. The fixture confirms identical bytes on distinct device/inode coordinates while retaining the original stage hard link; a failed fixture setup is not accepted as RED evidence.

Pool observation now verifies each complete stage's exact bytes and retained identity with the existing bounded, no-follow exact-record verifier. Absence remains `Absent`; kind, length, identity, or byte disagreement becomes `Different`, which the pure planner refuses through the existing pool-specific `PoolEntryDiffers`. The public observation documentation states the stage-object binding; logical identity and durable encoding are unchanged. The [pool identity decision](../adr/retention-recovery-pool-identity.md) records why byte-equal adoption and cleanup-time admission were rejected.

The root diagnostic assertion failed when a disposable planner mutation reported the manifest pool instead. Deleting the retained root stage before planning while leaving the correct refusal intact made both preservation assertions fail by name. The unfixed production runs already demonstrated that both head-absence assertions fail when recovery actually commits the head. Mutated and fixed checkouts used separate Cargo target directories.

Focused retention laws and the full all-feature workspace suites passed in debug and release; both Clippy configurations, formatting, source structure, and Markdown passed. Clippy initially flagged the fixture's inline tuple comparison; naming the two identity tuples clarified the intended comparison without changing its oracle. Observation-time inode admission does not prove safety against every later concurrent substitution, full predecessor/closure admission, or physical power loss; the remaining review queue and final independent approval remain binding.
