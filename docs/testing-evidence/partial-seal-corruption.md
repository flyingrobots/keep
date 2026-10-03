# Partial segment seal corruption

Change kind: bug fix for [#171](https://github.com/flyingrobots/keep/issues/171), under the T-13.2 audit. Owner: `@flyingrobots`; the author supplies execution evidence. The contract is KEEP-RECOVERY-010: demonstrated available fixed-framing corruption must refuse classification and assessment before a discard plan can be constructed.

## Parent RED and correction

Main `6051abb25a9fd33ae7ee0de5614514b709a4d82a` accepts a canonical one-zero segment truncated after seal version 2 as lawful truncation. The [parent RED receipt](partial-seal-corruption/parent-red.txt) records both public classifier and fingerprint-bound assessment regressions failing after successful compilation. The tests are committed separately at `1b6da984ece52e94777fe1b52dcf1bb6b82c3960`; run `cargo test --locked --test recovery_partial_seal` at that revision to reproduce the refusal gap.

The classifier now validates the available fixed fields of a recognized incomplete seal before returning `Truncated`. It preserves the exact `SegmentSealError` under `RecoverySegmentStageError::Seal`; assessment preserves that source. This is read-only slice classification, not a filesystem deletion experiment. The borrowed bytes are immutable by API construction; an equality assertion on the same untouched caller buffer would add no runtime evidence.

The finite sweep mutates each version, flags, embedded length, reserved and algorithm byte with a specified independent expected error, then visits every recognized incomplete seal length. A mutation beyond the observed end must remain a typed truncation; an observed contradiction must yield the exact specified refusal. The inputs and order are deterministic and finite, with offset/observed-length replay coordinates on failure. This is a fixed-framing family sweep, not arbitrary-byte coverage or proof of full future completion feasibility.

## Execution and current limits

[Focused and adjacent recovery GREEN](partial-seal-corruption/recovery-green.txt) covers the new public laws and existing classification, assessment and discard laws in debug/release. [Framing validation](partial-seal-corruption/framing-green.txt) includes formatting, source structure, all-feature workspace/all-target Clippy and the new laws in debug/release. Commands execute in copied Linux Docker trees with Rust 1.96.0 and the committed lockfile; the laws are small, in-memory and deterministic. Per-test resource ceilings and denied-egress enforcement remain repository gaps, not claimed implemented controls.

Receipt normalization replaces container source/target prefixes with `<isolated-build>` and removes redundant trailing blank lines; assertion diagnostics and outcomes are preserved. These receipts cover the implementation slice, not final acceptance. Stable-candidate full validation and independent exact-head review remain required before this PR is ready. No physical power-loss, actual unlink, allocation benchmark or complete recovery audit claim is made.

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
