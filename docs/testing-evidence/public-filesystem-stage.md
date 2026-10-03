# Public filesystem-stage evidence

Issue #147 closes the missing public integration target from original completed T-11.3. Change kind: correction of missing verification, with no product behavior or format change. Owner: `@flyingrobots`. The branch starts at main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`; the PR supplies the candidate commit and hosted validation identity. This receipt does not claim mainline integration before merge.

## Claims and independent oracles

`tests/segment_filesystem_stage.rs` enters through `FilesystemPlatformAdmission::initialize`, `FilesystemCatalogPublisher::open`, and the public stage/writer API. It requires actual production admission; unsupported filesystems fail setup rather than silently switching to unchecked authority. The target runs on Linux, matching the supported production platform; it is compiled out on other operating systems, whose platform refusal has separate coverage.

| Claim | Runtime observation and oracle |
| --- | --- |
| An existing fixed-name stage cannot be replaced | Exact `SegmentStageCreateError::Create` with `AlreadyExists`, followed by byte-for-byte comparison with independently supplied existing evidence. |
| A second creation cannot take an active stage | First stage receives the canonical header; the competing public creation returns the same exact typed refusal and leaves those bytes unchanged. This is an explicitly ordered contention schedule, not a stress test. |
| Explicit sealing produces canonical storage bytes | The actual staging file equals the frozen independently derived `one-zero-segment.hex` vector. |
| Dropping before sealing preserves evidence without publication | The actual file equals the header prefix of the independent empty-segment vector; complete-segment admission refuses with `WrongLength { minimum: 192, observed: 64 }`; no `HEAD` exists. These sizes come from the version-one format, not a test-case count. |

The drop test establishes preserved evidence and refusal of complete admission. It does not claim automatic prefix disposal, resumed publication, process-death coverage, or power-loss persistence. Existing public writer phase-injection, golden, corruption and segment-resume laws remain in place. Private adapter tests remain separate evidence; their unchecked initialization is not the admission path used here. No production accessors, bypasses or dependencies were added.

## RED, calibration and GREEN

On the exact parent above, `cargo test --test segment_filesystem_stage --all-features --locked` refuses because the target does not exist. This is static delivery evidence for the missing target, not runtime bug evidence. The new runtime laws pass against the unchanged production implementation; this change does not invent a product bug to obtain RED.

Separate copied source trees and dedicated build directories falsified each distinct load-bearing outcome. Calibration changes were never applied to the candidate or committed.

| Production mutation | Observed runtime failure |
| --- | --- |
| Replace exclusive creation with create/truncate | Both creation laws reject the unexpected successful second authority. |
| Overwrite existing bytes before returning the correct `AlreadyExists` refusal | Both creation laws fail their preserved-byte assertions. |
| Replace the creation error kind with `PermissionDenied` | Both creation laws fail exact typed-refusal assertions. |
| Write a zero header instead of the canonical header | Sealed-golden and dropped-prefix comparisons fail. |
| Write `HEAD` during stage creation | The drop law fails its no-publication assertion. |
| Report zero as the minimum complete-segment length | The drop law fails its exact `WrongLength` assertion. |

These failures execute the named assertions. Initial harness diagnostics are retained separately: a symlinked scratch root was correctly refused with `AdmitPlatform/FilesystemLoop`, Clippy required the sandbox module visibility used by other integration targets, the calibration postprocessor lacked `rg`, and source-structure checking required initializing the isolated copy as a Git repository. None is counted as assertion calibration or a product defect.

## Execution and limits

Validation runs in a copied Linux aarch64 Docker source tree with pinned Rust 1.96.0. The test scratch directory is a private ext4 bind mount inside the container, with no writable host checkout mount. Build outputs have a dedicated directory, separate from calibration builds. Production platform admission is unmodified. Debug and release run the exact requested Cargo target; related segment, writer and recovery-resume targets also run in both profiles. Formatting, warnings-denied Clippy and source-structure checking cover the change; final hosted checks are recorded on the PR.

All new laws are medium-size filesystem tests. Additional serial and parallel binary runs use `unshare -n`, a 1 GiB address-space cap and a 30-second suite deadline. Initial debug/release observations completed below one second; the ceiling is conservative runaway protection, not a performance guarantee. Network isolation and process limits apply to these additional runs; ordinary repository CI still has the per-test resource-enforcement gaps recorded in the [enforcement profile](../testing/enforcement.md). Scratch ownership isolates mutable state, but this is not a filesystem-access-denying sandbox. No suite p95 or flake-rate claim is made.

Replay with `cargo test --test segment_filesystem_stage --all-features --locked` and its `--release` variant on an admitted ext4 scratch root. No random inputs or seeds are consumed; the interruption schedule is exactly begin, then drop, with creation contention ordered first-owner before second-request. Independent process namespaces and per-process scratch names permit concurrent runs. There is no performance change, parser change, new crash campaign, or durability-protocol modification.

Keep the laws while their public contracts exist. Delete only when the contract is removed or stronger public-boundary evidence demonstrably subsumes it; preserve the golden and phase-injection owners. The original roadmap checkbox is unchanged.
