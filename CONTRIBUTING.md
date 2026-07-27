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

Documentation validation uses `markdownlint-cli2` 0.23.1, `lychee` 0.21.0,
and `actionlint` 1.7.12. Install those exact versions, then run the
repository-owned checks from the repository root:

```bash
npm ci --prefix scripts/documentation-tools --ignore-scripts --no-audit --no-fund
cargo install lychee --version 0.21.0 --locked
go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.12
export PATH="$PWD/scripts/documentation-tools/node_modules/.bin:$PATH"
python3 scripts/check_markdown.py
python3 scripts/check_workflows.py
git diff --check
git diff --cached --check
```

The checker admits tracked Markdown plus nonignored new Markdown and refuses
any other tool version. Build products, generated Rustdoc, fuzz artifacts,
and other ignored files therefore cannot change the result. Link validation
checks local files and fragments with network access disabled; external-site
availability cannot change the result. The two Git commands check unstaged
and staged whitespace errors separately.

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

Runtime fuzzing reads its exact tool versions and resource bounds from
`fuzz/campaign.env`. Load that reviewed policy, then install the pinned tools
without changing the repository's stable default:

```bash
source fuzz/campaign.env
rustup toolchain install "$FUZZ_TOOLCHAIN" --profile minimal
cargo install cargo-fuzz --version "$CARGO_FUZZ_VERSION" --locked
```

Run the same bounded smoke campaign as CI:

```bash
cargo xtask prepare-fuzz-corpus
python3 fuzz/run_campaign.py describe --profile smoke
python3 fuzz/run_campaign.py run --profile smoke
```

The deterministic seeds make every parser success path and the registered CDC
boundary transitions reachable before mutation begins. The smoke profile
remains a startup and shallow-exploration gate, not evidence of exhaustive
coverage.

For the longer scheduled profile, run:

```bash
python3 fuzz/run_campaign.py describe --profile scheduled
python3 fuzz/run_campaign.py build --profile scheduled
python3 fuzz/run_campaign.py run --profile scheduled
python3 fuzz/run_campaign.py minimize --profile scheduled
```

The scheduled profile uses the same per-input bounds with a larger,
still-finite exploration budget. Preserve any input under `fuzz/artifacts/`
that finds a defect, minimize it, and add it as a permanent deterministic
regression test.

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
