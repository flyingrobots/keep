---
Title: Keep Testing Standards
Status: Accepted
Binding: true
Created: 2026-10-01
Updated: 2026-10-01
Scope: every automated assertion in the repository
Source: user-supplied Testing Standards dated 2026-08-16
Portable: core rules yes; enforcement profile is Keep-specific
---

A test is a controlled experiment about a stated promise. Its failure must identify the broken promise. A suite can lie through false confidence, false alarms, or decay; every rule below names the failure it prevents. Keep's primary promise is exact bytes for a content identity, or an explicit refusal.

Do not count tests as evidence of correctness. Account for claims, counterexamples, and blind spots. Requirements in this document are binding; their implementation and present enforcement status are recorded in the [enforcement profile](testing/enforcement.md). Publishing a requirement does not establish that existing tests or CI enforce it.

## Evidence subjects

Every test identifies its subject: product behavior, tool behavior, oracle calibration, or a static/API restriction. Product verification asserts outcomes of production Keep code at its contract boundary. Tool verification asserts a real contract of a runner, validator, benchmark reporter, or other repository tool. Oracle calibration demonstrates that an assertion or checker detects a known violation. Static/API checks establish a format constant, compiler restriction, or declared repository policy.

All four subjects obey this standard. Tool behavior and calibration can be valuable when they protect an explicit claim, but cannot discharge a product acceptance criterion. A runner's case count, fixture registry, or source-text occurrence is not evidence that Keep survived those faults. Conversely, a test that drives a production operation through a real fault-injecting port and checks its result is product verification.

Compiler restrictions and static policy checks must use an appropriate oracle, such as compilation refusal or an executable policy validator. Searching source for a spelling is not proof that the corresponding runtime behavior occurred. A count is legitimate only when the count itself is contractual, such as a bounded allocation or exactly-once external effect, and its observation is independent of the implementation being checked.

## 1. Test the narrowest contract boundary

A contract exists when a consumer outside the owning implementation depends on a promise. Choose the owning domain API, semantic port, adapter boundary, CLI, wire format, or durable artifact; neither Rust visibility nor the outermost system boundary determines the choice. Do not expose an accessor only to freeze an implementation detail. Promote a complex internal to an independently meaningful module when its behavior needs its own contract, not merely to accommodate a test.

**Prevents:** Refactoring deadlock, private-helper pseudo-contracts, and unnecessarily expensive end-to-end tests.

## 2. Assert observable outcomes

Assert returned bytes, identities, exact typed failures, persisted state, emitted messages, released authority, or absence of forbidden effects. Prefer semantic projections; assert ordering, complete object equality, identifiers and timestamps only where contractual. Harvest universal properties over the entire output, reject unknown shapes, and report a nonzero witness count. Enumerate required existentials explicitly so missing output cannot satisfy a universal assertion vacuously.

Use real implementations when deterministic and affordable. Owned fakes require a shared conformance suite against the real implementation; document any residual drift risk. Interaction assertions are justified only when the observed interaction crosses a contract boundary and is itself promised behavior. Storage synchronization order can be such a promise; an internal helper-call sequence usually is not. Calling a recording double directly and checking its log tests the double, not Keep's protocol execution.

**Prevents:** Change detectors that miss bugs, missing effects behind successful status codes, vacuous universals, and silently divergent fakes.

## 3. Organize tests around behaviors and declare change kind

Each coherent change declares one of four kinds: pure refactoring, new feature, bug fix, or deliberate behavior change. A mixed PR identifies each coherent part and why it belongs together. Pure refactoring preserves existing behavioral expectations; a feature adds its new promises; a bug fix adds a regression; deliberate behavior change identifies the old and new contract when editing an expectation. Corrections to test defects or deletion of non-contract tests require their own stated reason and evidence, not an invented product behavior change.

Prefer one atomic promise per test, with several assertions when necessary to establish it. Names describe laws. A long scenario, several independent actors, multiple act phases, or “and” in a name triggers review for separate promises; none is automatically a defect. A short protocol exchange can legitimately require multiple steps.

**Prevents:** Function-shaped suites, expectation changes hidden in refactors, and scenarios whose failures cannot identify the broken promise.

## 4. Demonstrate that load-bearing assertions can fail

Every new or materially changed load-bearing assertion has recorded falsification evidence: a witnessed regression RED or a targeted mutation that changes the promised outcome. The named assertion must execute and fail for the intended reason. Compilation failure, setup failure, an unrelated assertion, or a zero-execution run is not calibration. Use isolated build artifacts or invalidate affected caches when switching source or applying mutations.

Use continuous diff-scoped mutation where risk justifies it; bound the campaign and triage survivors as oracle gaps, missing tests, equivalent mutations, optimization-only changes, or explicit blind spots. Do not force an incidental assertion to kill an equivalent mutant. Never gate on a mutation percentage.

**Prevents:** Tautologies, skipped assertions, misleading RED receipts, stale-cache evidence, and mutation-score theater.

## 5. Use generated evidence for quantified claims

“Nothing changed,” “these paths agree,” “always,” and “never” require generated differential, property, or metamorphic evidence, accompanied by readable specified examples. Describe input classes so a large corpus cannot conceal mostly empty cases. Keep a fixed-seed reproducible lane and a lane accumulating new seeds. The runner records the seed outside the test process before launch, so a crash cannot erase it.

Shrinking or an explicit deterministic reducer is required for generated failures. Minimize, deduplicate, retain and replay counterexamples permanently. State differential-oracle limits: shared misinterpretation can make both implementations agree incorrectly, and an old implementation can preserve a known bug. Re-scope or retire equivalence checks when behavior intentionally changes.

**Prevents:** Sparse examples presented as universal evidence, lost reproducers, irreducible failures, correlated-oracle confidence, and bug-for-bug lock-in.

## 6. Name every oracle

Record the source of expectations: (1) a specified requirement, (2) independently derived vectors or a reference, (3) an invariant or relation, (4) a pseudo/differential reference, or (5) change detection of previously captured output. Non-obvious tests carry a one-line oracle note. Prefer a small independent model over copying production logic; never derive expected and actual through the same production normalization helper.

Class-5 snapshots must be labeled, minimized and reviewed under Rule 17. A format golden derived independently from the specification can be class 1 or 2; being a file does not make it class 5. A count of harness entries is not an oracle for storage correctness.

**Prevents:** Oracle laundering, implementation-derived expectations, and confusing “different” with “wrong.”

## 7. Construct determinism at the seams

Control and record every source of nondeterminism actually observed: time, randomness, scheduling, environment and identity. Use fixed clocks and seeded generators where consulted, explicit scheduling where concurrency matters, and harness-owned locale, timezone, paths and identifiers. Never use a bare sleep for synchronization. Runner deadlines may use monotonic time to enforce a ceiling; domain assertions must not depend on elapsed host time unless a controlled performance experiment is their subject.

Existing uncontrolled call sites require a reviewed, monotonically shrinking inventory with an owner and a ratchet; do not claim hermeticity while such a dependency remains. Add seams for actual dependencies, not every imaginable source of nondeterminism.

**Prevents:** Non-reproducible failures, rerun culture, ambient-state dependence, and unnecessary abstractions.

## 8. Make tests hermetic

Each test owns its scratch state and mutable fixtures. Block ambient network access; use loopback and port zero only for an admitted medium-size contract. No ambient home directory, host configuration, shared mutable fixture, test-order assumption or uncontrolled filesystem ordering may affect the verdict. Sharing immutable conformance data or isolated infrastructure is permitted; sharing mutable scenes is not.

Verify changed tests alone, under reordered execution and concurrently when their admitted size allows it. Periodic minimal-environment runs exercise isolation assumptions. Declaring a sandbox is insufficient: record the mechanism that restricts observations and the limits it does not enforce.

**Prevents:** Order-dependent results, accidental external integration, worker interference, and laptop-only green runs.

## 9. Declare test sizes and enforce budgets

Size describes resources, not scope. Small uses one process with no filesystem, database, network, spawned threads or sleeps. Medium is confined to one machine and admitted loopback dependencies. Large exceeds those limits. Every test has a size, wall-time ceiling, memory ceiling and admitted resources; every suite has a latency budget and an owner. Choose the smallest honest size and justify duplicate verification at different sizes.

The runner must enforce deadlines and resource isolation, and report violations as failures. Record p95 suite latency to prevent relabeling slow tests from hiding decay. Keep small-test ceilings in milliseconds where practical; choose measured budgets in the execution profile rather than copying unrelated machines' numbers. Missing measurement or enforcement is an explicit gap, not a compliant size label.

**Prevents:** Unclassified slow suites, E2E accretion, fictional resource ceilings, and tests nobody runs before integration.

## 10. Preserve the first failure and manage flakes

A flaky verdict is a bug in the test or product until diagnosed. Repeated runs classify failures; they never replace the original failure with green. Remove the smallest untrusted test from the gate the same day, while continuing visible diagnostic execution. A quarantine needs a named owner, issue, root-cause investigation, date and fix-by deadline of two to four weeks. Expiry requires a fix or a justified deletion, not indefinite renewal.

Measure a declared flake budget and retain attempt counts and first-failure artifacts. An unowned or expired quarantine cannot authorize integration. A suspected production race is not resolved by labeling its detector flaky.

**Prevents:** Retry-to-green, real regressions hidden in noise, and a permanent quarantine graveyard.

## 11. Use coverage for inspection

Coverage exposes unexercised changes, supports subsystem archaeology and detects unexpected changes in execution on otherwise unchanged source. It does not demonstrate an oracle or a correct outcome. No project coverage percentage is a target or merge gate. Any component-specific branch-completeness decision must document costs and defensive-code/fuzzing tradeoffs; it cannot substitute for behavioral evidence.

**Prevents:** Assertion-free coverage gains, averages hiding critical gaps, and rejecting useful black-box verification.

## 12. Observe bug regressions RED on unfixed code

A bug fix includes a correct contract-level regression observed failing on the unfixed revision, followed by GREEN on the fix. Record exact source SHAs, commands, feature/profile settings and the named assertion's failure. Use a test commit followed by a fix commit, or a reproducible parent attestation. Include the reported instance and generated or boundary evidence for its class where appropriate.

For genuinely irreproducible failures, record attempted reproduction, add instrumentation that would identify recurrence, expand relevant exploration and retain an owned suite-deficiency issue. A written, approved, expiring risk decision records the remaining uncertainty; an obvious patch or inconvenient reproduction is not an exemption.

**Prevents:** Counterfactual RED claims, mechanism-pinned regressions, and rediscovered bugs without permanent evidence.

## 13. Fuzz parsers and property-test transformations

Every trust-boundary parser, decoder, codec and format reader needs a live fuzz target, bounded inputs, sanitizer/assertion-enabled exploration and a managed seed corpus. Targets build in CI and corpus regressions replay in CI. Property-test transformations for identity, canonical encoding, bounds, fixpoints and rejection invariants. A round trip alone does not prove the Keep format specification.

Strengthen crash-only oracles with typed admission/refusal checks, independent implementations and semantic properties. Crash-level findings block release. Minimized historical counterexamples remain permanent regression assets; retain them without uncontrolled corpus growth.

**Prevents:** Parser robustness gaps, silent invalid admission, dormant fuzz infrastructure, and lost corruption regressions.

## 14. Explore concurrency schedules

State safety and liveness separately. Use controllable execution, seeded scheduling, bounded systematic interleaving exploration, or deterministic simulation/history checking at the highest rung the architecture supports. Record the schedule or seed and exploration bounds. Liveness requires a stated progress condition and controlled divergence detection; stress loops, wall-time sleeps and a lucky absence of hangs are not correctness evidence.

Race detectors complement these checks; silence does not establish linearizability, deadlock freedom or eventual progress. State-space limits are recorded blind spots, not claims of exhaustive exploration.

**Prevents:** Unreproducible races, safety-only confidence, and stress testing presented as consistency verification.

## 15. Inject faults into durability and recovery promises

Maintain a fault matrix crossing fault type with protocol phase. Inject seeded or deterministically enumerated and replayable process death, failed I/O, full disk, torn writes, reordered unsynced writes, cancellation and other claimed faults. Log the schedule before launch. Include faults during recovery from another fault. Check exact restored bytes, acknowledged state, typed refusal and preserved evidence after each interruption through the production boundary.

The ability to enumerate a fault point is not evidence of injecting it. Process-kill recovery does not establish power-loss behavior; report which storage persistence model was exercised. Numeric recovery-point or recovery-time promises require their own appropriate oracle and controlled experiment.

**Prevents:** Unexecuted recovery paths, harness-count confidence, crash-during-recovery gaps, and unsupported power-loss claims.

## 16. Treat performance as an experiment

State the hypothesis, control or randomize confounders and execution order, and report p50/p90/p99/max rather than a single mean. Use a same-run baseline and declared tolerances on a recorded hardware class; uncontrolled shared-host absolute thresholds cannot gate. Address coordinated omission for queued latency. Calibrate the measurement infrastructure against known behavior and a deliberately worsened build that it must distinguish.

Keep records throughput, latency, peak memory, allocations, I/O bytes, amplification, sync count and deduplication where relevant. Performance evidence cannot weaken identity verification or recovery. Budget scheduled experiments separately; label coarse smoke trends honestly.

**Prevents:** Noisy benchmark decisions, omitted worst latency, undetected accumulated regressions, and measurement infrastructure nobody calibrated.

## 17. Govern golden changes and known failures

Minimize artifacts to their actual contract. Canonicalize incidental variation without removing identity-bearing bytes, meaningful ordering or durable coordinates. Re-baselines are reviewed behavior changes or explicitly justified oracle corrections, never an unread bulk blessing. Refactors preserve expected output. Specification-derived durable-format vectors retain their independent provenance and are checked against production behavior.

An XFAIL encodes correct behavior with an owner, issue and expiry. Its runner requires the known failure and rejects an unexpected pass until the test is promoted to a normal gate. XFAIL is distinct from quarantine: its verdict is trusted and inverted; quarantine's verdict is untrusted. Ignoring or commenting out a test implements neither policy.

**Prevents:** Unreviewed snapshot approval, machine-dependent goldens, silently changed formats, and forgotten known failures.

## 18. Review and curate test code

Favor obvious, focused tests and actionable failures that name expected and observed state and replay parameters. Straight-line arrange/act/assert is the default, not a syntactic law. Loops, conditionals, parameterization, shared builders and “and” in a name trigger review for obscured promises or unexecuted assertions; they are permitted when they make the contract and failure clearer. Generated tests, model checks and fault campaigns naturally need controlled iteration. Prefer clarity over abstraction or forced duplication.

Record deletion criteria: the protected behavior disappeared; a demonstrably stronger, cheaper test subsumes the risk; quarantine expired with a justified removal; an uninterpretable class-5 artifact has no defensible claim; or calibration fails and no protected contract can be identified. The deletion commit names the criterion and where any remaining risk is checked or explicitly unresolved. Deleting a regression merely to make CI green is not a criterion.

Measure suite latency, flakes, reruns, test churn during refactors, quarantine age and repair time. Review those trust indicators quarterly with risk maps, parser coverage, fault matrices and accepted blind spots; neither counts nor percentages replace that review.

**Prevents:** Test-code cleverness, syntactic theater, silent loss of meaningful evidence, and uncurated suite decay.

## 19. Gate on trustworthy evidence

Hermetic suites, assertion calibration, RED-on-parent bug evidence, contract verification, sanitizer/crash findings, resource violations and XFAIL symmetry may gate. Coverage percentages, mutation scores, naturally noisy performance signals and retry-to-green may not. Every required target has an owner; failures preserve build hash, command, environment, seed/schedule, logs, diffs and minimized reproducers.

Validate dependency-aware selection with periodic full suites. Releases run the relevant full platform and fault matrix, not only a presubmit subset. A broken mainline outranks feature work. Waivers require a named approving maintainer, reason, bounded scope, tracking issue and expiry; a gap ledger is not approval. Product diagnostic surfaces and replay hooks must make detected faults explainable.

**Prevents:** Untrusted integration gates, unowned failures, hidden selection gaps, and detected faults without useful diagnostics.

## Choosing evidence and reviewing it

| Claim | Suitable evidence and oracle |
| --- | --- |
| Exact bytes, identity or refusal | Specified public/API or adapter-boundary examples with exact output and typed failure. |
| A format's canonical bytes | Independently specified golden vectors plus production admission and corruption tests. |
| Refactor or path equivalence | Generated differential checks plus specified examples and stated reference limitations. |
| Invariant over transformed input | Property/metamorphic checks with classified inputs, reduction and retained counterexamples. |
| Cross-boundary semantics | Conformance tests shared by real adapters and any owned doubles. |
| Parser robustness | Live bounded fuzzing, sanitizer failures and strengthened semantic admission oracles. |
| Concurrent consistency or progress | Recorded schedule exploration checked against explicit safety and liveness models. |
| Durable publication or recovery | Injected faults checked against exact persisted/reopened state and a stated persistence model. |
| Performance change | Calibrated distributions against a controlled same-run baseline. |
| Tool safety or oracle detection | A tool-boundary outcome or deliberately invalid input, explicitly labeled as tool/calibration evidence. |

Review in this order: did each load-bearing assertion demonstrably fail; is its oracle independent and named; is a bug regression RED on the unfixed SHA; does the change declaration match the expectation diff; does the test cross the correct contract and observe its outcome? Then inspect witness counts and required output, fake conformance, isolation, size/ceiling enforcement, seed retention and reduction, schedules and stacked faults, golden diffs, waivers and deletion criteria. A missing answer requires revision or an explicit approved risk decision.

Each subsystem owns a short risk map: critical claims and consequences, boundaries and owner, oracle, fault model, CI stage and enforced budget, and accepted blind spots. Review it quarterly. A risk map allocates evidence; it does not duplicate the suite as a prose enumeration of cases. Choose the portfolio by blast radius, not a fixed pyramid ratio; storage can justify substantial deterministic simulation and recovery verification.

The contested choices remain explicit: mocks may help design but permanent interaction checks require contractual effects; coverage floors can reward vacuity and are not Keep gates; general test-first sequencing is optional but observed falsification is mandatory and bug diagnosis is RED on unfixed code; lightweight risk maps can decay and therefore need review; XFAIL inversion can obscure debt and therefore requires ownership, expiry and unexpected-pass enforcement. None of these tradeoffs authorizes unsupported confidence.
