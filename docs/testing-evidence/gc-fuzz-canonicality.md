# GC parser canonicality oracle repair

Change kind: test-oracle correction for #107's [existing fuzz review finding](https://github.com/flyingrobots/keep/pull/107#discussion_r4146802713). Owner: `@flyingrobots`. Scope: retirement intent, bound retirement receipt and recovery-disposition receipt canonicality at the public codec boundary. No production source, fixture bytes, identity, durable format, recovery protocol or performance contract changes.

## Claim and oracle

An admitted record must have exactly the bytes produced by encoding its admitted semantic value. The retirement receipt is reconstructed from its admitted intent and reported pool-state digest. The comparison also checks semantic values and the intent's reported candidate-set and intent digests. This invariant oracle complements frozen independently constructed conformance vectors; correlated encoder/decoder defects remain a blind spot.

The previous target compared `Admitted*::encoded()` with the slice supplied to `decode()`. Those accessors return the retained input, so their success supplies no canonicality evidence. The replacement exercises public encoders at runtime. It adds no source-string or harness-cardinality assertion.

## Coordinates and experiment

Unfixed branch revision: `4b9c38930f988911ab020b7c42e9221b721933af`, tree `3fa4386348712647213b64a23af125da7b495dc4`. The Docker source was copied from that exact Git archive, then given synthetic commit `6f0fbec73135a0648503a0cdd608c39e607f67f3` with the same tree. The synthetic SHA is not the product SHA.

The repaired target's final SHA-256 is `f31f4218dee13dc7a6b210121accdd683f4c14c5f5c928b8e501ae07cba80efa`. Calibration used the same behavior before rustfmt split the helper signature; subsequent focused validation and the sanitizer campaign used this final file. [Source profile](gc-fuzz-canonicality/source-profile.txt) records the matching Docker file hash, compiler and unchanged production sources.

Each [intent](gc-fuzz-canonicality/intent.patch), [retirement-receipt](gc-fuzz-canonicality/receipt.patch) or [disposition](gc-fuzz-canonicality/disposition.patch) control flips the first encoded byte after checksum construction in that record's production encoder. Controls are applied separately, retaining the same admitted fixture and decoder. They deliberately break a public encoder output; they do not alter the fuzz assertion or manufacture a setup failure.

| Runtime claim | Original target with faulty encoder | Repaired target with faulty encoder | Repaired target, restored encoder |
| --- | --- | --- | --- |
| Intent canonicality | [Survived, exit 0](gc-fuzz-canonicality/intent-original.txt) | [Named assertion, exit 77](gc-fuzz-canonicality/intent-corrected.txt) | [Pass, exit 0](gc-fuzz-canonicality/intent-restored.txt) |
| Bound retirement-receipt canonicality | [Survived, exit 0](gc-fuzz-canonicality/receipt-original.txt) | [Named assertion, exit 77](gc-fuzz-canonicality/receipt-corrected.txt) | [Pass, exit 0](gc-fuzz-canonicality/receipt-restored.txt) |
| Disposition canonicality | [Survived, exit 0](gc-fuzz-canonicality/disposition-original.txt) | [Named assertion, exit 77](gc-fuzz-canonicality/disposition-corrected.txt) | [Pass, exit 0](gc-fuzz-canonicality/disposition-restored.txt) |

[Calibration transcript](gc-fuzz-canonicality/calibration.txt) preserves seed 107 and observed statuses. This is old-oracle/new-oracle calibration against deliberate runtime mutations, not a claim that the unmodified production encoders have a demonstrated defect or that every compared coordinate was independently mutated. No mutation remains in the candidate. The replaced assertions are deleted because they fail calibration; their intended risk is now covered by the canonicality relation.

The [original-record archive](gc-fuzz-canonicality/original-records.tar.gz) preserves the tool output and initial mutation patches byte-for-byte. Readable `.txt` copies remove trailing whitespace and terminal blank lines to satisfy the repository whitespace gate; diagnostics and outcomes are unchanged. The standalone mutation patches use one context line to avoid trailing blank context while making the same source mutation. The required whole-tree whitespace check exposed this packaging issue after the first repair commit; it is not a runtime failure.

## Replay and resource profile

Run in a copied Docker checkout with Rust 1.96.0, a distinct Cargo target directory and the checked-in lockfiles. Build the fixed-input runner with `cargo build --locked --manifest-path fuzz/Cargo.toml --bin gc_format`. This stable replay runner executes the real target on fixtures; it is not a coverage-guided or sanitizer campaign.

The permanent seed recipes remain in `xtask/src/fuzz_seed_corpus/gc_seeds.rs` and their input bytes in `conformance/segment-store/v2/`. Intent framing is selector 0 plus decoded `one-candidate-gc-intent.hex`. Retirement receipt framing is selector 1, the big-endian u32 intent length, intent bytes, then decoded `one-candidate-gc-receipt.hex`. Disposition framing is selector 2 plus decoded `one-orphan-retire-disposition.hex`. These already-small valid witnesses need no new corpus fixture. No newly discovered production counterexample requires reduction.

Apply one recorded patch to the copied parent source and run both its original target and the repaired target against the corresponding seed with `-runs=1 -seed=107 -timeout=5 -rss_limit_mb=1024`, under an outer 75-second deadline. Restore the encoder before the next control. The seed is recorded outside the child before launch. The three original-target executions must succeed, each repaired-target mutation must fail the matching named canonicality assertion, and each restored execution must succeed.

The medium, single-machine fixed-input replay uses owned seed files, Rust's actual codec implementation and no remote services. Its libFuzzer input timeout is 5 seconds and RSS limit is 1024 MiB; the outer process deadline is 75 seconds. RSS monitoring is not a hard kernel memory limit. The stable replay inherits the Docker network namespace and does not establish network-denial compliance. This bounded evidence does not waive the ordinary-test resource-enforcement gaps recorded on main or assert repository-wide compliance.

The separate coverage-guided run uses pinned cargo-fuzz 0.13.2 and nightly-2026-07-24 with its default address sanitizer. It ran offline in an isolated network namespace (`unshare -n`), starting from the three canonical seeds. Build deadline: 600 seconds. Exploration: `-seed=107 -max_total_time=15 -timeout=5 -max_len=1048576 -rss_limit_mb=1024 -print_final_stats=1`, outer deadline 75 seconds. These are the checked-in smoke bounds in `fuzz/campaign.env`; the fixed seed is additional replay evidence. A time-bounded campaign can explore different numbers of inputs across machines even with the same seed.

## Validation and remaining acceptance

[Focused validation](gc-fuzz-canonicality/focused-validation-formatted.txt) passes root and fuzz formatting, changed-target Clippy with warnings denied, and existing public intent/receipt/disposition codec laws in debug and release. The [initial formatting failure](gc-fuzz-canonicality/focused-validation.txt) is retained; no runtime test ran in that attempt. The existing documentation container had completed its configured lifetime before the first Markdown attempt, so that attempt ran no lint; after its terminal state was verified and it was restarted, the changed Markdown files passed the pinned tool. Initial source-copy setup also invoked a login shell without Cargo on PATH; the explicit tool PATH corrected setup before builds, not a runtime failure.

The [fixed-seed sanitizer smoke log](gc-fuzz-canonicality/fuzz-smoke.txt) and [random-seed follow-up](gc-fuzz-canonicality/fuzz-random.txt) record successful execution and no counterexample under the same bounds. Random seed 180510830 was selected and written outside the child before launch; the follow-up retained the first campaign’s derived corpus. It is finite parser exploration, not proof of all inputs, storage recovery, physical power loss, all-platform behavior or complete #107 acceptance.

The larger PR still requires current-main integration, the remaining review dispositions, full stable-candidate validation and independent exact-head approval. These focused results do not transfer earlier CI or approve the other features in #107.
