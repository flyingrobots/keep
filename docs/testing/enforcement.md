# Testing standard enforcement profile

This profile gives the [Testing Standards](../Testing%20Standards.md) an execution and ownership model. It distinguishes current mechanisms, review obligations and missing automation. A listed requirement is not a claim that the repository already complies.

## Owners and evidence admission

The repository owner in [CODEOWNERS](../../.github/CODEOWNERS), currently `@flyingrobots`, owns this policy, suite budgets, quarantine/waiver approval and enforcement gaps until an explicit subsystem owner is recorded. A change author owns its evidence and replay artifacts. The named PR reviewer verifies them; the author cannot self-certify a risk waiver. An assigned engineer may own implementation of a gap without changing approval authority.

The [PR template](../../.github/pull_request_template.md) is the current review mechanism. Its evidence section must name each changed claim, subject, contract, oracle and falsification result. A reviewer records whether the evidence proves a product claim, a tool claim, calibration, or a static/API restriction. Tool success cannot be substituted for a storage acceptance check. Documentation-only changes identify their document outcome and validation; they do not require artificial runtime tests.

New or changed tests require review of the declarations below. Missing automation must be disclosed and resolved or covered by an explicitly approved, expiring risk decision before claiming compliance. Existing tests remain subject to audit; this profile provides no blanket grandfathering or automatic waiver.

## Required evidence record

Store substantial evidence beside the affected concept or under `docs/testing-evidence/`, and link it from the PR. A small change may put the complete record in the PR body. Each record contains:

- **Claim:** observable promise and failure consequence; product/tool/calibration/static subject; owning contract and named owner.
- **Oracle:** specified/derived/invariant/differential/change-detection class and exact independent source; any correlated-reference limitations.
- **Tests:** law names, feature gates and size classes; reason for each size and any duplicated verification.
- **Execution profile:** immutable source SHA, toolchain, debug/release profile, platform/filesystem, command, admitted dependencies and environment.
- **Ceilings:** per-test time and memory caps, allowed processes/threads/filesystem/network, enforcing runner or sandbox, and suite latency budget. Separate proposed caps from enforced caps.
- **Calibration:** named assertion, violated behavior, RED source or mutation, actual failure output and subsequent GREEN receipt; setup/compilation failures are excluded.
- **Replay:** seed or schedule recorded before launching the child, exact replay command, input classes, reducer and permanent minimized counterexample location.
- **Failure model:** fault-matrix cells including stacked faults, persisted state observed, and which process-death/power-loss assumptions were actually exercised.
- **Limitations:** unexplored inputs/schedules/platforms, missing enforcement, measurement variance, waiver approver/issue/expiry where applicable.
- **Expectation/deletion changes:** old/new contract or oracle correction; deletion criterion and location of remaining verification or unresolved risk.

Fields irrelevant to the change state why they are not applicable. Evidence retains original failures and all diagnostic attempts; a subsequent success is not a replacement artifact. Test metadata may be in a test's oracle note plus this record until a runner can admit the same declarations mechanically; review-only metadata does not implement resource enforcement.

## Present mechanisms and gaps

This table is a source inspection of [CI](../../.github/workflows/ci.yml) and the [scheduled fuzz workflow](../../.github/workflows/fuzz-scheduled.yml) at main `82374a995df095106aefe52f87ab3cb26184639d`. Repository ownership below comes from CODEOWNERS, not an invented implementation assignment. “Gap” means unresolved compliance work, not approved risk acceptance.

| Obligation | Present mechanism | Missing enforcement / next observable outcome | Accountable owner |
| --- | --- | --- | --- |
| Change kind, subject, oracle and calibration | Required PR evidence section and named reviewer; manual admission. | CI does not admit complete per-claim evidence or validate RED source coordinates. A future validator must reject missing/invalid records with a specific reason. | `@flyingrobots`; author supplies evidence. |
| Rust profiles and policy | CI executes debug/release workspace tests, doctests, feature checks, formatting, Clippy and dependency checks. | Green execution is not evidence that assertions are independent or calibrated. Review still checks those claims. | `@flyingrobots`. |
| Test size, resource ceilings and suite latency | Documentation and fuzz jobs have job deadlines; the fuzz runner admits bounded campaign policy. | Ordinary Rust tests have no per-test size admission, filesystem/network sandbox or per-test time/memory ceilings. Add an execution profile that refuses forbidden resources and records violations; measure suite distributions before setting numeric budgets. | `@flyingrobots`. |
| External seed retention and minimized reproducers | Fuzz inputs/seeds and failure-artifact upload exist; the scheduled workflow manages corpus artifacts. | This does not establish outside-process seed/schedule capture or mandatory reduction for every property, concurrency and crash test. A launch wrapper must persist a replay record before child execution and retain it on failure. | `@flyingrobots`. |
| Quarantine and XFAIL symmetry | Policy and PR review only. | No dedicated runner/ledger currently establishes expiry, continued diagnostic execution, first-failure preservation or unexpected-pass rejection. Implement and validate these outcomes before describing a test as quarantined or XFAIL-compliant; `#[ignore]` is insufficient. | `@flyingrobots`. |
| Fault coverage and persistence model | Debug and optimized production process-death campaigns run in CI. | Enumeration does not establish each promised fault type, stacked failure or power-loss persistence model. Own a subsystem fault matrix and attach actual reopened-state evidence for each claimed cell. | `@flyingrobots`; affected subsystem owner once named. |
| Trust and latency review | Owners review PR evidence. | No suite-wide measured SLO, flake budget or quarterly trust report is established by these workflows. Record baseline distributions, choose budgets and publish the first review before claiming this mechanism exists. | `@flyingrobots`. |

Gap implementation must be split at independently working boundaries: evidence admission, resource-limited execution, replay preservation, quarantine/XFAIL operation and measured suite reporting have different observable outcomes. Do not build source-string tests that freeze a future runner's internal shape. Verify each tool through its own contract, and retain product tests separately.

## Runtime and budget admission

An execution profile names its test set, owner, machine/container class, admitted resources, per-test deadline and memory ceiling, suite p95 budget, flake budget, artifact destination and enforcing mechanism. Numeric budgets require recorded measurements on that profile and a rationale for headroom. The owner approves the profile; a change to a budget includes before/after measurements and cannot conceal a regression by relabeling size.

Small profiles must deny filesystem/network and spawned-process/thread use, not merely recommend avoiding them. Medium profiles own scratch directories, isolate parallel workers, deny egress and admit only named local dependencies. Large profiles retain schedule/fault reproducibility and run in their recorded stage. A container alone does not establish any of these properties; state its actual resource and namespace restrictions.

Failure artifacts identify the profile and exact command, build SHA, seed/schedule, enforcing limit, expected and observed behavior, logs and replay input. Performance experiments additionally record hardware, baseline, run ordering, confounders and distributions. Deadlines prevent resource exhaustion; their elapsed time is not a storage correctness oracle.

## Debt and expiry records

A quarantine record contains test identity, first failing build/artifact, owner, issue, admission date, fix-by date two to four weeks later, non-gating diagnostic command and investigated root cause. Its executor preserves the first failure and reports all attempts. Expiry blocks use of the quarantine as an integration exception until fixed or explicitly deleted with displaced risk recorded.

An XFAIL record contains the correct promised outcome, narrowly identified known failure, owner, issue, expiry and a runner that fails on an unexpected pass or an unrelated failure. Quarantine and XFAIL cannot share an undifferentiated allow-failure list. Before such a runner exists, retain the known-failure evidence and report the gap rather than treating `#[ignore]` as implementation.

A risk waiver contains the exact unmet requirement, claim and scope, consequence, evidence attempted, compensating verification, named approving maintainer, linked issue and expiry. The gap table is not a waiver, and authoring an expiry does not constitute approval. Expired records require disposition before integration; retries cannot repair a missing approval or a broken gate.

Quarterly review examines subsystem risk maps, parser targets/corpora, fault matrices and schedule bounds, calibration survivors, quarantine/XFAIL/waiver expiry, coverage archaeology, suite latency, flakes, reruns, repairs and test-deletion decisions. The owner records the review date, findings and next review date. No fabricated zero-flake rate or latency target may replace a missing baseline.
