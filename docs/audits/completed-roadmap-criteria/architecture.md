# Architecture decision and structural-enforcement audit

This page owns T-10.1 and T-10.2 from the originally checked roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 496–513. Inspected main is
`f49cff732cf7a6e1b472decba9e4c4130990559e`. T-10.3's broader durable-port/fake
claim remains pending and is not counted as an audited verdict here.

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

Twenty-nine remaining checked tasks now have verdicts. Thirty-five other
checked tasks and nineteen reopened entries still need full accounting.
No checkbox changed, and neither issue #131 nor its tracking parent is closed.
