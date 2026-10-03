# Partial segment seal corruption

Change kind: bug fix for [#171](https://github.com/flyingrobots/keep/issues/171), under the T-13.2 audit. Owner: `@flyingrobots`; the author supplies execution evidence. The contract is KEEP-RECOVERY-010: demonstrated available fixed-framing corruption must refuse classification and assessment before a discard plan can be constructed.

## Parent RED and correction

Main `6051abb25a9fd33ae7ee0de5614514b709a4d82a` accepts a canonical one-zero segment truncated after seal version 2 as lawful truncation. The [parent RED receipt](partial-seal-corruption/parent-red.txt) records both public classifier and fingerprint-bound assessment regressions failing after successful compilation. The tests are committed separately at `1b6da984ece52e94777fe1b52dcf1bb6b82c3960`; run `cargo test --locked --test recovery_partial_seal` at that revision to reproduce the refusal gap.

The classifier now validates the available fixed fields of a recognized incomplete seal before returning `Truncated`. It preserves the exact `SegmentSealError` under `RecoverySegmentStageError::Seal`; assessment preserves that source. This is read-only slice classification, not a filesystem deletion experiment. The borrowed bytes are immutable by API construction; an equality assertion on the same untouched caller buffer would add no runtime evidence.

The finite sweep mutates each version, flags, embedded length, reserved and algorithm byte with a specified independent expected error, then visits every recognized incomplete seal length. A mutation beyond the observed end must remain a typed truncation; an observed contradiction must yield the exact specified refusal. The inputs and order are deterministic and finite, with offset/observed-length replay coordinates on failure. This is a fixed-framing family sweep, not arbitrary-byte coverage or proof of full future completion feasibility.

## Execution and current limits

[Focused and adjacent recovery GREEN](partial-seal-corruption/recovery-green.txt) covers the new public laws and existing classification, assessment and discard laws in debug/release. [Framing validation](partial-seal-corruption/framing-green.txt) includes formatting, source structure, all-feature workspace/all-target Clippy and the new laws in debug/release. Commands execute in copied Linux Docker trees with Rust 1.96.0 and the committed lockfile; the laws are small, in-memory and deterministic. Per-test resource ceilings and denied-egress enforcement remain repository gaps, not claimed implemented controls.

Receipt normalization replaces container source/target prefixes with `<isolated-build>` and removes trailing spaces and redundant trailing blank lines; assertion diagnostics and outcomes are preserved. These receipts cover the implementation slice, not final acceptance. Stable-candidate full validation and independent exact-head review remain required before this PR is ready. No physical power-loss, actual unlink, allocation benchmark or complete recovery audit claim is made.

## Permanent counterexample and parser exploration

The regression is reduced to the empty-segment header, seal magic and unsupported version 2 in `tests/fixtures/recovery/unsupported-partial-seal-version.hex`. Removing the record and unused seal suffix preserves both public failures on main, as shown by the [reduced parent runtime RED](partial-seal-corruption/reduced-parent-runtime-red.txt). Earlier compiler/setup failures are excluded from this runtime receipt. The ordinary tests and deterministic fuzz-seed preparation both consume the retained input.

The existing registered `segment_format` target adds selector 5 for the production recovery classifier. Its independent byte-table oracle requires every available fixed seal byte in a returned seal truncation to be canonical. The [fuzz parent RED](partial-seal-corruption/fuzz-parent-red.txt) records the version-byte assertion failing against main's classifier; this is a semantic oracle failure, not merely a parser crash. [Fuzz GREEN](partial-seal-corruption/fuzz-green.txt) records successful single-input replay followed by a seeded bounded campaign on the corrected implementation. Fuzz target Clippy and the instrumented target build also pass.

Replay inside a copied Docker checkout with the pinned `nightly-2026-07-24` toolchain and `cargo-fuzz 0.13.2`:

```sh
cargo xtask prepare-fuzz-corpus
cargo +nightly-2026-07-24 fuzz run segment_format fuzz/corpus/segment_format/recovery-unsupported-partial-seal-version -- -runs=1 -timeout=5 -rss_limit_mb=1024
cargo +nightly-2026-07-24 fuzz run segment_format -- -seed=17101 -max_total_time=15 -timeout=5 -rss_limit_mb=1024 -max_len=1048576
```

The seed and commands were recorded before launch. The campaign uses cargo-fuzz's instrumented release profile and address sanitizer, with libFuzzer timeout/RSS/input bounds; it does not establish exhaustive input coverage, an allocation benchmark, or isolation of every ambient dependency. The baseline replay used unchanged main production with only the new oracle/input copied in; the ordinary parent regression remains separately committed and reproducible. Future fuzz findings retain minimized reproducers through the repository's existing corpus workflow.

## Direct assertion calibration and seed-count retirement

The original parent RED establishes missing refusal, not the later exact diagnostic/source assertions. Review required additional direct calibration. These production mutations compile and reach their named runtime checks; [restored debug/release and Clippy](partial-seal-corruption/calibration-green.txt) pass after restoring source and invalidating timestamps.

| Broken contract | Replay patch | Observed failure |
| --- | --- | --- |
| Diagnostic reports a false observed value | [wrong diagnostic](partial-seal-corruption/wrong-diagnostic.patch) | [Exact expected/observed error comparison](partial-seal-corruption/wrong-diagnostic-red.txt) |
| Error loses the original seal source | [dropped source](partial-seal-corruption/dropped-source.patch) | [Required typed cause becomes absent](partial-seal-corruption/dropped-source-red.txt) |
| Canonical incomplete seal reports the wrong observed length | [wrong coordinate](partial-seal-corruption/wrong-coordinate.patch) | [Unobserved mutation must retain exact truncation](partial-seal-corruption/wrong-coordinate-red.txt) |

Apply one patch to a clean copied candidate, touch its changed Rust file, run `cargo test --locked --test recovery_partial_seal`, reverse only that patch, touch the restored file and rerun debug/release. The expected failures are assertion failures after compilation, not exit status alone. Patches do not belong in a committed production tree.

The first full candidate run also encountered `seed_preparation_materializes_the_complete_deterministic_set`: it failed because adding recovery inputs changed its frozen cardinality. Those counts neither asserted Keep behavior nor established that any seed reached its parser. They are retired under the no-protected-contract deletion criterion. The replacement feeds the actual materialized recovery input to Keep's classifier and requires the precise seal refusal; repeated preparation still must preserve the complete emitted bytes. This is tool-to-runtime replay evidence, distinct from the small direct classifier law and the fuzz campaign.

The emitted-input witness is calibrated with a [wrong selector](partial-seal-corruption/materialized-selector.patch), [missing named input](partial-seal-corruption/materialized-absent.patch), and the same wrong-diagnostic production patch above. The respective runtime RED receipts are [selector](partial-seal-corruption/materialized-selector-red.txt), [absent input](partial-seal-corruption/materialized-absent-red.txt), and [diagnostic](partial-seal-corruption/materialized-diagnostic-red.txt). Run `cargo test --locked --package xtask --bin xtask seed_preparation_preserves_a_replayable_recovery_counterexample` with each mutation separately. Earlier setup attempts selecting zero tests are excluded; the admitted receipts execute and fail the named law.

After all three emitted-input mutations are removed, [restored materialization GREEN](partial-seal-corruption/materialized-green.txt) records the named law executing in debug/release, followed by workspace Clippy and source-structure validation. The remaining broad acceptance chain is rerun on the resulting stable candidate; the earlier full run's seed-count failure is not hidden or treated as a product regression.
