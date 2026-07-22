# Problem

<!-- What concrete problem does this change solve? -->

# Invariant affected

<!-- Which Keep law or invariant does this establish, preserve, or change? -->

# Approach

<!-- Describe the implementation and its ownership boundary. -->

# Alternatives rejected

<!-- What other approaches were considered, and why were they rejected? -->

# Failure modes

<!-- Include malformed state, interruption, corruption, and resource failure. -->

# Tests added

<!-- Name the laws and failure modes covered by tests. -->

# Benchmark impact

<!-- Include measurements, state no expected impact, or explain why unmeasured. -->

# Format and API compatibility

<!-- Describe durable-format, identity, migration, and public-API consequences. -->

# Recovery implications

<!-- State the forward protocol, possible crash states, and recovery behavior. -->

# Security implications

<!-- Discuss confidentiality, integrity, availability, and dependency changes. -->

# Checklist

- [ ] I read `AGENTS.md` and the Keep Rust Engineering Standard.
- [ ] I recorded any decision affecting a governed boundary — as that
      concept's colocated `rationale.md`, or as a slugged ADR under
      `docs/adr/` if it cuts across subsystems.
- [ ] I added tests appropriate to the actual failure modes.
- [ ] I ran the relevant formatting, linting, testing, and policy checks.
- [ ] I did not mix unrelated refactoring with the semantic change.
