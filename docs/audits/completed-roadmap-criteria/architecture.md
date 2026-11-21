# Architecture decision and structural-enforcement audit

This page owns T-10.1 and T-10.2 from the originally checked roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 496–513. Inspected main is
`f49cff732cf7a6e1b472decba9e4c4130990559e`. T-10.3's durable-port/fake inventory is evaluated below.

## T-10.1 — Decide the architecture

**Acceptance and delivery: met on main.** ADR-0004 is accepted and defines
inward dependency flow, semantic ports, codecs at boundaries, canonical
JSON/CBOR profiles and justified substitution boundaries. It records the
codec relocation, alternatives, invariants and compatibility consequences.
The task is the architecture decision; this verdict does not prove universal
implementation conformity or substitute for T-10.3's review.

## T-10.2 — Enforce structurally

**Acceptance/definition of done: not met.** The roadmap explicitly names
module size, forbidden filenames, no Python and denied unreachable public
visibility. Size and Python admission exist in `xtask/src/source_structure/`;
`Cargo.toml` denies `unreachable_pub`. The collector/classifier has no admission
for the nine Rust filenames prohibited by AGENTS.md.

A copy-isolated Docker main-equivalent clone with its own build directory
received one new inventoried `src/utils.rs` containing a module-ownership
comment. `cargo xtask source-structure-check` exited successfully. This is
observed acceptance of a prohibited name, not a compile error or zero-test
result. The temporary file and its index entry were removed afterward; the
clone returned to a clean state. No host worktree was mutated by the probe.

Correction owner: [issue #144](https://github.com/flyingrobots/keep/issues/144).
Its coherent scope is the existing literal basename prohibitions, with exact
refusals and preservation of current source/path/Python/size checks. It does
not introduce a new dependency-analysis requirement or substring naming ban.

## T-10.3 — Durable protocol ports and fault-injecting fakes

**Named acceptance and delivery: met on main.** The implemented durable write
protocols expose storage capabilities and deterministic fakes. The inventory
below checks the port and failure laws rather than relying on filenames alone.

| Protocol | Port | Executed failure evidence |
| --- | --- | --- |
| Immutable segment writing | `SegmentStage` | Scripted short/interrupted/zero/overreported writes and synchronization refusals; no receipt after failed durability |
| Store initialization | `StoreInitializationStorage` | Failure at each of six initialization phases; exact phase/source and attempted prefix |
| Catalog generation publication | `CatalogPublicationStorage` | Recording storage with exact phase failures and stopping later writes |
| Recovery stage discard | `RecoveryStageDiscardStorage` | Exact expected-state and directory-sync refusals; retained stage on refusal |
| Recovery stage completion | `RecoveryStageCompletionStorage` | Stage/pool/staging synchronization failures, pool conflict and operation-prefix assertions |
| Recovery next-head finalization | `RecoveryNextHeadFinalizationStorage` | Verification, candidate sync, replacement and root-sync failure laws |
| Recovery segment resume | `RecoverySegmentResumeStorage` | Injected storage failure returns no resumable stage; stale fingerprint refuses |
| Retention publication | `RetentionPublicationStorage` | All 17 publication phase failures preserve exact attempted prefix; authority failure precedes mutation |
| Store migration | `StoreMigrationStorage` | All 21 phase failures preserve exact attempted prefix; current-state verification failure precedes mutation |

The immutable segment port is exercised by the scripted `stage_double`;
catalog, recovery, retention and migration suites carry their recording or
in-memory storage doubles. These are substitution boundaries with observable
failures, not placeholder traits. Inspection of existing domain directories
found no adapter/filesystem/network/Serde imports, but this targeted review is
not an exhaustive architectural analysis of every boundary module.

Copy-isolated Docker with pinned Rust 1.96.0 and the dedicated main-equivalent
source build directory ran all eleven listed public integration targets: 72
laws passed in debug and 72 in release, with zero filtered or ignored tests.
The source clone has unchanged main Rust/corpus content; no new production
implementation or native test was introduced for this evidence.

This verdict establishes ports and fakes for implemented protocols. It does
not assert that passing mocks proves filesystem durability, that partial
migration recovery is delivered, or that retention production admission is
complete; those require their own later-task and correction-owner evidence.
T-10.2 remains unmerged on main, with its correction in PR #145.

Thirty remaining checked tasks now have verdicts. Thirty-four other checked
tasks and nineteen reopened entries still need full accounting. No checkbox
changed, and neither issue #131 nor its tracking parent is closed.
