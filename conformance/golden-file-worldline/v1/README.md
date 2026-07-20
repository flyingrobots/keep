# Golden File Worldline Corpus v1

This directory is the implementation-independent executable corpus described
by [the Golden File Worldline specification](../../../docs/conformance/golden-file-worldline.md).

Run the independent vector checker with:

```bash
python3 conformance/golden-file-worldline/v1/check_vectors.py
```

The checker requires `b3sum` 1.8.5 or a compatible implementation on `PATH`.
It does not import or execute Keep's Rust implementation.

The TSV files are protocol inputs. Do not reorder rows, normalize source files,
or regenerate expected values with production `BlobId` code.
