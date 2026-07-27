# Golden File Worldline Corpus v1

This directory is the implementation-independent executable corpus described
by [the Golden File Worldline specification](../../../docs/conformance/golden-file-worldline.md).

Run the independent vector checker with:

```bash
cargo xtask golden-file-worldline-check
```

The checker requires `b3sum` 1.8.5 or a compatible implementation on `PATH`.
Every identity and content-mutation digest is calculated both by the
dependency-isolated Rust oracle and by the external executable. A disagreement
is a refusal; neither result is admitted as authoritative by itself.

The version-1 `xtask` command contract is silent on success. A refusal exits
with status 1, writes nothing to standard output, and writes one
`Error: <typed diagnostic>` line to standard error. This intentionally replaces
the legacy Python checker's success confirmation so repository checks compose
without output parsing or suppression.

The `xtask` crate has no dependency on Keep. It independently implements the
corpus grammar, identity preimage, invalid-input classifications, scenario
state machine, and capability contract without importing or executing
production `BlobId` code. Its in-process digest path uses the locked `blake3`
crate, while the external `b3sum` path independently checks the algorithm
boundary.

The TSV files are protocol inputs. Do not reorder rows, normalize source files,
or regenerate expected values with production `BlobId` code.

The [version-1 rationale](rationale.md) records the durable source-path
spelling and its portability boundary.
