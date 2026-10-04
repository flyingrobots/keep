# Retention reader catalog error evidence

Change kind: bug fix. Subject: Keep runtime reader refusal. Contract: `FilesystemRetentionSnapshot::load`. Owner: `@flyingrobots`. Oracle: specified documented error boundaries and the exact catalog restart phase, decoder refusal, and underlying I/O kind.

The public missing-catalog, missing-segment, and corrupt-catalog regressions were observed RED on unfixed head `57cd6df3b15023a9c21307e84f5514040fa4de9e` with the test overlay; regression commit `f5ecbed` preserves the reproduction. Their named assertions reported `View` wrappers where `Catalog` was required, rather than setup or compilation failures.

The corrupt coordinate-HEAD control passed before the fix and requires a `View` refusal with its concrete `PublicationHeadDecodeError::ChecksumMismatch` source, preventing overbroad catalog classification.

All new laws are medium because they operate on owned real filesystem stores. The pending-result schedule runs the production source under a shared reader fence, restores the selected catalog and publishes retention synchronously after the first load, then requires a collected view bound to the new retention head and the original catalog. Its temporary missing catalog is a controlled fault rather than production evidence of a legitimate writer losing published files.

The pinning regression's setup was adapted to the private source's collected admission-result value; its original path-replacement operation and catalog generation/digest oracles are unchanged.

Independent copied-source mutation builds used fresh Cargo target directories. Returning the catalog failure as I/O reproduced the catalog-boundary failures and failed the named moving-head assertion. Hiding the public error's catalog source failed its direct-source assertion. Stringifying a coordinate decoder refusal failed the head's exact typed-source assertion while retaining a `View` error.

Mutating the returned catalog generation failed both pinning and moving-head generation checks. Flipping a returned digest byte failed both digest checks. Omitting the loaded retention state failed the exact newly published head assertion. The initial constant-digest mutation triggered a dead-field compile error and is excluded; the corrected mutation retained the original field read and failed at the runtime digest assertions. The scheduled-publication fixture's initial unused receipt compile error is likewise excluded.

Complete workspace all-feature debug and release suites, default-feature and all-feature Clippy with warnings denied, formatting, and staged source-structure checking passed. The final strengthening of the moving-head assertion and error documentation then passed focused snapshot laws in debug and release, all-feature Clippy, formatting, and staged source checks. Markdown lint passed separately. Raw RED, calibration, and verification logs are retained outside tracked source.

Replay uses `cargo test --lib filesystem_retention_snapshot --all-features` for the reader laws; add `--release` for optimized execution. Full checks use `cargo test --workspace --all-features`, the same command with `--release`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, its default-feature counterpart, `cargo xtask source-structure-check`, and `cargo fmt --check`, all from copied source inside Docker.

The execution profile is copied Docker isolation on pinned Rust 1.96.0, Linux aarch64, and the existing ext4 audit filesystem. The owned scratch stores and synchronous publication schedule control observed inputs. No new network, process-death, power-loss, randomized-input, golden, performance, or fuzz claim is made because this change adds no parser or write protocol.

Ordinary per-test resource ceilings remain the existing enforcement-profile gap, not an approved waiver. Complete head coordinates, full reader platform admission, distinct valid replacement catalogs, and unrelated retention recovery findings remain separate unresolved work.
