# Independent scoped delta confirmation — PR #107

Exact reviewed successor: `e49fde520e9fbea10c899092387b529808e372d0`. Exact delta: `c5b3e349a5953686f10decb18532bb3981f9f4bb..e49fde520e9fbea10c899092387b529808e372d0`.

The P4 evidence overstatement in the preceding [independent review](https://github.com/flyingrobots/keep/pull/107#issuecomment-5977967778) is closed. No verified P0–P5 finding remains in this documentation delta. The prior runtime/path/evidence review applies to the unchanged implementation and tests. This confirmation approves the combined lower-executor outcome and shared recovery-progress slice with its corrected documentation; it does not approve the whole PR, resource-policy disposition, publishing or merging.

## Verification Checklist

- **Exact head and cleanliness:** independently checked the author checkout at `e49fde520e9fbea10c899092387b529808e372d0`, with no tracked or untracked changes. Its sole parent is reviewed head `c5b3e349a5953686f10decb18532bb3981f9f4bb`.
- **Entire delta:** independently inspected the complete commit and diff. Only `docs/formats/segment-store-v2/compaction.md:72` changes: one physical paragraph is replaced. No production source, tests, evidence receipts, dependencies, format, constants, public APIs or runtime sequence changes. The `src` and `tests` tree object identities each match the preceding reviewed head exactly.
- **P4 closure:** the general progress-law claim now lists action/boundary/effect reports, earlier completions and namespace/reopened observations without claiming that all those laws assert preserved typed causes. This matches the actual assertions at `src/adapters/compaction/recovery_progress_tests.rs:84–123` and `src/adapters/compaction/recovery_head_progress_tests.rs:50–82` and `115–131`, which I inspected again at the successor.
- **Specific cause claim:** the corrected paragraph attributes the original typed EISDIR assertion to the Linux unlink-refusal law. I inspected `src/adapters/compaction/recovery_uncertain_tests.rs:59–66`: it downcasts the driver's original source to `io::Error` and requires Linux errno 21. This supports the narrowed claim. The unsupported raw-substitution isolation limitation remains explicit.
- **Surrounding contract:** inspected `compaction.md:64–82`; the implementation promise to preserve typed causes remains distinct from the narrower account of tested evidence. Preflight versus execution, separate prior completions and failing-action effects, first-error stopping, fresh retry observation, unconfirmed durability and no retrospective certification remain unchanged. The consolidated evidence link remains intact.
- **Paths and errors:** there are no changed runtime paths in this delta. The preceding full checklist traces both `Derivable` compaction and `Unpublished` ingestion, complete/truncated stage cleanup, both head actions and lower retry outcomes, every post-mutation failure, original-handle guards and source chains. Identical source/test trees preserve the coordinates and conclusions of that review; this static confirmation does not create new runtime coverage.
- **Merges and integration constraints:** no merge commit occurs in this delta. The prior scoped review's inherited `864f804816ba5b201902143dd7b160ff6ff4b3c9` integration analysis remains applicable; this prose edit changes none of those invariants. No unrelated merge or whole-PR approval is inferred.
- **Constants and numeric claims:** no constants or new numeric figures are introduced by the correction. The EISDIR wording is supported by the existing exact errno-21 assertion. Historical test counts, profile receipts, allocation evidence and resource figures are unchanged; they retain their previously reviewed coordinates and limitations.
- **Repository standards and falsification:** this is a documentation correction. The preceding finding supplies concrete before/after evidence: the old paragraph attributed source assertions to six laws lacking them; the corrected paragraph removes that attribution and names the law that does contain the assertion. No artificial runtime test or renewed calibration is needed for this prose edit. The paragraph remains one physical line.

## Executed versus inspected

Executed: Git exact-head/parent/cleanliness checks, full delta inspection, unchanged `src`/`tests` tree-identity checks, and `git diff --check` for the exact delta, which passes.

Inspected: corrected paragraph and surrounding contract; unchanged progress/head/unlink assertions; preceding full independent review and its recorded execution boundaries. The parent reports Docker Markdown lint passed; I did not independently execute or inspect a new Markdown-lint receipt in this confirmation.

No new builds or runtime checks were executed. The previous exact-head review executed 14 compaction laws and four lower synchronization-failure laws at `c5b3e349a5953686f10decb18532bb3981f9f4bb`; the source and test trees are unchanged here. Prior release/public/writer/static receipts remain inspected evidence, not newly executed checks. No physical power-loss, failed-kernel-fsync, benchmark, broad crash/fuzz or hosted-CI conclusion is added.

No worker, image, container, volume, target or cache was created or used for this static confirmation. No source edits, pushes, comments, merges, configuration changes or cleanup were performed by the reviewer.

Forward-compaction bounds, GC/disposition effects and authority, direct-caller/nested-namespace obligations, per-test resource-policy disposition, full final validation and exact-head whole-PR acceptance remain open. The remote PR was not advanced or adjudicated by this confirmation. This is a scoped implementation/evidence approval only.

APPROVE
