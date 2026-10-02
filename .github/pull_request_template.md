# Pull request

## Problem

<!-- What concrete problem does this change solve? -->

## Invariant affected

<!-- Which Keep law or invariant does this establish, preserve, or change? -->

## Approach

<!-- Describe the implementation and its ownership boundary. -->

## Change kind

<!-- Pure refactoring / new feature / bug fix / deliberate behavior change.
For coherent mixed changes, declare each part. Explain any correction to a
test oracle or removal of a non-contract test separately. -->

## Alternatives rejected

<!-- What other approaches were considered, and why were they rejected? -->

## Failure modes

<!-- Include malformed state, interruption, corruption, and resource failure. -->

## Tests added

<!-- Name the laws and failure modes covered by tests. -->

## Testing evidence

<!-- Follow docs/Testing Standards.md and docs/testing/enforcement.md.
For each changed claim: name its owner, product/tool/calibration/static
subject, contract boundary, law and independent oracle. Tool/harness checks
cannot discharge Keep storage acceptance criteria.
Link recorded assertion calibration; for bug fixes include unfixed/fixed
SHAs, exact commands, named RED failure and GREEN receipts.
State sizes, resource ceilings, suite budgets and actual enforcing
mechanisms; distinguish proposed limits from enforced ones.
For generated, concurrency or fault evidence: outside-process seed/schedule,
replay command, reduction/corpus, fault-matrix cells and persistence model.
State blind spots and any approved waiver owner/issue/expiry.
For changed expectations/deletions: explain the contract/oracle correction
or deletion criterion and where remaining risk is checked.
Documentation-only changes name their document outcome and validation;
mark irrelevant fields not applicable with a reason. -->

## Benchmark impact

<!-- Include measurements, state no expected impact, or explain why unmeasured. -->

## Format and API compatibility

<!-- Describe durable-format, identity, migration, and public-API consequences. -->

## Recovery implications

<!-- State the forward protocol, possible crash states, and recovery behavior. -->

## Security implications

<!-- Discuss confidentiality, integrity, availability, and dependency changes. -->

## Checklist

- [ ] I read `AGENTS.md`, the Keep Rust Engineering Standard, and the Keep Testing Standards.
- [ ] I recorded any decision affecting a governed boundary — as that
      concept's colocated `rationale.md`, or as a slugged ADR under
      `docs/adr/` if it cuts across subsystems.
- [ ] I added tests appropriate to the actual failure modes.
- [ ] I declared change kind, evidence subjects and independent oracles, and linked relevant calibration/RED receipts.
- [ ] I reported actual resource enforcement and blind spots; I did not substitute tooling counts or retry-to-green for product evidence.
- [ ] I ran the relevant formatting, linting, testing, and policy checks.
- [ ] I did not mix unrelated refactoring with the semantic change.
