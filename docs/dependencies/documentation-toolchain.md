# Documentation Validation Toolchain

## Purpose

Keep uses three development-only tools to enforce deterministic documentation
and GitHub Actions facts:

- `markdownlint-cli2` 0.23.2 validates Markdown structure;
- `lychee` 0.21.0 validates local links and fragments with network access
  disabled;
- `actionlint` 1.7.12 validates GitHub Actions syntax and expressions.

These tools execute only in contributor environments and the unprivileged,
read-only `documentation` CI job. They do not enter the Rust dependency graph,
production binaries, public APIs, durable formats, or content identity.

## Admission boundary

The CI job pins Node.js 24.18.0 and installs exact tool releases. The committed
`scripts/documentation-tools/package-lock.json` pins every Markdownlint
transitive archive and Subresource Integrity digest. The installer uses
`npm ci` with lifecycle scripts disabled and refuses lockfile drift.
`markdownlint-cli2` 0.23.2 directly admits the patched `js-yaml` 5.2.2
release.

`scripts/install_documentation_tools.sh` verifies the native release archives
before extraction:

| Tool archive | SHA-256 |
| --- | --- |
| `actionlint_1.7.12_linux_amd64.tar.gz` | `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8` |
| `lychee-x86_64-unknown-linux-gnu.tar.gz` | `a06547250f10021dcafc6ed5bb20fca75835b65711745b63cfdda34c29ff6a73` |

The Rust `cargo xtask documentation-integrity-check` boundary verifies the
structure and exact BLAKE3 digest of the committed Node lock artifact and the
exact BLAKE3 digest of the reviewed installer. The byte-exact lock admission
refuses altered, omitted, or additional package records. The boundary parses
the CI workflow as YAML and admits only reviewed, unguarded `run` fields from
the documentation job, then verifies each executable's reported version
before admitting its output as evidence. A missing tool, changed archive,
unexpected version, empty input corpus, guarded or unreviewed command, or tool
failure refuses the check.

## Determinism and network posture

The Rust task derives the Markdown and workflow inputs from Git's tracked and
repository-nonignored source paths. Generated Rustdoc, build outputs, ignored
fuzz artifacts, ignored vendor trees, and user-global ignore policy cannot
enter either input set. Missing tracked paths are treated as pending
deletions; Git-trackable nonregular paths such as symlinks and tracked paths
replaced by FIFOs are refused. Non-trackable special files cannot enter the
Git-selected corpus.

Each Git path inventory runs in a dedicated process group under a two-minute
deadline that covers the child and both output readers. Standard output retains
at most the 16 MiB path-stream bound, and diagnostics retain at most 64 KiB. A
timeout, terminal signal, reader failure, or exceeded bound terminates the
whole group and reaps the child before the task refuses.

Documentation corpus tests construct their Git fixtures through the same
bounded process authority. Each fixture command runs in a dedicated process
group with a two-minute deadline and null output streams, so a stalled command
or descendant cannot outlive the test boundary. The helper clears the inherited
environment, then admits only the ambient executable search path, the `C`
locale, and explicit null system and global Git configuration. Ambient Git
directory, worktree, index, object, and configuration variables cannot redirect
a fixture.

Git inventory and each validation tool start through one retained repository
directory handle. Child-only setup changes directory through that handle after
fork and before exec; the parent working directory does not change. Replacing
the configured repository path, running checks against a substitute, and
restoring the original path cannot redirect either corpus selection or
validation.

Each selected source also retains its device, inode, size, modification time,
and change time. The Rust boundary reopens every path through the retained
repository capability and compares that identity before and after each
external tool. A source replacement, in-place mutation, or
substitute-then-restore sequence refuses the corpus instead of admitting tool
output from ambiguous bytes.

The workflow checker disables `actionlint`'s optional `shellcheck` and
`pyflakes` integrations. Neither auxiliary executable is admitted or pinned by
this toolchain, so ambient PATH contents cannot expand the validation boundary.

Lychee runs with `--offline` and `--include-fragments`. It checks local
destinations and anchors while excluding external network requests. External
website availability, DNS, redirects, rate limits, and certificates therefore
cannot decide whether a pull request passes.

Tool installation requires HTTPS access to the integrity-locked npm and pinned
GitHub release artifacts. Runtime validation performs no authenticated or
mutating network operation.

## Alternatives rejected

- Mutable action tags or unversioned package installs do not provide a stable
  reviewed tool boundary.
- Online external-link validation makes required pull-request CI depend on
  systems outside Keep's authority.
- Scanning every filesystem Markdown path admits ignored and generated state.
- Reimplementing Markdown, GitHub anchor, and workflow parsers locally would
  add a larger and less reviewed parser surface.

## Failure and recovery

The documentation job has read-only repository permissions and disables
checkout credential persistence. It writes tools only beneath
`RUNNER_TEMP`. A failed or interrupted installation leaves no authoritative
state and requires no recovery; a subsequent job starts from a fresh runner.
The Rust workflow contract requires the exact top-level `contents: read`
permission mapping and refuses `write-all`, write authority, additional scopes,
or omitted permissions before admitting any documentation step.

## Review triggers

Repeat this admission review when any of these changes:

- Node.js, `markdownlint-cli2`, `lychee`, or `actionlint` version;
- archive URL, checksum, or npm lock graph;
- npm dependency override;
- Markdown or workflow input boundary;
- `actionlint` auxiliary linter policy;
- link checking gains network access;
- job permissions or credential handling;
- a tool crosses into production code, a public API, or a durable format.
