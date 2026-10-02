# Retention Publication Recovery

This page owns fixed retention stages, restart publication, and the retention process-death boundaries.

Both publication-triggered and explicit recovery first verify that `retention`, `retention/roots`, and `retention/manifests` still name the directories writer admission pinned. A replaced directory refuses as `ProtocolDirectoryReplaced` before stage observation or mutation; explicit recovery preserves that refusal inside its `Observe` error boundary.

Both entry points then admit every retention entry name and every root and manifest pool entry's canonical name and regular kind before observing stages or executing recovery. Recovery permits only the three fixed stage names in addition to the forward namespace; stage observation separately verifies their exact kinds, bounds, and bytes. An unknown retention entry, non-namespace root entry, or noncanonical pool entry refuses inside `Observe` with its existing typed namespace source and preserves every retained file's bytes.

A root or manifest pool entry named by a complete retained stage must match that stage's device and inode identity as well as its exact bytes. Observation verifies the opened pool handle and named entry on both sides of the bounded read. Equal bytes on a substituted inode classify as `Different`, so planning returns `PoolEntryDiffers` for the exact pool before finalizing `HEAD` or removing stages. Inode coordinates are transition evidence, not public content identity.

A complete staged head must name the staged manifest's exact digest, generation, byte length, and predecessor. A length disagreement refuses as `HeadStageNamesOtherManifest`; a predecessor disagreement refuses as `HeadPredecessorMismatch`. Checksums and valid individual field values do not establish this relationship. Recovery compares these coordinates before head finalization or stage cleanup, preserving the previously published head and all retained bytes on refusal.

A staged successor manifest may change only the staged root's namespace entry. Every unrelated namespace entry must preserve its exact namespace, root generation, and root digest; additions, omissions, and changes refuse as `ManifestNotSuccessor` before manifest linking or head finalization. Already-committed cleanup uses its existing current-state handling rather than reconstructing unavailable predecessor history.

At restart, a fixed retention stage is classified from its exact framing and transitive evidence:

The forward protocol guarantees that `root.next` is durable before a new namespace directory is created. A new digest-named directory is created exclusively, verified as the exact regular directory rather than a link, and followed by synchronization of `retention/roots` before the immutable root is linked. An existing exact directory is idempotent; any wrong kind, substituted namespace, or unexpected entry refuses. Directory existence alone never proves a retained root.

| Fixed stage | Complete evidence | Recovery |
| --- | --- | --- |
| `root.next` | canonical successor root, matching namespace and closure proof | finalize its immutable pool link and retain the stage |
| `manifest.next` | canonical successor manifest naming only admitted roots | finalize its immutable pool link and retain both stages |
| `head.next` | canonical successor head naming the staged manifest | finalize the head, synchronize it, then remove retained stages |

Available fixed-field bytes in an interrupted retention stage must match the canonical magic, version, header or record width, flags, anchor or entry width, and reserved fields. A contradiction is `StageCorrupt` with a `PrefixByteMismatch` source naming the exact offset, expected byte, and observed byte; recovery refuses before mutation and preserves every retained file. Absent bytes are not padded or reported as observed. Complete generation fields must also admit through the positive root or liveness generation constructor; zero is corruption even if later fields are absent. Incomplete generation fields remain undecided. These checks do not establish semantic validity of other incomplete variable fields.

Recovery synchronizes each complete staged file and reverifies its exact bytes before creating its root or manifest pool link or replacing `HEAD`. The file synchronization precedes namespace creation for a recovered root. A failed synchronization stops that recovery step before publication and preserves the on-disk stage; directory synchronization alone does not establish durability of the file contents.

A pre-effect incomplete stage may be removed only when every later-ordered effect is absent and all earlier evidence admits exactly. Recovery pins that regular file, removes it, synchronizes `retention`, and returns a typed discard report. Any later effect, stale generation, mismatched digest, missing transitive member, reappeared stage, conflicting pool entry, or other corruption is a typed refusal. A complete valid orphan remains recovery-protected until explicit disposition.

This retention protocol requires pinning the incomplete regular file. The separate [migration discard path](migration-crash.md#fixed-stage-law) revalidates the current entry's regular kind and incomplete length before removal without retaining an incomplete-stage handle. These are distinct protocol boundaries; retention recovery awaits integration from PR #99.

The retention crash points are:

| Identifier | Boundary |
| --- | --- |
| `KEEP-CRASH-036` | root stage write |
| `KEEP-CRASH-037` | root stage synchronization |
| `KEEP-CRASH-038` | new namespace-directory creation or exact admission |
| `KEEP-CRASH-039` | namespace-pool synchronization after creation |
| `KEEP-CRASH-040` | immutable root link |
| `KEEP-CRASH-041` | root namespace-directory synchronization |
| `KEEP-CRASH-042` | manifest stage write |
| `KEEP-CRASH-043` | manifest stage synchronization |
| `KEEP-CRASH-044` | immutable manifest link |
| `KEEP-CRASH-045` | manifest pool synchronization |
| `KEEP-CRASH-046` | retention-head stage write |
| `KEEP-CRASH-047` | retention-head stage synchronization |
| `KEEP-CRASH-048` | retention-head atomic replacement |
| `KEEP-CRASH-049` | committed retention namespace synchronization |
| `KEEP-CRASH-050` | retained root-stage removal |
| `KEEP-CRASH-051` | retained manifest-stage removal |
| `KEEP-CRASH-052` | retention cleanup synchronization |

`RetentionPublicationPhase::ALL` freezes this exact order as a typed public vocabulary. `FilesystemRetentionPublicationAuthority::recover` implements the classification above and its effects, and the crash matrix kills a real writer before, during, and after every point and requires restart to recover to the documented state.

Each point requires before, during, and after process-death evidence. Restart must establish exact catalog visibility, retention head, namespace generation, orphan classification, stage disposition, and recovery report.
