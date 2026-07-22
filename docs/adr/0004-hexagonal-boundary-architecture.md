# ADR-0004: Hexagonal Boundary Architecture

- Status: Accepted
- Date: 2026-07-19
- Owners: Keep architecture and all core/port/adapter boundaries
- Related issue: none — introduced directly via commit `b9d3f30`, not
  through the issue-driven ADR workflow used by ADR-0001–0003

## Context

Keep's core law requires that identity, format, and durability claims be
exactly as strong as their physical evidence. That guarantee is easy to
state and easy to erode in normal Rust development: a domain type that
derives `Serialize` for convenience, a parser that lives next to the value
it decodes because that felt tidy, an error path that stringifies early —
each is individually small, and each quietly weakens what identity and
format claims can be trusted to mean once storage, layout, representation,
segments, catalog, retention, recovery, and GC all exist as real modules.

`BlobId` (ADR-0001) already needed both a canonical text codec and a
canonical binary codec. Without an explicit rule, nothing stops a codec from
being implemented as inherent methods colocated with the domain type it
encodes — which is exactly what happened before this decision was enforced
in code: `blob_id_binary.rs` and `blob_id_text.rs` lived inside the `blob`
identity module itself, and `blob/mod.rs` claimed ownership of both identity
calculation and canonical identity codecs in one module.

## Decision

Keep uses hexagonal architecture. The domain core owns storage laws,
validated domain types, state transitions, and policy-free orchestration.
Inbound ports name use cases Keep offers; outbound ports name capabilities
the core requires from its environment. Adapters implement those ports for
concrete technologies.

Dependency arrows point inward:

```text
CLI / API / foreign protocol                 filesystem / clock / randomness
              │                                           │
        inbound adapters                             outbound adapters
              │                                           │
        inbound ports ────────── domain core ─────── outbound ports
```

Core and port modules MUST NOT import adapter modules or dependency-owned
wire types. Ports speak in semantic requests, validated domain values,
staged work, and typed failures — never JSON values, CBOR values, filesystem
paths as identity, CLI argument structures, async-runtime handles, or vendor
SDK types.

Codecs are confined to ingress and egress boundary adapters. Inbound
adapters enforce bounds, decode into untrusted raw forms, validate canonical
form and cross-field invariants, then construct validated domain types
through checked admission APIs. Outbound adapters accept validated semantic
values and produce one canonical representation.

Any JSON or CBOR crossing a trust boundary, persisted, compared, signed, or
entering a hash preimage must name a canonical encoding profile in its
format specification, rationale, or ADR — never "whatever the current
serializer emits."

The full rule set is normative in `docs/Rust Standards.md` §5.3, §14.5, and
§14.6, and summarized in `AGENTS.md`'s "Hexagonal Architecture and
Determinism" section.

## Alternatives considered

- **Direct Serde-as-format**, where the durable or wire format is whatever a
  derived `Serialize`/`Deserialize` implementation happens to emit. Rejected
  because Rust struct layout and dependency serialization behavior are not a
  durable protocol (`docs/Rust Standards.md` §14.2); identity would silently
  drift across a dependency upgrade.
- **Core types depending on adapter or wire-format crates** for convenience
  (for example, a domain type implementing `serde::Serialize` directly).
  Rejected because it lets representation leak into domain types and
  couples core identity to a library's encoding defaults rather than to a
  named canonical profile.
- **Canonical order derived from `HashMap` iteration**, relying on whatever
  order a hash map happens to produce. Rejected because iteration order is
  not deterministic across processes or Rust versions, and Keep's identity,
  tests, and protocol behavior must not depend on it.
- **Hashing arbitrary serializer output** as a shortcut to a content hash.
  Rejected because it hashes an implementation detail instead of a typed,
  domain-separated canonical preimage; a serializer change would silently
  change content identity.
- **Leaving codecs colocated with domain types**, on the reasoning that a
  small crate does not yet need the separation. Rejected because it was the
  as-shipped state this ADR corrects: `blob/mod.rs` claimed ownership of
  both identity calculation and canonical codecs, and nothing distinguished
  domain law from wire format as more modules were added.

## Consequences

- Lower layers (identity, chunking, layout, representation, segments,
  catalog) must not import orchestration, CLI, or adapter code.
- Every durable JSON/CBOR surface needs an explicit canonical profile with
  golden fixtures before it can be treated as identity-bearing.
- New ports are justified only when they express a real substitution
  boundary with a concrete environmental implementation or deterministic
  test double — not for hypothetical flexibility.
- `BlobId`'s canonical binary and text codecs moved out of `blob` into a new
  `adapters` module; `blob` now owns only identity calculation. This is a
  pure internal reorganization — no public API, format, or golden-vector
  change.
