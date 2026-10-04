# Retention recovery pool identity

This decision owns exact pool-link admission before filesystem retention recovery effects in PR #99. Immutable pool names identify canonical content, but a retained publication stage also witnesses the exact file object the publication protocol linked.

Previously, pool observation compared only bytes. Replacing a root or manifest pool link with a distinct byte-identical file therefore yielded `Identical`. Recovery finalized `HEAD`, then stage removal refused the inode mismatch; the manifest substitution also allowed root-stage removal before refusal. Test commit `7d45af2` records both real-filesystem failures on unfixed production `aa71bc0`.

Each complete stage observation carries the device and inode identity read from its opened handle. Pool observation passes that identity and the exact canonical bytes to the existing bounded, no-follow exact-record verifier. The verifier checks the opened pool handle and named entry before and after reading. Absence remains `Absent`; a kind, length, identity, or byte contradiction becomes `Different`; operational I/O failures retain their source.

The pure planner continues to return the existing pool-specific `PoolEntryDiffers` refusal for `Different` before executing any recovery effects. Pool identity is a storage transition witness and never becomes stable public content identity. Forward stage publication and removal already enforce the same physical binding, so recovery now admits it before head publication rather than discovering its failure during cleanup.

Adopting equal bytes on a different inode was rejected because it weakens the exact hard-link publication contract. Repairing the pool link or delaying validation until cleanup was rejected because ambiguous evidence must refuse before mutation. No format bytes, public error variants, or logical identities change; the public observation documentation now states the required stage-object binding.

The regressions confirm equal bytes on distinct inodes, require `HEAD` to remain absent, require the exact pool-specific planning refusal, and compare every retained file's bytes. Existing successful recovery laws retain same-inode hard-link admission. This establishes observation-time substitution refusal; it does not establish safety against every later concurrent replacement, full closure or predecessor admission, or physical power loss. Those remain independent review obligations.
