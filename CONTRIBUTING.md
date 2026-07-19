# Contributing to Keep

Keep is foundational storage infrastructure. Contributions must preserve its
core law:

> For a given content identity, Keep must return exactly the bytes named by
> that identity—or refuse.

## Before changing code

Read:

1. [AGENTS.md](AGENTS.md);
2. the normative [Keep Rust Engineering Standard](docs/Rust%20Standards.md);
3. any architecture decision records governing the affected identity, format,
   durability, recovery, concurrency, garbage-collection, encryption, or API
   boundary.

Create an ADR before implementing a decision that changes one of those
boundaries.

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
