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

Documentation linting uses `markdownlint-cli2` 0.19.1. Install the pinned
version, then run the repository-owned configuration from the repository root:

```bash
npm install --global markdownlint-cli2@0.19.1
python3 scripts/check_markdown.py
git diff --check
git diff --cached --check
```

The checker admits tracked Markdown plus nonignored new Markdown and refuses
any other tool version. Build products, generated Rustdoc, fuzz artifacts,
and other ignored files therefore cannot change the result. The two Git
commands check unstaged and staged whitespace errors separately.

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

Runtime fuzzing uses `cargo-fuzz` 0.13.2 with
`nightly-2026-07-24`. Install those pinned tools without changing the
repository's stable default:

```bash
rustup toolchain install nightly-2026-07-24 --profile minimal
cargo install cargo-fuzz --version 0.13.2 --locked
```

Run the same bounded smoke campaign as CI:

```bash
set -euo pipefail
python3 fuzz/prepare_corpus.py
fuzz_targets="$(cargo +nightly-2026-07-24 fuzz list)"
test -n "$fuzz_targets"
fuzz_failed=false
while IFS= read -r fuzz_target; do
  if ! cargo +nightly-2026-07-24 fuzz run "$fuzz_target" -- \
      -max_total_time=15 \
      -timeout=5 \
      -max_len=1048576 \
      -rss_limit_mb=1024 \
      -print_final_stats=1; then
    fuzz_failed=true
  fi
done <<< "$fuzz_targets"
test "$fuzz_failed" = false
```

The deterministic seeds make every parser success path and the registered CDC
boundary transitions reachable before mutation begins. The 15-second budget
remains a startup and shallow-exploration gate, not evidence of exhaustive
coverage. Preserve any input under `fuzz/artifacts/` that finds a defect and
add it as a permanent regression test.

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
