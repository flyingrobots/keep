# Generated catalog model evidence

Change kind: test-evidence enhancement for #166, found by the T-12.3 audit under #131. Owner: `@flyingrobots`. The subject is Keep's public catalog construction, admission, snapshot lookup and successor refusal behavior. Production code and durable formats are unchanged.

## Oracle and bounded claim

The new model chooses membership before encoding. Chunk expectations use explicit source bytes and `ChunkId::hash_bytes`; layout expectations use the frozen `empty` and `one-zero` identities from `conformance/layout/v1/layouts.tsv` and their literal record fixtures. It never derives expected membership or payloads from segment iteration, catalog enumeration or returned records. Chunk hashing is still shared with production; this suite does not independently establish hash correctness. The existing independent format and identity corpus remains necessary.

All subsets of those four records form the generation states. The generator visits all three-generation histories in lexicographic mask order, with bundled segments and with reversed, separately packed records. Assertions compare exact returned bytes, absence, logical record count and each pinned generation after every snapshot has been created. Valid successor results are compared with input-derived generation arithmetic. Layout inclusion here is catalog membership, not a claim of retained closure or natural chunk boundaries.

Separate generated laws reject stale and skipped successor generations, a wrong predecessor digest, and a head/catalog generation mismatch with exact typed coordinates. These supplement existing named examples; they do not replace the format, restart, concurrency or crash suites.

## Replay and reduction

Run `cargo test --locked --test catalog_model generated_histories` in the copied Docker source, adding `--release` for optimized execution. The external runner records this command and source commit before launch. Enumeration has no random seed or environment-controlled input. Every assertion identifies its history or mask and relevant packing/generation coordinates; the complete finite input space is reconstructible even if the child crashes.

The deterministic reducer is exhaustive replay in the same ascending mask-tuple order, stopping on the first failure. This identifies the least failing tuple in that bounded order without relying on a random seed or a large opaque counterexample. It does not minimize arbitrary-length histories or prove behavior outside the fixed record universe and three-generation bound. Newly discovered production failures must additionally become named permanent regression cases; none was found in the unmutated implementation during this change.

## Calibration

The test source was committed as `c9b41e99fb7e89217afbd896732dbea09a0c97ab`, on production base `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. Subsequent formatting changes do not change the calibrated assertions. Each mutation below changed only a production file in an isolated Docker copy; the normal working tree retained the correct implementation.

| Violated behavior | Production mutation | Observed runtime RED |
| --- | --- | --- |
| Present records remain retrievable | Filter every admitted catalog lookup result away | Exact payload/absence assertion |
| Absent records remain absent | Failed binary search falls back to the first binding | Exact payload/absence assertion |
| Logical record count describes the selected catalog | Return zero from admitted catalog count | Membership count assertion |
| Snapshot reports its pinned generation | Return generation one from snapshot generation | Pinned generation assertion |
| Successor proof reports its admitted candidate | Return generation one from successor generation | Successor assertion |
| Stale/skipped candidate reports the specified generation refusal | Invert the generation mismatch guard | Refused-operation check |
| Wrong predecessor reports the specified digest refusal | Invert the predecessor mismatch guard | Refused-operation check |
| Head and catalog generation mismatch remains precise | Invert the head generation mismatch guard | Exact typed generation assertion |

The original handwritten model passes the absent-record mutant: it checks present records but never queries an absent identity. The generated law fails against the same mutated production lookup. This is direct evidence that the additional runtime oracle catches a defect the original model missed; it is not a claim that main contained that production defect.

The first missing-record mutant failed compilation because it made a private method unused; that result is excluded. The corrected mutant retained the call and filtered its result, producing the recorded assertion failure. An initial GREEN attempt reused a cached mutant binary after timestamp-preserving restoration; that result is also excluded. The isolated Keep package artifacts were then invalidated and the unmutated source rebuilt before successful debug/release execution. A source-policy invocation without Git metadata failed setup and required a correctly initialized copied repository.

## Execution and limits

The laws execute in memory with the pinned Rust 1.96.0 toolchain on Linux Docker. The serialization sink supplies owned bytes and makes no filesystem durability claim. No clock, random source, sleep, scheduling dependency or network is consulted by the test. The proposed size is small; ordinary Cargo execution does not enforce the repository's desired per-test filesystem/network/time/memory ceilings, as documented in `docs/testing/enforcement.md`. This record does not claim those enforcement gaps are closed or waived.

Focused debug/release execution and Clippy pass. Formatting, source policy and final hosted checks are recorded on the PR's exact head before readiness. Broad checks from another SHA are not transferred. Raw calibration and setup receipts remain with the review evidence; the mutation descriptions and commands above permit reproduction without machine-local source paths.

Delete these tests only if the catalog contract is removed or a stronger, cheaper independently calibrated model subsumes their claims. A production bug found by generation keeps its minimized named counterexample even if the exploratory generator is later replaced.
