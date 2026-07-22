# Architecture Decision Records

This directory holds ADRs: decision records for changes that cut across
multiple subsystems, or that predate a colocated home for them. See
`docs/Rust Standards.md` §16.5 and `docs/Documentation Standards.md` §3.10
for the full rule.

An ADR is a protocol commitment, not proof that its implementation has
shipped. Executable evidence and the changelog establish implementation
maturity.

Most governed decisions do **not** belong here. A decision scoped to one
format, invariant, or architecture page belongs in that page's colocated
`rationale.md` instead. Default to a rationale note; reach for an ADR only
when the decision genuinely does not belong to one page.

## Naming

Every ADR filename MUST carry a descriptive slug after its number:

```text
0004-hexagonal-boundary-architecture.md
```

Never a bare number (`0004.md`) and never a slug with no number
(`hexagonal-boundary-architecture.md`). The number gives a stable
chronological reference; the slug lets this directory be scanned by name
alone, without opening every file to see what it's about.

## Contents

Each ADR states:

- the decision;
- the alternatives rejected, and why;
- the invariant, format, durability, recovery, GC, encryption, concurrency,
  or public-API surface it governs.

## Index

- [ADR-0001: Exact logical byte identity](0001-exact-logical-byte-identity.md)
- [ADR-0002: Separate identity from physical storage](0002-separate-identity-from-physical-storage.md)
- [ADR-0003: Deterministic content-defined chunking profiles](0003-deterministic-content-defined-chunking-profiles.md)
- [ADR-0004: Hexagonal boundary architecture](0004-hexagonal-boundary-architecture.md)
