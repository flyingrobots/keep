# Bind explicit recovery to admitted protocol directories

This decision owns directory binding at the retention recovery entry point.
It implements the existing exact-capability invariant from ADR-0009 and
addresses the directory-replacement finding on PR #99 for issue #19.

Writer admission pins the store root, retention directory and both immutable
pools. Retaining those handles avoids following a replacement directory, but
does not prove that the live protocol names still identify the admitted
directories. Recovery through an unreachable old handle can otherwise delete
stage evidence while returning success for a different visible store.

`FilesystemRetentionPublicationAuthority::recover` runs the existing
device/inode guard before observing stages or executing recovery effects.
Publication invokes this same entry point, so both paths apply one rule. A
replacement returns the existing `ProtocolDirectoryReplaced` refusal through
the recovery `Observe` boundary, preserving its source. No durable bytes,
identity encoding, synchronization order or successful recovery classification
changes.

Keeping the guard only in publication was rejected because explicit recovery
is a public production path. Reopening replacement directories was rejected
because they did not supply the authority's admission proof. The check is
synchronous capability-relative I/O with bounded extra directory opens and no
record-body allocation.

Three permanent laws rename and recreate retention, roots and manifests in
turn, then require the exact refusal and unchanged retained root-stage bytes.
The laws fail on head `c9277eadbcb22a5ba1330760a3b908178957d4e5` at the
intended refusal assertion. They use filesystem unit fixtures; they establish
post-admission binding rather than production platform admission or physical
power-loss durability. Other recovery findings remain independently open.
