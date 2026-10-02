# Retention reader root identity

Status: Accepted

Joint admission of `FORMAT`, `migration.intent`, and `migration.receipt` establishes that those records agree with each other. It does not establish that the directory receiving the read request is the directory the migration intent names. Reader admission must compare the opened root's physical identity before accepting the version-two read capability.

The reader uses the same root identity probe and `require_root_identity` predicate as writer admission. Restart-stable device and inode coordinates must match; mount-instance identity is not compared, preserving the established remount rule. A mismatch remains a concrete `FilesystemPlatformAdmissionError::RootIdentityChanged` source inside the reader admission error, with the coordinate and both values intact. No record, identity preimage, or format bytes change.

The regression transplants a complete mutually consistent migration-record set from one controlled migrated directory into another on the same filesystem. Reading the recipient must refuse with the donor inode as expected and the recipient inode as observed. Ordinary migrated-root snapshot laws establish the positive path. This experiment does not exercise an actual device move or remount, and does not establish catalog-path pinning, every read interleaving, or platform admission beyond the existing identity probe.
