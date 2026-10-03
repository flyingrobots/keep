# Sealed-stage observation capability

Change kind: bug fix for #146, with a cohesive replacement observation boundary for its crash-harness consumer. Owner: `@flyingrobots`. The branch starts at main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`; it is independent of the public-stage and catalog-admission follow-up branches. No on-disk format, identity, publication phase or new dependency is introduced. Removing the feature-gated public `map_stage` method intentionally breaks callers that requested writable authority from an already sealed receipt.

## RED and protected boundary

The permanent compile-fail example in `SealedSegment` attempts to call `map_stage` and write another byte through its callback. On the unfixed parent, `cargo test --doc --all-features sealed_segment --locked` failed with “Test compiled successfully, but it's marked compile_fail.” Regression commit `11be73d` preserves that RED. This is compiler-enforced capability evidence; it is not represented as a filesystem failure. With the arbitrary conversion removed, the same example passes by refusing access to that API.

The replacement stores both stage and observer in private fields. Observation receives only requested/actual write lengths and explicit before/after flush/synchronize events. `without_observer` is implemented only for the built-in wrapper and returns another sealed receipt over the same stage; no callback or public extractor receives writable storage. Generic `close` still drops the stage. Filesystem selection still checks its original publisher authority and admitted segment coordinates. The usual `SegmentStage` exclusive-ownership precondition remains; a caller's independently duplicated storage handle is not made exclusive by a wrapper.

## Runtime evidence and oracles

| Claim | Evidence and oracle |
| --- | --- |
| Removing observation preserves bytes and private publisher provenance | Actual admitted ext4 stage equals the independently specified empty-segment golden; selection by its original publisher succeeds after observer removal. |
| Short-write observation preserves ordinary writer output over the tested input family | Generated payload lengths 1 through 64, with a fixed repeating byte pattern, compare observed seven-byte-prefix writes against plain filesystem writing. Every case owns fresh storage. Ascending exhaustive exploration reports the smallest failing length in this family; the independent golden complements this shared-implementation differential oracle. |
| Excessive observer limits refuse before writes | Exact header `SegmentWriteError::Write`, zero prior completed bytes and `InvalidInput`; the actual stage remains empty. |
| A post-write interruption cannot retry completed effects | Exact header write refusal retains the nested original `Interrupted` cause, with a non-retryable outer kind; actual bytes equal exactly one golden header. The prior-completed-call offset is not a rollback claim for this failed call. |
| Observation cannot fabricate successful durability | Faulting stage ports drive the production writer through the observation wrapper; exact prefix flush/synchronize failures preserve errno 5 and produce no sealed receipt. These are port-level fault simulations, not physical disk failure evidence. |
| Crash injection remains connected to production operations | Complete existing debug and optimized process-death matrices pass after replacing the writable decorator with `CrashSegmentObserver`. Header, record and seal interruption offsets, before/during/after choices, flush/sync ordering, restart bytes and immutable admission oracles remain unchanged. |

Separate copied mutations demonstrated runtime RED for skipped underlying flush, skipped underlying synchronization, clamped excessive limits, retryable post-effect interruption, and zeroed writes. Each used a dedicated build directory. Zeroed writes also failed the generated differential assertion at its first payload length. These are distinct outcome calibrations, not a mutation score or harness-case-count claim. The mutation sources were not committed.

## Execution and limits

All Rust execution uses copied Docker source, pinned Rust 1.96.0 and dedicated output on Linux aarch64. Filesystem laws use actual production platform admission on private ext4 scratch; no writable host checkout mount or fabricated admission proof is used. The capability regression is run with `--all-features`, since that is what exposed the old method. Both feature configurations are checked with warnings-denied Clippy. The final PR records exact candidate and hosted-check SHAs alongside full debug/release, formatting, source-structure, documentation and existing golden/corruption/memory evidence.

Filesystem laws are medium; the faulting durability-port laws and capability example are small/static evidence. No random seeds, sleeps or uncontrolled scheduling drive the oracles. The differential input family is bounded and deterministic; it does not establish equivalence for arbitrary payloads or malicious storage implementations. Replay with `cargo test --test observed_segment_stage --all-features --locked`, its `--release` variant, and the all-feature doctest command above. Re-run the production campaigns with `cargo xtask durability-crash-matrix` and its optimized invocation.

Existing resource-enforcement and suite-budget gaps remain those disclosed by the binding testing enforcement profile; this change claims neither a new sandbox policy nor measured latency SLOs. Process death and simulated errno failures do not establish physical power-loss behavior. No performance optimization is claimed. Retire the laws only if the capability is removed or stronger boundary evidence demonstrably subsumes the risk. Original roadmap checkboxes and unrelated recovery/admission scope remain unchanged.
