# ADR-0010: Filesystem Platform Adapters

- Status: Accepted design; implementation and native acceptance in progress
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

Native macOS support targets writable local APFS. Exact namespace spelling
must remain enforced on case-insensitive volumes. Other Linux filesystems
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
- **Claim support from Linux containers on a Mac:** exercises Linux semantics,
  not Darwin/APFS. Native acceptance requires native production-path evidence.

## Consequences

The OS port is an infrastructure boundary; semantic domain ports remain free
of descriptors and filesystem-library types. Adding an OS entails one adapter,
one selector entry, and its conformance evidence, rather than changing every
durability protocol. Filesystem support stays explicit and fail-closed.

Existing Linux wire bytes remain unchanged. Darwin physical recovery
coordinates need a documented mapping and compatibility analysis; logical
content identity must not incorporate native metadata. Any format change
discovered necessary during implementation requires an explicit revision of
this decision and the format contract before it ships.

The native validation environment is separately bounded and serialized within
the same project resource budget. Successful process-death recovery is not
evidence of physical power-loss survival. Design acceptance does not claim
that implementation, resource enforcement, or platform validation is complete.
