# Writer acquisition identity evidence

Change kind: test-oracle correction with a private deterministic scheduling seam for [#169](https://github.com/flyingrobots/keep/issues/169), discovered while auditing T-13.1 under #131. Main `6051abb25a9fd33ae7ee0de5614514b709a4d82a` already propagates identity refusal correctly. This is not an ordinary runtime bug fix: unmutated parent production is not claimed to violate the contract.

## Promise and boundary

`KEEP-RECOVERY-004` requires the authority-producing acquisition path to reject an opened lock file whose canonical pathname now names a different device/inode after kernel acquisition. The old helper test did not exercise that authority path. The first replacement test exercised acquisition but replaced the entry before locking; it detected an ignored refusal yet survived moving verification before locking. Neither is credited as sufficient evidence for the final ordering claim.

The final law opens and retains the real root lock and original lock file, invokes the shared acquisition boundary, and replaces the canonical pathname at a private synchronous checkpoint after the kernel file lock and before identity verification. The ordinary and initialization paths invoke that same body with a no-op checkpoint. The checkpoint runs under root-then-file lock ordering, exposes no public callback API and acquires no additional locks. It exists to control the observed transition without sleeps, stress or global hooks.

The test requires no guard to escape, exact `VerifyFileIdentity/InvalidData`, and unchanged bytes in both files. Existing public acquisition laws separately cover ordinary success, exclusion, missing-file refusal and no-follow behavior. This test enters the shared module boundary after outer pathname opening; it does not claim to explore every public-entry-point or raw namespace race.

## Reproducible calibration

[Checked-in receipts and replay commands](writer-acquisition-identity/README.md) provide the pinned parent, toolchain, features, profiles, exact production mutation patches, actual RED output and restored GREEN output. The resulting final candidate SHA and required hosted checks are recorded on PR #170 because a document cannot contain its own commit hash.

Ignoring identity refusal survives the old helper/public-lock/initialization tests. Moving verification before locking survives the initial stronger test. Both fail the final after-lock law with `replacement received writer authority`. Separate wrong-phase and destructive-original/replacement mutations fail the precise diagnostic and persisted-byte assertions. The earlier removed-call mutant hit dead-code lint and is excluded from runtime evidence. Source timestamps are invalidated after restoration to avoid stale binaries.

Deletion criterion: the helper-only test is subsumed by a stronger test of the actual authority-producing transition. No accepted behavior or refusal expectation is relaxed; production lock ordering, identity checks and guard construction remain intact around the private checkpoint.

## Execution and limits

The medium test uses one owned filesystem sandbox and real kernel locks in copied Linux Docker source with a dedicated build target. Initial library/mutation runs used overlay; public integration fixtures used ext4. The first broader all-feature attempt correctly refused overlay in unrelated production-platform laws and remains preserved as a setup failure, not a regression RED. Corrected broad validation and final calibration bind both library and integration scratch roots to ext4 without bypassing platform admission.

There is no new network dependency, random schedule, persistent format, benchmark, allocator experiment or physical power-loss claim. Per-test resource enforcement gaps remain those in the binding testing enforcement ledger. The replay page names both the focused proof and its limits; earlier green CI is never substituted for checks on a changed head.
