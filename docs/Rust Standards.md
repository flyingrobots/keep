# **KEEP RUST ENGINEERING STANDARD**

**Status:** Normative  
**Posture:** Strict by default; exceptions require written justification  
**Primary objective:** Make incorrect storage behavior difficult to express, easy to find, and impossible to merge unnoticed.

---

## **1. Governing Doctrine**

Keep is storage infrastructure.

Storage infrastructure does not get to be:

- “probably correct”;
- clever but undocumented;
- fast under the benchmark and mysterious under failure;
- tolerant of malformed internal state;
- dependent on incidental filesystem behavior;
- casually unsafe;
- difficult to search;
- difficult to review;
- difficult to delete.

The governing priorities are:

1. **Correctness**
2. **Recoverability**
3. **Auditability**
4. **Maintainability**
5. **Predictability**
6. **Performance**
7. **Convenience**

Performance matters greatly. It does not outrank correctness.

A slower implementation with explicit invariants is preferable to a faster implementation whose safety depends on tribal knowledge.

---

## **2. Mandatory Toolchain Policy**

### **2.1 Rust edition**

Keep MUST use:

```toml
edition = "2024"
```

The workspace MUST pin an explicit stable Rust toolchain:

```toml
# rust-toolchain.toml

[toolchain]
channel = "1.xx.x"
components = ["clippy", "rustfmt", "rust-src"]
profile = "minimal"
```

Do not use an unpinned `stable` channel in CI.

Toolchain upgrades MUST occur through dedicated pull requests containing:

- compiler version change;
- formatter diff;
- Clippy diff;
- dependency-resolution diff;
- benchmark comparison;
- test results;
- any newly allowed lint with justification.

Rust style editions can evolve separately from semantic editions, so Keep MUST make formatter behavior explicit rather than silently inheriting whatever happens to be installed. (⁠[Rust Docs](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-style-edition.html?utm_source=chatgpt.com))

### **2.2 MSRV**

Keep MUST declare a Minimum Supported Rust Version:

```toml
rust-version = "1.xx"
```

MSRV is a contract.

Every CI run MUST include:

```bash
cargo +${MSRV} check --workspace --all-targets --all-features
```

Raising MSRV requires:

- a dedicated PR;
- release-note entry;
- explicit rationale;
- semver review.

### **2.3 Formatting**

Formatting is not discussed in code review.

CI MUST run:

```bash
cargo fmt --all --check
```

Keep MUST use stable `rustfmt`. Do not adopt nightly-only formatter options unless the project deliberately pins nightly formatting separately.

Recommended:

```toml
# rustfmt.toml

edition = "2024"
style_edition = "2024"
max_width = 100
use_small_heuristics = "Default"
newline_style = "Unix"
```

The Rust Style Guide defines the standard style followed by `rustfmt`; Keep should deviate only where there is an overwhelming readability benefit. (⁠[Rust Docs](https://doc.rust-lang.org/cargo/commands/cargo-fmt.html?utm_source=chatgpt.com))

No manual alignment.

No decorative whitespace.

No “I prefer it this way.”

The formatter wins.

---

## **3. Compiler and Lint Policy**

### **3.1 Workspace lint inheritance**

All crates MUST inherit workspace lints.

```toml
# Cargo.toml

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "deny"
unused_must_use = "deny"
unreachable_pub = "deny"
unexpected_cfgs = "deny"
rust_2018_idioms = { level = "deny", priority = -1 }
rust_2024_compatibility = { level = "deny", priority = -1 }

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
nursery = { level = "deny", priority = -1 }

unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
exit = "deny"

indexing_slicing = "deny"
integer_division = "deny"
arithmetic_side_effects = "deny"
float_arithmetic = "deny"

cast_possible_truncation = "deny"
cast_possible_wrap = "deny"
cast_sign_loss = "deny"
cast_precision_loss = "deny"
as_conversions = "deny"

mem_forget = "deny"
large_stack_arrays = "deny"
large_types_passed_by_value = "deny"
needless_pass_by_value = "deny"
needless_collect = "deny"
implicit_clone = "deny"
clone_on_ref_ptr = "deny"

string_slice = "deny"
str_to_string = "deny"
inefficient_to_string = "deny"

wildcard_imports = "deny"
enum_glob_use = "deny"
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "deny"
missing_panics_doc = "deny"
```

Each crate:

```rust
#![deny(warnings)]
#![forbid(unsafe_code)]
#![warn(clippy::cargo)]
```

Ordinary crates strengthen the workspace's `unsafe_code = "deny"` to
`forbid`. A dedicated unsafe-boundary crate MAY carry a crate-level,
reason-bearing allowance only after satisfying §4.2.

Clippy’s `pedantic` group is explicitly aggressive and can produce false
positives; that is acceptable here. Exceptions must be local and justified
rather than weakening the workspace globally.

### **3.2 No broad lint suppression**

Forbidden:

```rust
#![allow(clippy::pedantic)]
#![allow(dead_code)]
#[allow(warnings)]
```

Any lint exception MUST:

- target exactly one lint;
- use the smallest possible scope;
- include a `reason`;
- explain why the code is clearer or safer with the exception.

Example:

```rust
#[allow(
    clippy::too_many_arguments,
    reason = "All fields are independently authenticated format components; grouping them would obscure the canonical order"
)]
fn decode_record_header(/* ... */) {
    // ...
}
```

An exception without a reason is a CI failure.

### **3.3 Clippy invocation**

CI MUST run:

```bash
cargo clippy \
  --workspace \
  --all-targets \
  --all-features \
  --locked \
  -- \
  -D warnings
```

Also run the minimal feature graph separately:

```bash
cargo clippy \
  --workspace \
  --all-targets \
  --no-default-features \
  --locked \
  -- \
  -D warnings
```

“All features pass” does not prove individual feature combinations compile.

---

## **4. Unsafe Rust**

### **4.1 Default rule**

`unsafe` is forbidden throughout Keep V1 except inside a dedicated crate
admitted under §4.2.

```rust
#![forbid(unsafe_code)]
```

Do not permit unsafe merely because storage engines often eventually use:

- memory mapping;
- direct I/O;
- custom allocation;
- platform syscalls;
- SIMD;
- uninitialized buffers.

Earn it later.

### **4.2 Unsafe admission**

If unsafe becomes demonstrably necessary, it MUST live in a dedicated crate
such as:

```text
keep-platform
keep-io-unsafe
```

That crate MUST contain:

- `#![deny(unsafe_op_in_unsafe_fn)]`;
- no unrelated business logic;
- explicit safety invariants;
- Miri-compatible tests where applicable;
- property tests around the safe wrapper;
- platform-specific integration tests;
- a written alternative analysis;
- measurements proving that safe alternatives cannot establish the required
  behavior;
- benchmarks when performance is part of the justification.

Every unsafe block MUST have a nearby `SAFETY:` explanation proving all preconditions.

“Required for performance” is not a proof.

The only current admission is
[`repository-process-spawn`](../repository-process-spawn/src/lib.rs), governed
by the
[descriptor-bound child working-directory decision](adr/0006-descriptor-bound-child-working-directory.md).
Its one hook calls only POSIX async-signal-safe `fchdir` between fork and exec.

---

## **5. Repository and Crate Structure**

Start with one main crate unless dependency boundaries justify more.

Recommended:

```text
keep/
├── Cargo.toml
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── deny.toml
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── formats/
│   ├── invariants/
│   ├── threat-model/
│   └── recovery/
├── src/
│   ├── lib.rs
│   ├── blob/
│   ├── chunk/
│   ├── layout/
│   ├── representation/
│   ├── ingest/
│   ├── read/
│   ├── segment/
│   ├── catalog/
│   ├── retention/
│   ├── recovery/
│   ├── verify/
│   ├── gc/
│   ├── ports/
│   ├── adapters/
│   └── error.rs
├── tests/
│   ├── conformance/
│   ├── crash/
│   ├── corruption/
│   ├── fixtures/
│   ├── recovery/
│   └── public_api/
├── fuzz/
├── benches/
└── tools/
```

### **5.1 Crate split rule**

A new crate is allowed only when at least one is true:

- it establishes an enforceable dependency boundary;
- it permits `unsafe` isolation;
- it supports a materially different platform;
- it avoids forcing heavy dependencies on core consumers;
- it has an independent public API and release purpose;
- it substantially improves compile times;
- it is a reusable test/conformance package.

“File count is getting large” is not sufficient.

### **5.2 Dependency direction**

Dependency flow MUST be acyclic and obvious:

```text
identity / format
        ↓
chunking / layout
        ↓
physical representation
        ↓
segments / catalog
        ↓
store orchestration
        ↓
CLI / adapters
```

Lower layers MUST NOT import:

- orchestration;
- CLI;
- logging policy;
- Echo concepts;
- filesystem policy unless that is their explicit layer;
- higher-level error enums.

Cycles disguised through traits are still cycles.

### **5.3 Hexagonal architecture**

Keep MUST use hexagonal architecture.

The domain core owns storage laws, validated domain types, state transitions,
and policy-free orchestration. Inbound ports name use cases offered by Keep.
Outbound ports name capabilities the core requires from its environment.
Adapters implement those ports for concrete technologies.

Dependency arrows point inward:

```text
CLI / API / foreign protocol                 filesystem / clock / randomness
              │                                           │
        inbound adapters                             outbound adapters
              │                                           │
        inbound ports ────────── domain core ─────── outbound ports
```

Core and port modules MUST NOT import adapter modules or dependency-owned wire
types. Ports speak in semantic requests, validated domain values, staged work,
and typed failures. They do not expose JSON values, CBOR values, filesystem
paths as identity, CLI argument structures, async-runtime handles, or vendor
SDK types.

An adapter may depend on the core and its ports. The core MUST NOT depend on an
adapter. A trait is justified here only when it expresses a real port with at
least one concrete environmental substitution or deterministic test double.

Tests SHOULD exercise the core through ports using deterministic in-memory
adapters. Integration tests separately prove that concrete adapters preserve
the port contract.

---

## **6. File Size and Findability**

These limits are intentionally severe.

### **6.1 Source file limits**

Hard CI limits:

- **Target:** 200 lines
- **Review required:** 300 lines
- **Absolute maximum:** 500 lines
- **Generated code:** exempt only in clearly marked generated directories
- **Tests:** same 500-line absolute maximum; prefer scenario subdivision

Count physical lines for deterministic enforcement. Blank and comment lines
remain part of the maintainability surface; reviewers should also examine
logical structure.

A file above 300 lines MUST begin with a decomposition issue or contain an approved exception explaining why splitting it would damage locality.

A file above 500 lines does not merge.

### **6.2 Function limits**

- Target: **20 logical lines**
- Review threshold: **40 logical lines**
- Absolute maximum: **60 logical lines**
- Cyclomatic complexity target: **≤ 5**
- Cyclomatic complexity maximum: **10**
- Cognitive complexity maximum: **12**
- Nesting depth maximum: **3**
- Parameters maximum: **5**
- Boolean parameters maximum: **0**
- Tuple return arity maximum: **3**

Exceptions are rare and local.

Parsing state machines may exceed the function limits only when:

- states are explicitly named;
- transitions are exhaustive;
- invariants remain visible;
- tests cover every transition;
- splitting would make the transition relation harder to audit.

### **6.3 Module rules**

Each module MUST have one sentence that completes:

This module owns…

If the answer contains “and” more than once, the module is probably too broad.

`mod.rs` files MUST contain:

- module declaration;
- re-exports;
- module-level documentation;
- very small coordination logic.

They MUST NOT become implementation junk drawers.

### **6.4 Forbidden file names**

Do not create:

```text
utils.rs
helpers.rs
common.rs
misc.rs
shared.rs
manager.rs
service.rs
types.rs
models.rs
stuff.rs
```

These names hide ownership.

Use names that answer what the code means:

```text
varint.rs
segment_header.rs
root_generation.rs
range_plan.rs
crash_recovery.rs
layout_validation.rs
```

`error.rs` is permitted because error ownership is obvious.

### **6.5 Findability rule**

A maintainer unfamiliar with the implementation should locate the code for a concept through filename search within two attempts.

Public nouns should map visibly to modules:

```text
BlobId                → blob/id.rs
ChunkId               → chunk/id.rs
Layout                 → layout/mod.rs
SegmentHeader          → segment/header.rs
RetentionCommit        → retention/commit.rs
VerificationReport     → verify/report.rs
```

Do not place ten unrelated public types in `types.rs`.

---

## **7. Naming Rules**

Follow ordinary Rust conventions and the Rust API Guidelines. Getter names omit `get_`; ownership-changing conversions use `into_`; borrowing views use `as_`; potentially expensive conversions use `to_`. (⁠[Rust Language](https://rust-lang.github.io/api-guidelines/naming.html?utm_source=chatgpt.com))

### **7.1 Names must carry semantic weight**

Bad:

```rust
data
info
item
thing
obj
ctx
mgr
util
handler
process
do_work
execute
run_internal
```

Better:

```rust
plaintext_chunk
catalog_generation
staged_blob
retention_namespace
segment_offset
validate_layout
publish_catalog
reconcile_preparation
```

Single-letter names are restricted to:

- trivial iterators;
- conventional coordinates in tiny mathematical expressions;
- generic type parameters;
- very short closures.

Storage code does not get `x`, `tmp`, or `buf2` for values that cross more than five lines.

### **7.2 Acronyms**

Prefer:

```rust
BlobId
GcPlan
IoError
CdcProfile
```

Not:

```rust
BlobID
GCPlan
IOError
CDCProfile
```

### **7.3 Units in names and types**

Never rely on implied units.

Bad:

```rust
timeout: u64
offset: usize
size: u32
```

Better:

```rust
timeout: Duration
offset: ByteOffset
logical_len: ByteLength
segment_id: SegmentId
```

For serialized formats, use fixed-width types and checked conversion at boundaries.

---

## **8. Type System Policy**

### **8.1 Primitive obsession is prohibited**

Do not pass naked primitives where values have distinct meaning.

Bad:

```rust
fn read(id: [u8; 32], start: u64, len: u64) -> Result<Vec<u8>, Error>;
```

Better:

```rust
fn read_range(
    blob: BlobId,
    range: ByteRange,
) -> Result<RangeReader, ReadError>;
```

Required newtypes include, at minimum:

- `BlobId`
- `ChunkId`
- `LayoutId`
- `RepresentationId`
- `SegmentId`
- `ByteOffset`
- `ByteLength`
- `RootGeneration`
- `CatalogGeneration`
- `StorageProfileId`
- `RetentionNamespace`

### **8.2 Illegal states should be unrepresentable**

Do not use:

```rust
struct Record {
    sealed: bool,
    checksum: Option<Checksum>,
}
```

Use:

```rust
enum RecordState {
    Staging(StagingRecord),
    Sealed(SealedRecord),
}
```

A sealed record should not be constructible without the evidence required to be sealed.

### **8.3 Parse, then validate, then admit**

Distinguish:

```text
Raw bytes
    ↓ parse
Syntactically decoded value
    ↓ validate
Structurally lawful value
    ↓ admit
Store-authorized operation
```

Do not deserialize untrusted bytes directly into a type whose constructor implies validity.

Prefer:

```rust
RawLayout
ValidatedLayout
```

or a private representation with a checked constructor.

### **8.4 Boolean blindness**

Public functions MUST NOT take boolean parameters.

Forbidden:

```rust
store.open(blob, true, false)?;
```

Use enums:

```rust
store.open(
    blob,
    VerificationPolicy::OnRead,
    CachePolicy::Bypass,
)?;
```

The Rust API Guidelines explicitly recommend conveying meaning through types rather than `bool` or ambiguous `Option` parameters. (⁠[Rust Language](https://rust-lang.github.io/api-guidelines/checklist.html?utm_source=chatgpt.com))

### **8.5 `Option` means absence, not failure**

Do not return `Option` when “not found” requires explanation.

```rust
Result<Option<T>, Error>
```

is acceptable only when absence is a normal, semantically unambiguous result.

Missing required content is an error.

### **8.6 Collections**

Prefer:

- slices over `&Vec<T>`;
- iterators over unnecessary collection;
- `BTreeMap` when canonical order matters;
- `HashMap` only when nondeterministic iteration cannot affect behavior;
- `SmallVec` only after profiling;
- purpose-built collections when invariants matter.

Never derive canonical serialization from `HashMap` iteration.

---

## **9. Error Philosophy**

### **9.1 No panics in library paths**

The public library MUST NOT intentionally panic for:

- malformed input;
- missing files;
- corruption;
- exhausted disk;
- overflow;
- stale generations;
- unsupported versions;
- filesystem races;
- failed invariants reachable from external state.

Forbidden outside tests:

```rust
unwrap()
expect()
panic!()
todo!()
unimplemented!()
unreachable!()
```

`unreachable!()` is still a panic. Use exhaustive state modeling.

### **9.2 Errors are domain artifacts**

Errors MUST be typed according to the failing boundary:

```rust
IngestError
LayoutDecodeError
LayoutValidationError
ReadError
RetentionError
RecoveryError
VerificationError
GcPlanError
GcExecutionError
```

Do not expose one universal `KeepError` internally.

A public facade may provide an aggregate error, but lower layers retain precise types.

### **9.3 Error variants must be actionable**

Bad:

```rust
Error::InvalidData
Error::Io
Error::Failed
```

Better:

```rust
LayoutValidationError::ChunkLengthMismatch {
    chunk: ChunkId,
    declared: ByteLength,
    observed: ByteLength,
}
```

Each error should answer:

- what failed;
- which identity was involved;
- expected state;
- observed state;
- whether retry is meaningful;
- whether data may be corrupt;
- which operation boundary produced it.

Do not include secrets or plaintext content in errors.

### **9.4 Preserve sources**

I/O and dependency errors MUST preserve their source where safe.

Never stringify an error early.

### **9.5 Error messages**

Error enum variants are machine-facing concepts.

`Display` text:

- begins lowercase;
- has no trailing period;
- does not repeat the error type;
- is concise;
- contains stable factual context.

Do not make downstream consumers parse error strings.

---

## **10. Arithmetic and Bounds**

Storage software is arithmetic software wearing a filesystem hat.

### **10.1 Checked arithmetic**

All externally influenced arithmetic MUST use checked operations:

```rust
let end = offset
    .checked_add(length)
    .ok_or(RangeError::Overflow { offset, length })?;
```

No unchecked:

- addition;
- subtraction;
- multiplication;
- shifts;
- integer casts;
- indexing.

### **10.2 Conversions**

Use:

```rust
usize::try_from(value)?
u64::try_from(value)?
```

Never:

```rust
value as usize
```

unless the conversion is proven lossless and accompanied by a tightly scoped lint exception.

### **10.3 Indexing**

Prefer:

```rust
slice.get(range)
```

over:

```rust
&slice[start..end]
```

Unchecked indexing is forbidden in production paths.

### **10.4 Explicit bounds**

Every parser and collection-building operation MUST have explicit limits:

- maximum record size;
- maximum layout entries;
- maximum hierarchy depth;
- maximum chunk length;
- maximum blob length;
- maximum varint length;
- maximum root count;
- maximum path length where relevant;
- maximum recovery scan;
- maximum allocation.

Reject before allocating.

---

## **11. Ownership, Borrowing, and Allocation**

### **11.1 Borrow by default**

Functions SHOULD accept borrowed values unless they need ownership.

Prefer:

```rust
fn verify(layout: &ValidatedLayout) -> Result<VerificationReport, VerifyError>;
```

not:

```rust
fn verify(layout: ValidatedLayout) -> Result<VerificationReport, VerifyError>;
```

### **11.2 Clone policy**

Every nontrivial `.clone()` in a hot or storage path is review-worthy.

Forbidden explanations:

- “the borrow checker was annoying”;
- “it is only a small struct”;
- “easier this way.”

Clone only when ownership semantics require duplication.

Use `Arc` only for genuine shared ownership, not to escape design decisions.

### **11.3 Allocation policy**

No hidden whole-blob allocation.

Any API that can allocate proportional to blob size MUST make that obvious in its name and documentation:

```rust
read_to_vec
materialize_blob
collect_manifest_entries
```

Streaming APIs are the default.

### **11.4 Capacity**

When a reliable bound is known, preallocate with checked conversion.

Do not preallocate from untrusted declared lengths before validating configured limits.

---

## **12. Concurrency**

### **12.1 Concurrency is a contract**

Keep V1 SHOULD support:

```text
one writer
many readers
```

unless concurrent writers are deliberately designed and proven.

The store lock behavior MUST be documented:

- process scope;
- machine scope;
- stale lock recovery;
- lock acquisition order;
- timeout behavior;
- read behavior during publication.

### **12.2 Lock discipline**

Every lock-owning type MUST document:

- protected fields;
- permissible lock ordering;
- whether I/O may occur while held;
- whether callbacks may occur while held.

Never hold a mutex across:

- blocking external I/O;
- user-provided code;
- arbitrary logging;
- another lock unless order is specified;
- lengthy hashing or compression without justification.

### **12.3 Channels**

Channels MUST be bounded.

Unbounded channels are prohibited in storage pipelines.

Backpressure is a feature.

### **12.4 Async**

The core library SHOULD remain synchronous until a real consumer proves otherwise.

Do not introduce async traits merely because reads can block.

Async wrappers may exist above a synchronous core.

---

## **13. Filesystem and Durability Rules**

### **13.1 Filesystem operations are adversarial boundaries**

Assume:

- short reads;
- short writes;
- interruption;
- stale metadata;
- permission changes;
- path races;
- full disk;
- truncated files;
- duplicate directory entries during recovery;
- files disappearing between metadata and open;
- reordered durability unless explicitly synchronized.

### **13.2 No correctness from path existence alone**

A file existing does not prove:

- completeness;
- identity;
- seal validity;
- catalog membership;
- retention;
- durability.

Verify its framing and identity.

### **13.3 Append loops**

Never assume one `write` writes everything.

Use `write_all`, or explicit loops where partial progress must be recorded.

### **13.4 Publication**

Publication of a new generation MUST follow a documented order, such as:

1. write immutable data;
2. flush user-space buffers;
3. synchronize immutable data;
4. write new catalog generation;
5. synchronize catalog;
6. atomically publish generation head;
7. synchronize containing directory where required;
8. only then retire superseded state.

The exact protocol must be platform-reviewed and crash-tested.

### **13.5 Destructors**

`Drop` MUST NOT be relied upon for fallible durability.

Destructors cannot report failure through ordinary return values; the Rust API Guidelines likewise caution that destructors should not fail. (⁠[Rust Language](https://rust-lang.github.io/api-guidelines/checklist.html?utm_source=chatgpt.com))

Use explicit:

```rust
writer.commit()?;
writer.abort()?;
```

`Drop` may perform best-effort cleanup only.

---

## **14. Serialization and Format Standards**

### **14.1 Formats are protocols**

On-disk formats MUST NOT be treated as internal implementation details.

Each format needs:

- magic bytes;
- format version;
- canonical encoding;
- explicit endianness;
- fixed limits;
- domain-separated identity;
- checksum behavior;
- unknown-version behavior;
- corruption behavior;
- migration posture;
- golden fixtures.

### **14.2 No direct Serde-as-format**

Serde may assist implementation.

It MUST NOT define the format by accident.

Do not say:

“The format is whatever `bincode` currently emits for this Rust struct.”

Rust struct layout and dependency serialization behavior are not your durable protocol.

### **14.3 Decode into raw forms**

Decoders MUST:

- reject trailing bytes unless explicitly allowed;
- reject duplicate fields;
- reject noncanonical integer encodings;
- reject unknown mandatory flags;
- reject invalid ordering;
- enforce maximum depth;
- enforce maximum lengths;
- validate all cross-field invariants.

### **14.4 Round-trip is insufficient**

These tests are insufficient alone:

```rust
assert_eq!(decode(encode(value)), value);
```

The encoder and decoder can share the same bug.

Required:

- golden encoded bytes;
- independent fixture generator where feasible;
- mutation tests;
- canonicalization tests;
- alternate implementation tests eventually.

### **14.5 Codecs are boundary adapters**

Encoding and decoding belong only at ingress and egress boundaries.

The core may own a format's semantic law, canonical schema, and validated
domain representation. Codec code that translates bytes, text, JSON, CBOR, or
dependency-owned values MUST remain in a boundary adapter. An outer public
facade may delegate to that adapter; domain constructors accept validated
semantic components and must not acquire wire-format logic.

Inbound adapters MUST:

1. enforce byte, depth, collection, and allocation bounds;
2. decode into untrusted raw forms;
3. validate canonical form and cross-field invariants;
4. construct validated domain types through checked admission APIs.

Outbound adapters MUST accept validated semantic values and produce one
canonical representation. Ports MUST NOT traffic serializer-owned value trees
or make Serde, JSON, CBOR, compression, encryption, or framing dependencies
part of the domain API.

### **14.6 Deterministic JSON and CBOR**

Determinism is a correctness requirement, not a testing convenience.

Any JSON or CBOR that crosses a trust boundary, is persisted, is compared, is
signed, enters a hash preimage, or appears in a golden fixture MUST name a
canonical encoding profile in its format specification, rationale, or ADR.
“Whatever the current serializer emits” is not a profile.

At minimum:

- JSON profiles define UTF-8, object-key ordering, number rendering, string
  escaping, Unicode treatment, duplicate-key refusal, and whitespace posture;
- CBOR profiles define deterministic map-key ordering, definite versus
  indefinite lengths, shortest integer and length encodings, floating-point
  posture, tag posture, and duplicate-key refusal;
- encoders produce exactly one byte representation for one semantic value;
- identity-bearing decoders reject noncanonical encodings rather than silently
  repairing or normalizing them;
- non-identity ingress may canonicalize only when the port contract explicitly
  permits normalization and the resulting evidence records that translation;
- maps and sets are ordered explicitly before encoding;
- golden bytes and mutation vectors prove the selected profile independently.

Never hash arbitrary JSON/CBOR serializer output. Parse, validate, canonicalize
at the adapter boundary, frame the result with its type and version, then hash
the canonical bytes.

---

## **15. Public API Standards**

The Rust API Guidelines are the minimum, not the aspiration. (⁠[Rust Language](https://rust-lang.github.io/api-guidelines/about.html?utm_source=chatgpt.com))

### **15.1 Public surface minimization**

Everything is private by default.

A symbol becomes public only when:

- a consumer needs it;
- its invariants are stable enough to support;
- it is documented;
- it is tested through the public API;
- exposing it does not leak representation accidentally.

`pub(crate)` is preferred over `pub`.

`pub use` must be deliberate.

### **15.2 Constructors**

Validated types MUST not expose public field construction.

Use:

```rust
impl ByteRange {
    pub fn new(start: ByteOffset, length: ByteLength) -> Result<Self, RangeError>;
}
```

Not:

```rust
pub struct ByteRange {
    pub start: u64,
    pub length: u64,
}
```

### **15.3 Must-use**

Types representing unfinished or consequential work MUST be `#[must_use]`:

```rust
#[must_use]
pub struct StagedBlob { /* ... */ }

#[must_use]
pub struct GcPlan { /* ... */ }

#[must_use]
pub struct RetentionCommit { /* ... */ }
```

### **15.4 Exhaustiveness**

Public enums that may grow SHOULD be `#[non_exhaustive]`.

Format enums describing frozen wire values should instead reject unknown values explicitly.

### **15.5 Generic APIs**

Do not make APIs generic merely to appear flexible.

Prefer the narrowest useful abstraction.

Bad:

```rust
fn store<T, R, E, P, C>(...)
```

Good:

```rust
fn stage<R: Read>(&self, source: R, profile: StorageProfileId)
    -> Result<StagedBlob, IngestError>;
```

### **15.6 Builders**

Use builders only when construction is genuinely complex.

Required fields should remain required.

Do not hide required invariants behind a builder that can fail mysteriously at `build()`.

---

## **16. Documentation Standards**

### **16.1 All public items documented**

`missing_docs = "deny"`.

Every public module, type, trait, function, method, field, and variant MUST explain:

- what it means;
- its invariants;
- failure behavior;
- durability implications;
- complexity where meaningful;
- whether it allocates;
- whether it blocks;
- whether it performs I/O;
- whether it verifies content;
- whether returned data is authenticated or merely read.

### **16.2 Standard documentation headings**

Public fallible functions SHOULD include:

```rust
/// # Errors
///
/// Returns ...
```

Functions that can panic MUST include:

```rust
/// # Panics
```

For Keep library APIs, the preferred content under `# Panics` is:

```text
This function does not intentionally panic.
```

Unsafe APIs, should they ever exist, require:

```rust
/// # Safety
```

### **16.3 Examples are tests**

Every important public workflow MUST have a compiling documentation example.

Rust documentation code blocks can be executed by `cargo test`, so Keep should treat examples as maintained public contract rather than decorative snippets. (⁠[Rust Docs](https://doc.rust-lang.org/rust-by-example/testing/doc_testing.html?utm_source=chatgpt.com))

### **16.4 Invariant comments**

Comments explain:

- why;
- invariants;
- nonobvious ordering;
- safety;
- crash reasoning;
- format constraints.

Comments do not narrate syntax.

Bad:

```rust
// Increment offset
offset += len;
```

Good:

```rust
// The offset is advanced only after the record checksum has been validated;
// recovery may safely treat the preceding prefix as fully admitted.
```

### **16.5 ADR rule**

Any decision affecting:

- content identity;
- on-disk format;
- durability;
- recovery;
- GC;
- encryption;
- concurrency;
- public API compatibility;
- threat model;

requires a written decision record.

A decision scoped to one format, invariant, or architecture page MUST be
recorded as that page's colocated `rationale.md`: the decision, the
alternatives rejected, and why. A reader following the concept should find
its rationale without leaving the concept's own documentation.

A decision that cuts across subsystems, or predates a colocated home for it
(such as choosing hexagonal architecture itself), MUST be recorded as an ADR
under `docs/adr/`. Every ADR filename MUST carry a descriptive slug after its
number — `0004-hexagonal-boundary-architecture.md`, never `0001.md` or
`001-foo.md` — so the directory can be scanned by name alone.

---

## **17. Testing Doctrine**

Keep tests do not ask merely:

Did the function return the expected value?

They ask:

Did the system preserve its invariants under valid input, malformed input, interruption, concurrency, corruption, migration, and resource exhaustion?

Cargo conventionally places unit tests near source and integration tests under `tests/`; Keep should follow that division while treating public integration tests as the primary behavioral contract. (⁠[Rust Docs](https://doc.rust-lang.org/cargo/guide/tests.html?utm_source=chatgpt.com))

### **17.1 Test pyramid**

Required layers:

1. **Unit tests**
2. **Public API integration tests**
3. **Golden-format tests**
4. **Property tests**
5. **Model-based tests**
6. **Mutation/adversarial tests**
7. **Crash-injection tests**
8. **Corruption tests**
9. **Concurrency tests**
10. **Fuzz tests**
11. **Benchmark regression tests**
12. **Cross-version compatibility tests**

### **17.2 Unit tests**

Every nontrivial private algorithm MUST have focused unit tests.

Tests should sit close to the implementation only when they test private behavior.

Do not embed 1,000 lines of test code under a 50-line module. Move large scenario suites to dedicated test modules or integration tests.

### **17.3 Public API tests**

At least half of behavioral tests SHOULD exercise Keep exactly as an external crate would.

This prevents the test suite from depending on internal shortcuts unavailable to users.

### **17.4 Naming**

Test names describe the law:

```rust
fn staged_blob_is_not_visible_before_commit()
fn stale_root_generation_is_rejected()
fn range_read_never_materializes_unrequested_prefix()
fn truncated_segment_tail_is_ignored_during_recovery()
fn conflicting_chunk_representation_fails_closed()
```

Forbidden:

```rust
fn test_store()
fn test_1()
fn works()
fn edge_case()
```

### **17.5 One behavior per test**

A test may perform many actions to establish one scenario.

It should prove one principal law.

When it fails, the name should tell us what contract broke.

### **17.6 Assertions**

Prefer complete semantic assertions.

Bad:

```rust
assert!(result.is_err());
```

Good:

```rust
assert_matches!(
    result,
    Err(LayoutValidationError::ChunkLengthMismatch {
        chunk,
        declared,
        observed,
    }) if chunk == expected_chunk
        && declared == expected_declared
        && observed == expected_observed
);
```

### **17.7 No nondeterministic tests**

Tests MUST NOT depend on:

- wall-clock sleeps;
- scheduler luck;
- random ports without reservation;
- ambient home-directory state;
- machine-local locale;
- filesystem iteration order;
- network services;
- test execution order;
- global mutable state.

Use injected clocks, seeded randomness, temporary directories, barriers, and deterministic schedulers.

### **17.8 Property tests**

Property tests are mandatory for:

- varint encoding;
- layout encoding;
- chunk boundary calculation;
- range planning;
- offset arithmetic;
- reconstruction;
- canonical serialization;
- generation transitions;
- compaction equivalence.

Each property-test failure MUST print or persist the minimized counterexample.

### **17.9 Model-based tests**

Maintain a simple reference model:

```text
BlobId → exact bytes
Root namespace → set of BlobId
```

Generate operation sequences:

- stage;
- retain;
- release;
- read;
- read range;
- verify;
- compact;
- recover.

After every operation, compare production state with the model.

The model should be boring enough to trust.

### **17.10 Crash testing**

Every durability boundary gets a stable crash point identifier:

```text
KEEP-CRASH-001
KEEP-CRASH-002
...
```

A crash matrix MUST kill the process:

- before write;
- during write;
- after write;
- before sync;
- after sync;
- before publication;
- after publication;
- during cleanup;
- during compaction;
- during recovery itself.

After restart, the store MUST resolve to a documented lawful state.

No test may merely assert “it opens.”

It must assert:

- visible blobs;
- retained roots;
- orphan classification;
- generation;
- segment status;
- verification result;
- recovery report.

### **17.11 Corruption tests**

Mutate every structural field:

- magic;
- version;
- flags;
- lengths;
- offsets;
- digests;
- checksums;
- record ordering;
- chunk order;
- layout depth;
- catalog references;
- root generation;
- segment seal;
- trailing bytes.

Keep MUST fail closed.

It MUST distinguish corruption from normal absence where possible.

### **17.12 Fuzzing**

Continuous fuzz targets:

- every decoder;
- every parser;
- layout validator;
- record scanner;
- range planner;
- recovery scanner;
- catalog loader;
- root loader;
- importer;
- decompression boundary if added;
- encryption envelope decoder if added.

Every fuzz-discovered defect becomes a permanent regression test.

### **17.13 Mutation testing**

Mutation testing SHOULD gate release candidates.

Targets:

- remove validation branch;
- invert comparison;
- change checked add to unchecked;
- omit sync;
- skip checksum;
- alter ordering;
- change inclusive/exclusive bound;
- drop error propagation.

If the test suite survives meaningful mutations, it is lying about coverage.

### **17.14 Coverage**

Line coverage target:

- workspace: **≥ 90%**
- identity and format modules: **≥ 95%**
- recovery and retention transitions: **≥ 95%**
- unsafe wrapper crate, if created: **100% line and branch coverage**

Coverage is evidence of execution, not proof of correctness.

No one gets to improve coverage with meaningless assertions.

### **17.15 Test profiles**

Run tests in both debug and optimized configurations:

```bash
cargo test --workspace --all-features --locked
cargo test --workspace --all-features --release --locked
```

Cargo supports selecting release or custom profiles for tests; optimized test execution is important because overflow, timing, and optimization-sensitive behavior can differ from debug runs. (⁠[Rust Docs](https://doc.rust-lang.org/cargo/commands/cargo-test.html?utm_source=chatgpt.com))

---

## **18. Benchmark Standards**

Benchmarks are not advertisements.

They are regression instruments.

Required scenarios:

- cold ingest;
- warm ingest;
- repeated near-neighbor edits;
- many tiny blobs;
- large blob;
- already compressed data;
- high deduplication;
- zero deduplication;
- random range reads;
- sequential reads;
- verification;
- recovery scan;
- root publication;
- GC planning;
- compaction;
- post-compaction reads.

Record:

- throughput;
- p50/p95/p99 latency;
- allocations;
- peak memory;
- bytes read;
- bytes written;
- write amplification;
- read amplification;
- fsync count;
- dedup ratio;
- CPU time;
- store size.

Benchmark changes above an agreed threshold require explanation.

Do not merge “faster” code that:

- weakens durability;
- disables verification;
- expands memory without disclosure;
- reduces test coverage;
- relies on warm cache only.

---

## **19. Dependency Policy**

### **19.1 Fewer dependencies**

Every dependency adds:

- API risk;
- supply-chain risk;
- compile cost;
- MSRV pressure;
- feature complexity;
- maintenance obligations.

New dependencies require written justification answering:

- Why is this needed?
- Why not standard library?
- Why not 50 lines of local code?
- What unsafe code does it contain?
- What is its MSRV?
- Is it actively maintained?
- Does it enter Keep’s public API?
- What features are enabled?
- What transitive dependencies arrive?
- What is the exit strategy?

### **19.2 Default features**

Always specify features deliberately:

```toml
some-crate = { version = "...", default-features = false, features = ["..."] }
```

Using default features requires justification.

### **19.3 Public dependencies**

Do not expose dependency-owned types in Keep’s public API unless intentionally accepting that dependency as part of Keep’s stability contract.

The Rust API Guidelines note that a stable crate cannot honestly claim stability while exposing unstable public dependencies. (⁠[Rust Language](https://rust-lang.github.io/api-guidelines/necessities.html?utm_source=chatgpt.com))

### **19.4 Auditing**

CI SHOULD include:

```bash
cargo deny check
cargo audit
```

The lockfile MUST be committed.

Dependency updates occur through isolated PRs whenever possible.

---

## **20. Feature Policy**

Features are additive.

No feature may:

- change meaning of existing API;
- change content identity;
- silently weaken verification;
- alter canonical encoding;
- change durability defaults;
- produce mutually incompatible public types.

Forbidden:

```text
fast = disables checksums
performance = skips fsync
compat = accepts malformed layouts
```

Acceptable:

```text
compression-zstd
encryption-aes-gcm
backend-memory
backend-fs
tracing
```

Feature combinations MUST be tested, not only `--all-features`.

---

## **21. Logging and Observability**

Libraries MUST NOT print to stdout or stderr.

Use structured tracing behind an optional feature.

Events MUST be:

- stable enough for diagnostics;
- free of plaintext content;
- free of encryption keys;
- free of unbounded paths where sensitive;
- explicit about blob and generation identities;
- explicit about lifecycle phase.

Example event names:

```text
keep.ingest.started
keep.segment.sealed
keep.catalog.published
keep.retention.conflict
keep.recovery.orphan_found
keep.verification.failed
```

Logs are evidence for operators, not a substitute for typed return values.

---

## **22. Pull Request Standard**

Every PR MUST be small enough to review rigorously.

Targets:

- ≤ 400 changed non-generated lines;
- ≤ 10 changed production files;
- one architectural purpose;
- no unrelated cleanup.

Larger PRs require an explicit reason and reviewer agreement.

Every PR description MUST include:

```text
Problem
Invariant affected
Approach
Alternatives rejected
Failure modes
Tests added
Benchmarks affected
Format/API compatibility
Recovery implications
Security implications
```

A storage PR without a failure-mode section is incomplete.

---

## **23. Review Standard**

Reviewers do not merely inspect whether code works.

They ask:

- What invariant does this establish?
- What malformed state can reach this branch?
- What happens after process death here?
- What survives power loss?
- Which arithmetic is externally influenced?
- What is the maximum allocation?
- Is iteration order observable?
- Could this API represent an invalid state?
- Does this error lose diagnostic context?
- Is the content identity stable?
- Is this code still correct after compaction?
- Does recovery agree with the write protocol?
- Can the behavior be tested without private access?
- Is this abstraction actually simpler than the code it replaced?

“Looks good” is not sufficient review for format, durability, identity, retention, or recovery code.

Those areas require a reviewer to restate the invariant in their own words.

---

## **24. CI Gates**

Every ordinary PR MUST pass:

```bash
cargo fmt --all --check

cargo check --workspace --all-targets --all-features --locked
cargo check --workspace --all-targets --no-default-features --locked

cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --all-targets --no-default-features --locked -- -D warnings

cargo test --workspace --all-features --locked
cargo test --workspace --all-features --release --locked
cargo test --workspace --doc --locked

cargo +${MSRV} check --workspace --all-targets --all-features --locked

cargo deny check
cargo audit
```

Also gate:

- source-file line limits;
- function complexity;
- documentation coverage;
- dependency policy;
- forbidden terms and files;
- committed generated-code cleanliness;
- public API diff;
- format fixture diff;
- test coverage threshold.

Nightly CI SHOULD additionally run:

- fuzz smoke tests;
- Miri where applicable;
- ignored slow tests;
- crash matrix subset;
- mutation-test subset;
- benchmarks;
- all supported operating systems.

Release CI MUST run the full crash, corruption, fuzz, compatibility, and benchmark suites.

---

## **25. Forbidden Patterns**

The following require rejection unless extraordinarily justified:

```rust
unwrap()
expect()
panic!()
todo!()
unimplemented!()
dbg!()
println!()
eprintln!()
process::exit()
unsafe
static mut
mem::forget()
Box::leak()
```

Also forbidden:

- public boolean parameters;
- stringly typed identities;
- raw integer offsets crossing module boundaries;
- implicit integer casts;
- unchecked indexing;
- unbounded channels;
- ambient global configuration;
- hidden filesystem access;
- hidden network access;
- hidden allocation proportional to input;
- Python source files, including `.py`, `.pyw`, and executable Python shebangs
  in regular files regardless of filename suffix;
- random IDs where content identity is required;
- wall-clock time in deterministic algorithms;
- hashing arbitrary serializer output;
- catch-all error strings;
- silent corruption repair;
- best-effort verification presented as proof;
- mutation of sealed records;
- filesystem enumeration as the normal index;
- tests that use sleep for synchronization;
- constructors that bypass validation;
- canonical behavior dependent on hash-map iteration;
- commits mixing mechanical refactoring and semantic change.

---

## **26. RUST PHILOSOPHY — OPINIONATED AND CORRECT™**

### **26.1 Rust is not Java with ownership errors**

Do not build:

```text
BlobManager
SegmentManager
CatalogManager
StorageService
RetentionService
```

Build values and transitions:

```text
BlobId
StagedBlob
SealedSegment
ValidatedLayout
CatalogGeneration
RetentionCommit
RecoveryPlan
```

Rust is strongest when nouns carry invariants and ownership models transitions.

### **26.2 Traits are not interfaces-for-everything**

Create a trait when:

- multiple implementations exist now;
- consumers genuinely need substitution;
- the abstraction is behaviorally coherent;
- object safety or generic dispatch is deliberate.

Do not create `BlobIdProviderFactoryStrategy`.

Concrete types are good.

### **26.3 Enums beat flag soup**

Use enums for closed state spaces.

Use bitflags only for truly independent combinable flags.

Do not encode state machines with six booleans.

### **26.4 Ownership is architecture**

Who owns a value is not an implementation nuisance.

It answers:

- who may mutate it;
- who controls lifetime;
- who may publish it;
- whether work is committed;
- whether cleanup is required;
- whether data can outlive the operation.

Design ownership before appeasing the borrow checker.

### **26.5 Lifetimes are not merit badges**

Do not spread explicit lifetime parameters everywhere to avoid one small allocation.

Use lifetimes where they describe genuine borrowing relationships.

Readable owned values are sometimes the correct choice.

### **26.6 Zero-copy is not automatically better**

Zero-copy can create:

- lifetime coupling;
- pinning;
- fragmentation;
- retained buffers;
- complex APIs;
- unsafe pressure;
- hidden memory growth.

Measure before worshipping it.

### **26.7 Macros require suspicion**

Use declarative macros for genuine repetitive structure.

Avoid macros that:

- invent mini-languages;
- conceal control flow;
- obscure error propagation;
- make symbols difficult to search;
- produce poor diagnostics;
- generate public APIs invisibly.

Procedural macros are dependencies and compiler plugins. Treat them accordingly.

### **26.8 Generic code is paid for in comprehension**

Monomorphization is not free.

Neither is reading five trait bounds to understand a file read.

Be generic at stable boundaries, concrete inside implementations.

### **26.9 Explicit beats magical**

Keep should have boring code:

```rust
let decoded = decode_header(bytes)?;
let validated = decoded.validate(limits)?;
let location = catalog.lookup(validated.chunk_id())?;
let record = segment.read(location)?;
record.verify(validated.chunk_id())?;
```

Not an invisible pipeline driven by global registries and blanket traits.

### **26.10 The type system proves structure, not reality**

A `SealedSegment` type can prove that the program followed a constructor.

It cannot prove the disk obeyed the constructor unless the constructor performed and witnessed the required durability protocol.

Do not confuse static validity with physical truth.

---

## **27. SENSEI WISDOM™**

### **The file-size limit is not about file size**

It is about making ownership visible.

A 900-line file often means several concepts are sharing one namespace because nobody decided where the seams are.

Splitting blindly is also bad. The standard is not “small files at any cost.”

The standard is:

One concept should be understandable without loading an unrelated concept into working memory.

### **Complexity is stored confusion**

Complexity does not disappear when code compiles.

It gets stored in:

- branches;
- state combinations;
- error cases;
- recovery behavior;
- reviewer uncertainty;
- future changes.

Keep should treat complexity like disk usage: account for it, bound it, and compact it deliberately.

### **Every hidden assumption becomes recovery code later**

Any sentence beginning with:

- “normally”;
- “the OS should”;
- “this cannot happen”;
- “the file will already exist”;
- “the catalog should agree”;

must become either:

- a proven invariant;
- a validated precondition;
- a typed error;
- a recovery branch;
- a test.

### **Corruption is not an exception to the model**

Corruption is part of the model.

The store must remain capable of answering:

- what is known;
- what is missing;
- what conflicts;
- what can be reconstructed;
- what is unsafe to trust;
- what action is permitted next.

### **Recovery is part of every write**

Do not implement a write path and later add recovery.

Every durable transition is designed simultaneously as:

```text
forward protocol
+
crash-state enumeration
+
recovery protocol
```

If you cannot state all three, the write protocol is unfinished.

### **Abstraction debt is real debt**

Every wrapper, trait, builder, generic parameter, feature, and error conversion demands ongoing interest.

Create abstraction only where it removes more reasoning than it introduces.

### **Make the good path obvious and the dangerous path loud**

A maintainer should be able to do the correct thing through the easiest API.

Dangerous operations should require:

- explicit types;
- explicit names;
- explicit policies;
- explicit evidence;
- explicit review.

### **Code quality is not prettiness**

For Keep, high-quality code means:

- invariants are local;
- behavior is searchable;
- errors preserve truth;
- allocation is bounded;
- arithmetic is checked;
- failure states are testable;
- recovery is deterministic;
- public contracts are smaller than implementations;
- physical claims are never stronger than the evidence.

That is the bar.

---

## **28. Final Law**

Keep has one unforgiving promise:

The bytes returned for an identity are exactly the bytes that identity names—or Keep refuses.

Every coding rule exists downstream of that promise.

No silent fallback.

No approximate reconstruction.

No “best effort” masquerading as success.

No convenience path around verification.

No mutation hidden behind an immutable identity.

Keep either knows, proves, and returns the bytes—or it tells the truth.
