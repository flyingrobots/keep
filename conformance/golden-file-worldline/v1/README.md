# Golden File Worldline Corpus v1

This directory is the implementation-independent executable corpus described
by [the Golden File Worldline specification](../../../docs/conformance/golden-file-worldline.md).

Run the independent vector checker with:

```bash
cargo xtask golden-file-worldline-check
```

The version-1 `xtask` command contract is silent on success. A refusal exits
with status 1, writes nothing to standard output, and writes one
`Error: <typed diagnostic>` line to standard error. This intentionally replaces
the legacy Python checker's success confirmation so repository checks compose
without output parsing or suppression.

The `xtask` crate has no dependency on Keep. It independently implements the
corpus grammar, identity preimage, invalid-input classifications, scenario
state machine, and capability contract without importing or executing
production `BlobId` code.

The TSV files are protocol inputs. Do not reorder rows, normalize source files,
or regenerate expected values with production `BlobId` code.
