# Partial segment seal corruption

Change kind: bug fix for [#171](https://github.com/flyingrobots/keep/issues/171), under the T-13.2 audit. Owner: `@flyingrobots`; the author supplies execution evidence. The contract is KEEP-RECOVERY-010: demonstrated available fixed-framing corruption must refuse classification and assessment before a discard plan can be constructed.

## Parent RED and correction

Main `6051abb25a9fd33ae7ee0de5614514b709a4d82a` accepts a canonical one-zero segment truncated after seal version 2 as lawful truncation. The [parent RED receipt](partial-seal-corruption/parent-red.txt) records both public classifier and fingerprint-bound assessment regressions failing after successful compilation. The tests are committed separately at `1b6da984ece52e94777fe1b52dcf1bb6b82c3960`; run `cargo test --locked --test recovery_partial_seal` at that revision to reproduce the refusal gap.

The classifier now validates the available fixed fields of a recognized incomplete seal before returning `Truncated`. It preserves the exact `SegmentSealError` under `RecoverySegmentStageError::Seal`; assessment preserves that source. This is read-only slice classification, not a filesystem deletion experiment. The borrowed bytes are immutable by API construction; an equality assertion on the same untouched caller buffer would add no runtime evidence.

The finite sweep mutates each version, flags, embedded length, reserved and algorithm byte with a specified independent expected error, then visits every recognized incomplete seal length. A mutation beyond the observed end must remain a typed truncation; an observed contradiction must yield the exact specified refusal. The inputs and order are deterministic and finite, with offset/observed-length replay coordinates on failure. This is a fixed-framing family sweep, not arbitrary-byte coverage or proof of full future completion feasibility.

## Execution and current limits

[Focused and adjacent recovery GREEN](partial-seal-corruption/recovery-green.txt) covers the new public laws and existing classification, assessment and discard laws in debug/release. [Framing validation](partial-seal-corruption/framing-green.txt) includes formatting, source structure, all-feature workspace/all-target Clippy and the new laws in debug/release. Commands execute in copied Linux Docker trees with Rust 1.96.0 and the committed lockfile; the laws are small, in-memory and deterministic. Per-test resource ceilings and denied-egress enforcement remain repository gaps, not claimed implemented controls.

Receipt normalization replaces container source/target prefixes with `<isolated-build>` and removes redundant trailing blank lines; assertion diagnostics and outcomes are preserved. These receipts cover the implementation slice, not final acceptance. Parser fuzz/corpus extension, stable-candidate full validation and independent exact-head review remain required before this PR is ready. No physical power-loss, actual unlink, allocation benchmark or complete recovery audit claim is made.
