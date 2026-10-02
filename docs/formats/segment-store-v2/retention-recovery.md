# Retention Publication Recovery

This page owns fixed retention stages, restart publication, and the retention process-death boundaries.

Both publication-triggered and explicit recovery first verify that `retention`, `retention/roots`, and `retention/manifests` still name the directories writer admission pinned. A replaced directory refuses as `ProtocolDirectoryReplaced` before stage observation or mutation; explicit recovery preserves that refusal inside its `Observe` error boundary.

Both entry points then admit every retention entry name and every root and manifest pool entry's canonical name and regular kind before observing stages or executing recovery. Recovery permits only the three fixed stage names in addition to the forward namespace; stage observation separately verifies their exact kinds, bounds, and bytes. An unknown retention entry, non-namespace root entry, or noncanonical pool entry refuses inside `Observe` with its existing typed namespace source and preserves every retained file's bytes.

A root or manifest pool entry named by a complete retained stage must match that stage's device and inode identity as well as its exact bytes. Observation verifies the opened pool handle and named entry on both sides of the bounded read. Equal bytes on a substituted inode classify as `Different`, so planning returns `PoolEntryDiffers` for the exact pool before finalizing `HEAD` or removing stages. Inode coordinates are transition evidence, not public content identity.

A complete head stage without a manifest stage refuses as `HeadStageWithoutManifestStage`. When both head and manifest stages are complete but the root stage is absent, recovery instead reports `ManifestStageWithoutRootStage`, preserving the retained evidence before any execution.

A complete staged head must name the staged manifest's exact digest, generation, byte length, and predecessor. A length disagreement refuses as `HeadStageNamesOtherManifest`; a predecessor disagreement refuses as `HeadPredecessorMismatch`. Checksums and valid individual field values do not establish this relationship. Recovery compares these coordinates before head finalization or stage cleanup, preserving the previously published head and all retained bytes on refusal.

A staged successor manifest may change only the staged root's namespace entry. Every unrelated namespace entry must preserve its exact namespace, root generation, and root digest; additions, omissions, and changes refuse as `ManifestNotSuccessor` before manifest linking or head finalization. Already-committed cleanup uses its existing current-state handling rather than reconstructing unavailable predecessor history.

Before executing a recovery plan for a complete staged root, the filesystem authority reopens the published root selected for that namespace through its pinned roots capability. A successor requires the predecessor record to decode to the manifest's exact generation and digest; an already-selected staged root requires exact committed bytes. Missing, corrupt, or substituted selections refuse at observation before any recovery effects. A newly inserted namespace has no published predecessor.

Every complete staged root also requires a fresh catalog snapshot loaded relative to the authority's pinned root capability. Recovery authenticates the current head, catalog, and all selected segments, then replays every staged anchor and its stored closure limits through the same pure verifier used by preflight. Missing members, corruption, coordinate disagreement, and closure-limit failures remain typed observation sources and refuse before root linking, manifest linking, head replacement, or cleanup. Forward publication performs the same live verification after checking the prepared catalog coordinates; the fresh proof must still name that prepared catalog.

`recover_with_catalog_policy` accepts explicit segment-admission and aggregate retained-segment byte limits. It materializes the selected catalog and all selected segments; catalog encoding and decoded indexes have additional protocol bounds. `recover` and forward publication use an aggregate retained-segment cap equal to the protocol's maximum segment length, 1 GiB, independently of the staged root's closure counters. Larger selections refuse before mutation; a recovery caller may supply a larger explicit policy. This cap is a loading policy rather than an on-disk format limit or a total resident-memory claim. Clean recovery and recovery without a complete staged root do not load segment bodies.

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

Once all bytes of a staged head's manifest-length field are available, recovery requires that value to satisfy the canonical manifest bounds and entry alignment even if the rest of the head is missing. An invalid value refuses as `StageCorrupt(Head)` with its typed `ManifestLength` cause, preserving the retained evidence.

When a short head contains all generation and predecessor bytes, recovery applies the same semantic history rules as complete head decoding: generation one cannot name a predecessor, and a successor must name one. Contradictions refuse as `StageCorrupt(Head)` with the exact semantic cause before any stage removal.

Once a short head contains the entire checksum preimage, recovery computes its checksum and checks every checksum byte already present. Any mismatch refuses as `StageCorrupt(Head)` with `PrefixByteMismatch` naming the offset and exact expected and observed bytes; the incomplete stage remains intact.

A root or manifest prefix containing all framing-size fields must declare the exact length those fields imply. Recovery uses the same checked framing calculation as complete decoding and refuses a disagreement with `DeclaredLengthMismatch` inside the exact stage-corruption error before discarding any stage.

Available complete anchor and manifest-entry counts must satisfy their format ceilings before interrupted-stage discard. Excessive counts refuse with `AnchorCountExceeded` or `EntryCountExceeded` inside the corresponding stage-corruption error, even when the declared record length matches the excessive count.

A complete namespace-length field in an interrupted root must declare between one and 255 bytes. Recovery uses the domain namespace admission rule without allocating a namespace or waiting for its payload; empty or oversized declarations refuse with the exact namespace cause and preserve the stage. Namespace contents remain opaque and are not interpreted as text or paths.

When an interrupted root or manifest contains its complete generation and predecessor fields, recovery applies the same domain history rules as complete-record construction. Initial records naming a predecessor and successors omitting one refuse with the precise semantic cause before stage discard; all retained evidence remains intact.

When an interrupted root contains the complete realization-profile group (through byte 88) or closure-policy group (through byte 116), recovery applies the domain profile and closure-limit validators before discarding the stage. Unsupported profile coordinates, definition-digest mismatches, zero limits, and excessive limits refuse as `StageCorrupt(Root)` with their exact typed cause and preserve retained evidence. For an incomplete profile group, every available identity, version and digest byte must match the sole registered profile; a contradiction refuses with `PrefixByteMismatch` naming only an observed byte. Each complete closure-limit field is admitted independently as soon as it arrives. For a partial big-endian limit, recovery checks whether any positive completion fits the domain ceiling; an impossible prefix refuses with `ClosureLimitPrefixAboveMaximum`, reporting the resource, minimum possible completion and maximum admitted value. Missing bytes are never reported as observed, and incomplete all-zero prefixes remain admissible because they can still become positive.

For interrupted root and manifest records, recovery verifies each available trailer digest or checksum byte once its complete preimage is present. After the body has arrived, it also verifies the header's anchor-set or entry-set digest. Contradictions refuse before discard with the exact available-byte coordinates or body-set digest cause; canonical prefixes remain interrupted writes. Complete-record decoding retains its existing checksum, digest and semantic validation order.

Each complete anchor or manifest entry already present in an interrupted body is admitted by the same identity, generation and strict-order rules used by full decoding. Recovery scans only available complete entries without allocating the unavailable suffix; typed entry failures preserve the stage. For a partial manifest entry, recovery admits its root generation once complete and compares the greatest possible namespace completion against the preceding namespace; an impossible strict ordering refuses with the current entry index. Unknown namespace bytes are used only to compute that bound, never exposed as an observed identity. Partial anchor entries, remaining-count feasibility and partial header constraints remain under remediation.

A truncated manifest requires a complete root stage already linked to its immutable pool entry. A truncated head requires both complete root and manifest stages already linked to their immutable pool entries. Missing or incomplete earlier evidence refuses as `TruncatedStageWithoutEarlierEvidence`, naming both stages, before any scheduled discard or relinking. Existing corruption and conflicting-pool diagnostics retain precedence; cross-record and history admission still precede execution.

This retention protocol requires pinning the incomplete regular file. The separate [migration discard path](migration-crash.md#fixed-stage-law) revalidates the current entry's regular kind and incomplete length before removal without retaining an incomplete-stage handle. These are distinct protocol boundaries.

Retention recovery is implemented in this branch, with correctness remediation and independent acceptance tracked in [PR #99](https://github.com/flyingrobots/keep/pull/99). Incomplete-stage pinning and failure handling after removal remain open implementation findings; the requirements above remain binding.

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
