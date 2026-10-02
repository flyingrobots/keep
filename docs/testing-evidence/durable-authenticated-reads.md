# Durable authenticated reads (#109)

Change kind: new read API with a shared-core extraction. The baseline is main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. No write protocol or on-disk format is changed; #125's candidate-catalog publication gate is not added or bypassed. This is an in-progress implementation ledger, not acceptance of #109.

## Delivered candidate behavior

`DurableStore` pins a fresh `DurableSnapshot` per convenience call. Snapshot admission keeps the existing shared reader fence, catalog, retention head and manifest view and verifies every selected retained closure against the catalog. Blob lookup scans selected roots one at a time and chooses the lowest retained layout identity; exact-layout reads use the catalog directly. Reconstruction and ranges use the existing immutable reference cores and return receipts with the view coordinates only after emission succeeds.

The allocation, blocking and failure contract is in the public API documentation and [rationale](../../src/adapters/durable/rationale.md). Catalog and selected segment bytes are materialized under explicit caller policy; the API does not claim lazy segment reads or constant total memory. It adds no whole-blob output buffer or aggregate anchor index.

## Closure ledger

| Obligation | Current evidence | Required exit condition |
| --- | --- | --- |
| Golden bytes and exact read coordinates | `durable_read_law_tests` reconstructs the independent one-zero corpus and exercises empty/nonempty ranges and unretained-blob refusal. | Extend receipt assertions to independently expected digests and cover all relevant reference read laws. |
| Stable retained view across publication | `durable_view_law_tests` releases the retained root while an older snapshot remains alive; the old reader emits its original byte and head generation, and a fresh view sees release. | Add catalog-generation successor/restart evidence without introducing unsupported v2 publication. |
| Collector exclusion | The public snapshot holds the actual kernel shared fence; a deterministic exclusive try-lock refuses until drop. | Preserve this claim as fence evidence; actual GC execution remains absent on main under #21. |
| Operational failures versus evidenced absence | Removing the selected segment yields exact `OpenSegment` / `NotFound` through the public snapshot error. | Add missing catalog member, corruption and output-failure laws with exact typed coordinates and source preservation. |
| Reference-core equivalence | Existing reference algorithms are generalized only over a crate-private immutable source; their single-pass authentication tests are retained. | Run generated reference properties plus complete debug/release validation, and calibrate distinct new load-bearing assertions. |
| Golden File Worldline | Not yet connected to the durable reader. | Run its restart and range assertions against the durable backend with independent expected bytes and identities. |
| Documentation and final acceptance | API costs and design rationale are documented. | Update reconstruction requirements and Linux example only after complete evidence; run required final checks and reconcile exact-head review. |

## Evidence limits

The initial filesystem laws use owned test directories and the existing repository migration fixture with platform admission bypassed; they establish read behavior on the test filesystem, not production platform admission. No sleep or probabilistic race is used. The collector test uses a kernel try-lock in one process, not a full GC integration or process-death claim.

New laws are medium-size because they own filesystem state. Per-test resource enforcement and suite SLOs remain gaps described in the repository [enforcement profile](../testing/enforcement.md); no compliance waiver or new resource ceiling is claimed here. Deletion criteria and oracles appear beside the laws.

Initial debug/release golden-read checks and all-feature Clippy passed; expanded view laws passed in debug. Initial authoring/compilation and Clippy failures are retained in the local evidence logs and are not product RED evidence. Final source SHAs, complete check results and actual mutation observations belong in the final PR receipt. This new API did not exist on the parent; parent compilation failure would not prove a runtime regression.

## First implementation receipt

Implementation commit `d6c38a1` passed formatting, source structure, both workspace Clippy feature profiles, complete all-feature workspace debug/release tests and doctests, and rustdoc generation on pinned Rust 1.96.0 in a copied Docker source tree. These runs include the existing generated range-to-source-slice properties and reference single-hash-pass laws. Subsequent documentation and visibility narrowing require their own final-head checks; this receipt does not transfer to future semantic changes.

Three isolated source/build mutations were observed RED on that implementation: substituting `[1]` during emission after verification made whole-read, exact-layout and nonempty-range assertions report expected `[0]` versus observed `[1]`; advancing the receipt's catalog generation made its coordinate assertion report expected 1 versus observed 2; replacing shared fence acquisition with unlock made the collector-exclusion assertion report expected `WouldBlock` versus observed successful acquisition. These are runtime calibration, not parent-bug reproductions or an assertion that every new check has already been calibrated.

The remaining closure-ledger obligations above are still open, including generated multi-chunk durable laws, Worldline restart/range witnesses, precise missing-member/output failures, independent digest expectations and complete final-head review/CI. The issue stays open and its PR remains a draft until they are satisfied.
