# Complete reader coordinate evidence

Change kind: bug fix. Subject: Keep runtime reader selection. Owner: `@flyingrobots`. Oracle: specified equality of every validated head coordinate across reader collection.

The medium filesystem regression `a_changed_catalog_length_refuses_the_loaded_reader_view` was observed RED on unfixed head `206ef7b47041926e730e906a786804a6da973c09` with its test overlay; regression commit `953f7fc` preserves the reproduction. The original code accepted the view and failed the named refusal assertion after a correctly checksummed HEAD changed only its admitted catalog length.

The deterministic schedule runs the production filesystem source under a real shared reader fence, changes HEAD after loading, and requires `AttemptsExhausted { attempts: 1 }` under a single-attempt policy. The added length is one canonical catalog entry, 160 bytes, at the version-one head's catalog-length offset; the changed head is admitted before installation. The initial one-byte length perturbation violated the length congruence rule and is excluded as fixture setup failure.

Small storage-independent laws assert the returned semantic view, never a harness call count. One generated domain covers every admitted manifest length above the minimum with the original generation, digest and predecessor fixed; another covers every nonzero first predecessor byte with all other bytes fixed. Both require the later stable view instead of the superseded view.

The generation/digest pinning and moving-error regression setup was adapted to the catalog tuple's extra field without changing their observable expectations. The public coordinate value now requires complete retention heads rather than generation/digest pairs.

Independent copied-source mutations used fresh Cargo target directories. Normalizing the filesystem source's catalog length to a constant failed the exact single-attempt refusal assertion. Comparing retention heads only by generation and digest failed both generated semantic laws, which returned `superseded` instead of `stable`; the first counterexamples were the next admitted manifest length and predecessor byte one.

The generated domains are deterministic ascending enumerations, with the smallest changed value first and its coordinates printed on failure. They use no ambient randomness or schedule; replay is the same focused command and the smallest counterexamples remain represented by those first inputs.

Tests run from copied source in Docker on pinned Rust 1.96.0, Linux aarch64 and the existing ext4 audit filesystem; ordinary per-test resource ceilings remain an explicitly unresolved enforcement gap. Replay uses `cargo test --lib coordinate --all-features`; add `--release` for optimized execution.

Complete workspace all-feature debug and release suites, default-feature and all-feature Clippy with warnings denied, formatting, staged source-structure checking, and Markdown lint passed. Raw runtime RED, excluded fixture failure, mutation and complete validation logs are retained outside tracked source. Core source and regression hashes were compared with the copied Docker files before publication.

The domains do not exhaust predecessor digest space, physical filesystem faults, platform admission, or arbitrary head replacement/restoration between reads. Generic port loaded-value binding and other audit findings remain separate work. No parser, fuzz, process-death, power-loss, performance or durability claim is added by this semantic coordinate change.
