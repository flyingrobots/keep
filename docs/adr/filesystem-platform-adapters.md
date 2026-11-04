# Filesystem Platform Adapters

- Status: Accepted; implemented for Linux ext4 and macOS APFS
- Date: 2026-11-02
- Owners: Keep storage implementation and repository owner
- Related issue: none; directly requested platform design and implementation

## Context

Keep's production admission accepts only Linux ext4. Several focused storage
ports already separate protocol ordering from filesystem effects, but their
implementations share concrete Linux assumptions without one native mechanism
boundary. Adding macOS separately to initialization, publication, retention,
migration, and recovery would duplicate authority and durability decisions.

The design baseline is main `3887f2a`, including its existing distinction
between live mount identity and restart device/inode coordinates.

## Decision

Retain the focused semantic storage ports. Implement their filesystem effects
through common Unix protocol adapters over one pinned store authority, backed
by a `PlatformFilesystem` interface. Native implementations supply observations
and OS mechanisms; shared admission policy establishes which profiles are
supported and refuses incomplete evidence.

The [architecture design](../architecture/filesystem-platforms.md) defines
the operations, ownership, Linux profile extension rules, APFS naming and
durability obligations, and acceptance gates. Backend selection happens once.
It must not alter logical identities or duplicate protocol state machines.

Native macOS support admits writable local APFS in both naming variants, with
no format difference between them. Case-insensitive volumes keep exact namespace
spelling: a name the lookup resolves must also appear byte-for-byte in a listing
of its parent, or the operation refuses before it reads, reuses, or mutates the
entry. No unsafe code or Darwin-specific dependency is needed. Other Linux filesystems
are distinct admitted profiles with their own evidence, rather than being
accepted because they provide familiar POSIX operations. Windows and network
filesystems are excluded from this implementation.

Keep existing public admission constructors and protocol entry points during
the refactor. A future convenience facade may construct typed protocol views
from retained authority, but a general object factory is not necessary for OS
selection and must never create independent competing writer authorities.

## Alternatives considered

- **A single broad `StorageInterface`:** conflates initialization, reads,
  mutation, and recovery authority; implementations must expose operations
  that are invalid in their current state. Keep narrower capability ports.
- **A factory returning unrelated storage objects:** chooses implementations
  without sharing their pinned root, observations, or lock lifetime. A
  factory can compose the design but cannot replace the platform boundary.
- **A full generic virtual filesystem:** duplicates existing capability file
  APIs and spreads type parameters through every protocol. Share Unix file
  operations and abstract only the native mechanisms that actually differ.
- **A complete protocol adapter per OS:** duplicates ordered state machines
  and lets platform implementations drift on exact-byte and recovery laws.
- **Remove the Linux gate and try ordinary file operations:** turns successful
  individual syscalls into an unsupported durability claim. Admission must
  remain tied to a named, tested filesystem profile.
- **Require a separate case-sensitive volume without evaluating default
  APFS:** avoids a useful native deployment but leaves ordinary Mac storage
  unsupported. Exact canonical namespace handling is part of the target.
- **Check spelling with `F_GETPATH`:** it reports a name cached on the vnode.
  With hard links, which publication uses, it can name another link of the same
  file, so it both refuses valid entries and misreports aliases.
- **Detect the naming variant and scan only on case-insensitive volumes:** needs
  an unsafe `fpathconf` binding to save a scan, and adds a branch whose untaken
  side no single volume exercises.
- **Claim support from Linux containers on a Mac:** exercises Linux semantics,
  not Darwin/APFS. Native acceptance requires native production-path evidence.

## Consequences

The OS port is an infrastructure boundary; semantic domain ports remain free
of descriptors and filesystem-library types. Adding an OS entails one adapter,
one selector entry, and its conformance evidence, rather than changing every
durability protocol. Filesystem support stays explicit and fail-closed.

Existing Linux wire bytes remain unchanged. Darwin recovery coordinates set the
top device bit and store `st_dev` as both device and mount, because each mounted
APFS volume has its own `st_dev`; a record written on one OS can never match a
root on the other. Logical content identity does not incorporate native metadata.

On macOS, opening an existing entry costs one streaming scan of its parent
directory. A rename that replaces a name during that scan could in principle hide
it and cause a false refusal; a bounded stress law did not observe one, and such
a refusal fails closed.

The native validation environment is separately bounded and serialized within
the same project resource budget. Successful process-death recovery is not
evidence of physical power-loss survival.
