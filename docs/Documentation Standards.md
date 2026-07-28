# **KEEP DOCUMENTATION STANDARD**

**Status:** Normative for new and substantially changed documentation.
**Applies to:** Contributor, architecture, format, invariant, recovery,
threat-model, and release documentation in Keep.
**Normative terms:** **MUST**, **SHOULD**, and **MAY** indicate requirement
strength.

This standard adapts a reader-task documentation model to Keep's existing
correctness-and-recovery discipline. It does not require a mass rewrite of
existing pages. Apply it when creating documentation, changing behavior, or
touching a page enough that leaving it below this bar would create new debt.

---

## **1. Purpose**

Documentation is part of Keep's contract, alongside its types and its tests.
A Keep page should help a specific reader do one of these jobs:

- integrate Keep through a guided first success, such as the Golden File
  Worldline;
- complete a real task against Keep's public API in their own environment;
- look up exact facts while integrating — a type, a format field, an error
  variant, a crash-point identifier;
- understand a concept, boundary, or design decision — why a port exists, why
  a format is versioned, why unsafe is forbidden;
- troubleshoot an observable failure — a refused open, a failed
  verification, an orphaned segment;
- change the implementation safely and verify the result.

A page MUST have one primary job. Do not force a README, an invariants page,
or a release note to behave as a tutorial, reference manual, roadmap, and
architecture guide at the same time.

---

## **2. Corpus map**

Keep keeps its durable truth in a small set of known places.

| Location | Job |
| --- | --- |
| `README.md` | Public front door: what Keep is, its core law, its design boundary, current status, and links to deeper docs. |
| `AGENTS.md` | Normative engineering contract for contributors and agents: governing doctrine, hexagonal boundaries, and hard rules. |
| `docs/Rust Standards.md` | The full normative Rust Engineering Standard that `AGENTS.md` summarizes. |
| `docs/Documentation Standards.md` | This standard. |
| `docs/architecture/` | Current architecture: hexagonal boundaries, dependency direction, and core/port/adapter ownership, by subsystem. |
| `docs/invariants/` | Living statement of what Keep currently guarantees: identity, format, durability, and recovery invariants. |
| `docs/formats/` | Durable format specifications: magic bytes, versioning, canonical encoding, bounds, and golden fixtures. |
| `docs/threat-model/` | What Keep defends against, and what it explicitly does not. |
| `docs/recovery/` | The crash-point catalog, the publication protocol, and the documented lawful state recovery must reach. |
| `docs/adr/` | Slugged ADRs for decisions that cut across subsystems or predate a colocated home. See §3.10. |
| `docs/<category>/<concept>/rationale.md` | Colocated decision record for a decision scoped to one concept: the decision, alternatives rejected, and why. See §3.10. |
| `CONTRIBUTING.md` | Contributor-facing operational contract: what to read, required local checks, and PR expectations. |
| `CHANGELOG.md` | Release-visible historical ledger. |
| `SECURITY.md` | Vulnerability reporting and support posture. |
| generated rustdoc (`cargo doc`) | Generated public API reference: exact types, functions, invariants, errors, panics, and examples. |

Keep splits decision records in two, by scope. A decision scoped to one
format, invariant, or architecture page lives as that page's colocated
`rationale.md` — a reader following the concept should find why it works
that way without leaving the concept's own documentation. A decision that
cuts across subsystems, or predates a colocated home for it, goes in
`docs/adr/` instead, and MUST carry a descriptive slug after its number
(`0004-hexagonal-boundary-architecture.md`, never `0001.md` or `001-foo.md`)
so the directory can be scanned by name alone rather than opened file by
file. See §3.10.

Add a how-to guide or troubleshooting page under the relevant category above
when a reader need is not well served by an architecture, invariant, or
format page.

Recommended additions as Keep grows a public API and a release process:

```text
docs/
  how-to/
  troubleshooting/
docs/RELEASING.md
```

Do not create empty placeholder directories ahead of a real reader job.
`docs/architecture/`, `docs/invariants/`, `docs/formats/`,
`docs/threat-model/`, and `docs/recovery/` are recommended locations named
in `docs/Rust Standards.md` §5 and the corpus map above — create each one
when it has a real page to hold, the same as any other location in this
standard, not preemptively. `docs/adr/` already holds real content
(ADR-0001 through ADR-0004); it is not an exception to this rule, it
simply has pages in it already.

---

## **3. Page types**

### **3.1 Domain reference**

A domain reference describes current behavior for a durable Keep concept,
such as chunking, segment layout, catalog generations, or the recovery
protocol. It lives under `docs/architecture/`, `docs/invariants/`, or
`docs/formats/`, whichever facet it documents.

A domain reference MUST:

- describe only behavior that exists on `main`;
- state public contracts, invariants, and supported usage;
- link to the tests, fixtures, or crash-point identifiers that prove it;
- distinguish current behavior from known gaps;
- avoid roadmap promises except as explicitly labeled limitations.

It MUST NOT become the only integrator-facing guide for a task that needs
step-by-step help; that belongs in a how-to guide.

Repository operations such as dependency updates, toolchain upgrades, and
releasing belong in a workflow reference (§3.8), not a domain reference.

### **3.2 Requirement ledger**

A requirement ledger is the contract ledger for a governed invariant or
format. It MUST identify:

- stable requirement or crash-point identifiers (for example
  `KEEP-CRASH-004`, matching the identifiers required by
  `docs/Rust Standards.md` §17.10);
- planned or implemented cases;
- the exact invariant or lawful state under test;
- the oracle — a reference model, golden fixture, or corruption-matrix entry;
- the evidence type, from the test pyramid in `docs/Rust Standards.md` §17.1;
- the status;
- the concrete test, fixture, or crash-injection harness once implemented.

Planned work is not evidence. A gap MUST be marked as a gap and tied to an
issue or a rationale note (§3.10) when it matters.

### **3.3 Tutorial**

A tutorial is a guided learning path. Use it for a controlled first success
against Keep's public API, such as the Golden File Worldline described in
`README.md`.

A tutorial MUST:

- state prerequisites and starting state, including the crate version and
  feature flags in use;
- use a known-good path;
- provide actions in tested order;
- show expected intermediate and final results, ideally as doctest output;
- end with what the reader learned and where to go next.

### **3.4 How-to guide**

A how-to guide helps a competent reader complete a real task against Keep's
public API, such as "ingest a large file with bounded memory" or "recover a
store after an interrupted write."

A how-to guide MUST:

- be titled as a goal, preferably starting with a verb;
- state the expected result;
- identify blocking prerequisites;
- give the shortest safe route to the result;
- include exact types, function calls, or configuration, not paraphrased
  ones;
- explain how to verify success;
- link to reference or explanation instead of reproducing it.

Keep's core library exposes no CLI (`docs/Rust Standards.md` §5.2). A
how-to guide for an adapter or tool built on Keep belongs with that
adapter, not with the core library's documentation.

### **3.5 Reference**

Reference pages support exact lookup. Add or generate them for:

- the public API surface — types, functions, invariants, and errors;
- durable format fields — magic bytes, version, encoding, bounds;
- the typed error catalog, by boundary (`docs/Rust Standards.md` §9.2);
- the crash-point catalog (`docs/Rust Standards.md` §17.10).

Reference MUST state exact names, syntax, fields, defaults, constraints,
compatibility behavior, output, errors, and examples. The public API
reference SHOULD be generated by `cargo doc` and coverage-checked by
`missing_docs = "deny"` rather than maintained by hand.

### **3.6 Explanation**

Explanation develops a mental model: why unsafe is forbidden in v1, why ports
exchange only validated domain types, why formats are versioned protocols,
why recovery is designed alongside every write.

Explanation SHOULD describe mechanisms, relationships, tradeoffs,
alternatives, and limits. It MUST NOT become an unstructured code tour.

### **3.7 Troubleshooting**

Troubleshooting starts with a symptom an integrator or operator can observe,
such as:

- `Store::open` refuses to open a store;
- verification reports a mismatch for a blob believed to be intact;
- recovery leaves a segment classified as orphaned;
- an ingest is rejected for exceeding a configured bound.

A troubleshooting page MUST list discriminating checks first — which error
variant, which crash point, which invariant — map signals to likely causes,
give concrete recovery actions that reference `docs/recovery/`, and show how
to verify the fix, typically by re-running verification.

### **3.8 Workflow reference**

A workflow reference describes a recurring contributor or maintainer
operation, such as a dependency update, an MSRV or toolchain upgrade
(`docs/Rust Standards.md` §2.1–§2.2), or a release.

A workflow reference MUST:

- describe only the current operational contract;
- identify the authoritative runbook when one exists;
- link to the CI checks or evidence that verify it;
- avoid duplicating domain references, release notes, or roadmap promises.

### **3.9 Contributor guide**

Contributor docs explain how to change Keep's implementation safely.
`CONTRIBUTING.md`, `AGENTS.md`, and `docs/Rust Standards.md` fill this role.
They SHOULD explain the system model — hexagonal boundaries, dependency
direction, the testing doctrine — before listing files. Source links support
an explanation; they do not replace one.

### **3.10 Decision records: rationale note and ADR**

Keep records governed decisions (`docs/Rust Standards.md` §16.5) in one of
two places, chosen by scope, never by default habit.

A **rationale note** is the default. It is scoped to one concept and lives
as `rationale.md` inside the same directory as the architecture, invariant,
or format page it explains — for example
`docs/formats/segment_header/rationale.md` next to that format's `README.md`.
Use it when a reader following the concept should find why it works that way
without leaving the concept's own documentation.

An **ADR** is the exception, reserved for a decision that cuts across
multiple subsystems or predates a colocated home for it — such as choosing
hexagonal architecture itself. ADRs live under `docs/adr/`. Do not default to
an ADR merely because a decision feels important; a decision that already
belongs to one format, invariant, or architecture page belongs in that
page's rationale note instead. Reserving `docs/adr/` for genuinely
cross-cutting decisions is what keeps it small enough to read end to end,
rather than becoming a lookup-by-number pile a reader has to open file by
file to understand.

Both forms MUST:

- name the decision;
- state the alternatives rejected, and why;
- name the invariant, format, durability, recovery, GC, encryption,
  concurrency, or public-API surface it governs.

An ADR filename MUST also carry a descriptive slug after its number —
`0004-hexagonal-boundary-architecture.md`, never `0001.md` or `001-foo.md` —
so `docs/adr/` can be scanned by name alone.

A rationale note or ADR MAY be short. Neither MAY be skipped for a decision
that `docs/Rust Standards.md` §16.5 requires recording.

---

## **4. Maintenance loop**

For a meaningful behavior change:

1. Add or update a decision record (§3.10) when the change affects identity,
   format, durability, recovery, GC, encryption, concurrency, public API
   compatibility, or the threat model (`docs/Rust Standards.md` §16.5): the
   affected concept's `rationale.md` if it is scoped to one page, or a
   slugged ADR under `docs/adr/` if it cuts across subsystems.
2. Update the relevant `docs/invariants/` or `docs/formats/` page, and its
   requirement ledger, before implementation. Tests are the spec: a failing
   test or fixture should exist before the behavior does.
3. Add the smallest deterministic executable evidence — unit, property,
   golden-format, model-based, crash-injection, corruption, or fuzz, per the
   test pyramid — that fails for the missing behavior.
4. Implement the behavior.
5. Update the living `docs/architecture/`, `docs/invariants/`, or
   `docs/formats/` page in the same change once the implementation exists.
   Describe the resulting post-merge contract, not branch mechanics or an
   unimplemented promise.
6. Mark the requirement or crash-point identifiers implemented and record the
   actual evidence — the test file, fixture, or crash harness.
7. Update `README.md`, `CONTRIBUTING.md`, and `CHANGELOG.md` when the public
   surface, documentation routing, or project status changes.

Small fixes may scale this down, but they still need a clear claim, evidence
when behavior changes, and honest current truth.

### **4.1 Pre-publication documentation upkeep**

Before pushing a branch or opening a pull request for a meaningful change:

1. Inspect the complete branch diff for changes to public APIs, formats,
   invariants, durability, recovery, security, dependencies, workflows, and
   operator-visible errors.
2. Reconcile every affected living page, requirement ledger, rationale or ADR,
   `README.md`, `CONTRIBUTING.md`, and `CHANGELOG.md`. Touch only pages whose
   reader job or authoritative claim actually changed.
3. Confirm that commands and examples name the exact implemented target, use
   supported behavior, and state observable verification.
4. Confirm that planned or deferred work is labeled and linked to active
   ownership rather than presented as current behavior.
5. Run the documentation, doctest, and whitespace gates required by §8.
6. Push or open the pull request only after the documentation review and its
   deterministic gates are clean.

This upkeep is a delivery gate, not permission to mix unrelated documentation
reorganization into an otherwise narrow change.

---

## **5. Examples and executable truth**

Examples are part of the contract. Every important public workflow MUST have
a compiling documentation example (`docs/Rust Standards.md` §16.3).

User-facing examples MUST:

- be syntactically valid Rust, or valid TOML/shell for configuration and
  tooling;
- use supported behavior;
- include enough context to run or interpret them;
- use least-privileged and safe defaults;
- identify destructive or privileged operations clearly;
- show an observable result when one exists.

### **5.1 Runnable, illustrative, and abridged examples**

A runnable example uses supported behavior and includes required context.
Rust examples SHOULD be doctests, executed by `cargo test --workspace --doc`.

An illustrative example may omit setup or nonessential detail, but it MUST be
labeled as illustrative and MUST NOT be presented as directly runnable. Use a
`rust,ignore` code fence rather than a plain `rust` fence that silently omits
required context.

An abridged example may shorten large input or output, but it MUST identify
the omitted material and preserve the behavior relevant to the explanation.

### **5.2 Code blocks and terminal examples**

Every fenced block SHOULD declare its language:

- `rust` for compiling examples and doctests;
- `bash` or `sh` for copyable local-check commands;
- `toml` for `Cargo.toml`, `clippy.toml`, `rustfmt.toml`, and `deny.toml`
  fragments;
- `text` for expected output or error text;
- `console` only for a transcript that deliberately includes prompts and
  output.

Do not include `$` prompts in a block intended for copy and paste. Present
commands and their output separately, matching the existing style in
`CONTRIBUTING.md`.

### **5.3 Placeholders**

Use clearly fictional and context-safe values.

| Context | Preferred placeholder |
| --- | --- |
| Example blob content | `b"example content"` or `sample_blob` |
| Configuration value | `StorageProfileId::new("example-profile")` |
| Filesystem path | `/example/store`, never a real local path |
| Formal syntax notation | `<blob_id>` |
| Secret or credential | an explicitly fake value, such as `EXAMPLE_KEY_DO_NOT_USE` |

### **5.4 Dangerous operations**

For destructive, privileged, costly, or irreversible operations — such as GC
execution, compaction, or a forced recovery override:

1. Place the warning before the operation.
2. State the exact consequence and scope.
3. Provide a dry-run or safer alternative when available.
4. State required permissions or lock ownership.
5. Include rollback guidance when applicable.
6. Explain how to verify the result, typically via verification or a
   recovery report.

---

## **6. Diagrams and accessibility**

Keep's core library has no CLI, LSP, or editor surface
(`docs/Rust Standards.md` §5.2), so this standard scopes visual material to
diagrams rather than screenshots or terminal captures: dependency-direction
diagrams, hexagonal-boundary diagrams, on-disk byte-layout diagrams, and
publication or crash-state diagrams. Reinstate screenshot- and
recording-specific guidance if Keep ever ships an adapter with an
interactive surface.

Use a diagram when it answers a reader question that prose alone would
bury in detail, such as the dependency-direction diagrams already used in
`docs/Rust Standards.md` §5.2 and §5.3. Do not add a diagram as
ornamentation.

Every nontrivial diagram MUST:

- answer a stated or obvious reader question;
- have meaningful labels or adjacent explanatory prose, never a bare image or
  block with no surrounding text;
- distinguish conceptual simplification from exact implementation when
  needed;
- omit or redact real paths, keys, or other sensitive operational detail.

ASCII diagrams, as already used throughout `AGENTS.md`, are the default: they
are inherently text-equivalent and render identically everywhere Keep's
Markdown is read. Prefer them over image-based diagrams unless an image
communicates something ASCII genuinely cannot.

Diagrams MUST NOT rely on color, position, or shape alone to communicate
essential meaning that a plain-text reader would miss.

---

## **7. Writing, style, and terminology**

Write like a competent teammate: direct, precise, and approachable.

- Use `you` for actions the reader or integrator performs.
- Use `Keep` for actions the library performs, and `the store` for actions a
  running instance performs.
- Use imperative verbs for procedures.
- Prefer active voice when it clarifies who is responsible.
- Use passive voice when the actor is unknown, irrelevant, or less important
  than the result.
- Use present tense for current behavior.
- Avoid hype, marketing claims, vague reassurance, unnecessary apology, and
  excessive exclamation.
- Avoid `we` unless referring to an explicit project decision or policy.

Prefer:

> Call `Store::read_range`. It returns `ReadError::Overflow` when the
> requested range exceeds the blob's declared length.

Avoid:

> You might run into trouble if the range you pass isn't quite right.

### **7.1 Sentences, paragraphs, and lists**

Write for comprehension, not for a readability score.

- Put the result, decision, warning, or essential condition first.
- Give each sentence one main job.
- Keep sentences short enough to understand in one pass, but do not enforce
  a universal word limit.
- Keep each paragraph focused on one coherent idea.
- Use numbered lists for ordered procedures.
- Use bullets for parallel options, requirements, or checks.
- Use prose when relationships, causality, or tradeoffs matter — as
  `docs/Rust Standards.md` §27 does for its design rationale.

Sentence length, paragraph length, passive voice, jargon density, and bullet
count are editorial signals. They MUST NOT become universal merge gates.

### **7.2 Markdown and typography**

Use formatting to communicate type, not to manufacture emphasis.

- Use bold sparingly for warnings or genuine emphasis, not for every concept
  on first mention.
- Use inline code for types, functions, error variants, crash-point
  identifiers, configuration keys, file paths, and literal values —
  `BlobId`, `ReadError::Overflow`, `KEEP-CRASH-004`.
- Use exact casing for public types, functions, and error variants.
- Use descriptive link text that states what the destination provides. Do
  not use `here`, `this link`, or a bare filename as the entire link label.

Use tables for genuinely two-dimensional lookup, comparison, or structured
facts, as this standard and `docs/Rust Standards.md` §8.1 both do. Do not use
tables for long narrative passages or multi-step procedures.

### **7.3 Terminology**

Use one canonical term for each concept. Define unfamiliar terms at first
use. Keep's canonical vocabulary is set by `docs/Rust Standards.md` §7–§8 and
MUST NOT be renamed casually in documentation:

- `BlobId`, `ChunkId`, `SegmentId`, `RootGeneration`, and `CatalogGeneration`
  are identity newtypes, not raw integers or strings.
- `ValidatedLayout`, `StagedBlob`, `SealedSegment`, and `RetentionCommit` are
  domain values with checked constructors, not primitive bags of fields.
- core, ports, and adapters are separate hexagonal layers; do not use them
  interchangeably.
- error types are named by boundary — `IngestError`, `ReadError`,
  `RecoveryError`, and so on — not collapsed into one universal `KeepError`
  in prose.

Shared vocabulary belongs in domain references or a future
`docs/glossary.md`. A glossary is a lookup aid, not a prerequisite for
understanding a page.

### **7.4 Inclusive and accessible language**

Use literal, neutral language that describes the technical condition
directly.

- Use gender-neutral language when a person's gender is irrelevant.
- Avoid identity-based or stigmatizing metaphors.
- Prefer terms such as unavailable, corrupted, orphaned, stale, or refused
  when those are the actual conditions Keep reports.
- Avoid culturally specific idioms when they make instructions harder to
  understand or translate.

### **7.5 Notes, cautions, and warnings**

Use callouts consistently:

- Note — useful context that does not affect correctness or durability.
- Important — information required to complete the task correctly.
- Caution — an action may cause an undesirable or costly result.
- Warning — an action may cause data loss, a durability regression, or an
  irreversible change.

Do not use a warning merely to make ordinary text look important.

---

## **8. Checks and enforcement**

Documentation quality requires both deterministic checks and human judgment.

Run for documentation changes:

```bash
cargo xtask documentation-integrity-check
git diff --check
git diff --cached --check
```

Use `markdownlint-cli2` 0.23.2. The repository-owned configuration records
deliberate rule choices. The Rust checker selects tracked Markdown plus
nonignored new Markdown, disables configuration globs for that invocation,
and refuses a different tool version. It also runs `lychee` 0.21.0 offline
with fragment checking, so external-site availability cannot affect the
result. Run it from the repository root. The two Git commands inspect
unstaged and staged whitespace errors separately.

The same Rust command checks workflows with `actionlint` 1.7.12 and refuses
another version. It also verifies the committed Node lock graph, Dependabot
manifest coverage, and the documentation job's delegation to this command.
The dedicated `documentation` job in `.github/workflows/ci.yml` installs the
pinned tools, runs malformed-input refusal laws and the repository-owned
command, and verifies repository whitespace before admitting the result as CI
evidence.

CI SHOULD block on facts it can determine reliably:

- malformed Markdown;
- broken internal links and anchors;
- failed doctests declared runnable (`cargo test --workspace --doc
  --locked`);
- stale generated reference — public API documentation that no longer
  matches `cargo doc` output;
- undocumented public items, already enforced today by `missing_docs =
  "deny"` in `Cargo.toml`;
- invalid diagrams in changed pages;
- changed contract behavior without an updated invariant, format, rationale,
  or ADR page;
- references to files, types, tests, fixtures, or crash-point identifiers
  that do not exist;
- destructive-operation examples without a preceding warning marker;
- copied examples containing real paths, keys, or credentials.

The following SHOULD normally be advisory:

- page length;
- source-line length;
- sentence length;
- paragraph length;
- passive voice;
- jargon density;
- number of bullets;
- suspected missing diagrams;
- tone and template-like phrasing;
- overuse of bold;
- table complexity;
- external-link health.

These signals are useful for editors. They are poor universal merge gates.

---

## **9. Review checklist**

Before calling a documentation change done, check:

- The page has one primary reader job.
- Living references under `docs/architecture/`, `docs/invariants/`, and
  `docs/formats/` describe the resulting current behavior of the change, not
  unimplemented roadmap intent.
- Planned work lives in a requirement ledger, a rationale note, an issue, or
  a PR — not silently implied by a reference page.
- Examples use supported behavior and show observable results, ideally as
  passing doctests.
- Public types, functions, errors, and crash-point identifiers have or link
  to reference coverage.
- A governed decision (identity, format, durability, recovery, GC,
  encryption, concurrency, public API, threat model) has a rationale note
  colocated with the concept it affects, or a slugged ADR under `docs/adr/`
  if it genuinely cuts across subsystems.
- Release-visible changes update `CHANGELOG.md`.
- Markdown and diff checks pass.

The objective is not a perfectly uniform library. The objective is a
documentation corpus where readers, reviewers, tests, and agents can find the
right authoritative page at the moment they need it.
