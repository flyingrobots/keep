# Contributing to Keep

Keep is foundational storage infrastructure. Contributions must preserve its
core law:

> For a given content identity, Keep must return exactly the bytes named by
> that identity—or refuse.

## Before changing code

Read:

1. [AGENTS.md](AGENTS.md);
2. the normative [Keep Rust Engineering Standard](docs/Rust%20Standards.md);
3. any ADR under [docs/adr/](docs/adr/), or colocated `rationale.md`,
   governing the affected identity, format, durability, recovery,
   concurrency, garbage-collection, encryption, or API boundary.

Record your decision before implementing a change to one of those
boundaries: as that concept's colocated `rationale.md` when the decision is
scoped to one format, invariant, or architecture page, or as a slugged ADR
under `docs/adr/` when it cuts across subsystems. State the decision, the
alternatives rejected, and why.

## Before changing documentation

Read the [Keep Documentation Standard](docs/Documentation%20Standards.md)
before creating documentation or substantially changing an existing page. It
does not require rewriting pages that are merely below the bar; apply it when
a change would otherwise add new documentation debt.

## Development checks

The minimum local checks are:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo check --workspace --all-targets --no-default-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo test --workspace --all-features --release --locked
cargo test --workspace --doc --locked
cargo deny check
cargo audit
```

Every meaningful change also needs tests appropriate to its actual failure
modes. Round-trip tests alone are not sufficient for durable formats.

## Pull requests

Keep pull requests should be small and single-purpose. Every pull request must
describe:

- the problem;
- the invariant affected;
- the chosen approach;
- alternatives rejected;
- failure modes;
- tests added;
- benchmark impact;
- format and API compatibility;
- recovery implications;
- security implications.

Do not mix semantic changes with unrelated refactoring.
