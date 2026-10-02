# Retention reader catalog refusal boundary

Status: accepted.

The public fenced reader must distinguish catalog restart admission failures from failures to collect consistent head coordinates.

Previously the filesystem source wrapped `CatalogRestartError` in `io::Error`, the collector wrapped it again, and the reader returned `FilesystemRetentionSnapshotError::View`; the documented `Catalog` variant was unreachable.

The filesystem source now carries the catalog admission result as its collected view value, retaining the concrete restart error without converting it into I/O.

After consistent collection, the public loader maps that admission error directly into `FilesystemRetentionSnapshotError::Catalog`, while coordinate and retention I/O errors remain `View` failures.

A catalog refusal observed between moving heads is discarded with that attempt, just like a successful speculative view; the existing bounded collector retries, and only a stable pair selects the admission result.

This uses the existing view-value abstraction instead of adding a breaking public error parameter to the port or storing a separate mutable error sentinel on the filesystem source.

No format, identity preimage, writer protocol, or public signature changes; catalog refusal paths perform the collector's second coordinate read before reporting an admission error, and successful paths retain their existing allocation and I/O behavior.

The public error regressions cover absent selected catalogs, absent selected segments, corrupt selected catalog bytes, and a corrupt coordinate HEAD; a controlled real-filesystem schedule additionally restores a missing catalog and publishes retention between the load and second coordinate read.
