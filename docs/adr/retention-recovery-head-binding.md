# Retention recovery head binding

This decision owns the cross-record relationship of a complete staged retention head and its selected manifest in PR #99. Canonical framing and checksums admit each record independently; they do not prove that the records describe one transition.

Previously, recovery compared head and staged manifest digest and generation but omitted the exact manifest byte length and predecessor. A checksummed head with another valid length or another predecessor could therefore replace `HEAD` and permit stage cleanup, leaving a committed head that ordinary current-state observation rejects. Test commit `cd9e948` records both real-filesystem failures on unfixed production `6852d41`.

The pure recovery planner now compares all four coordinates before finalization. It converts the admitted manifest's encoded length with checked conversion, requires equality with the head's declared length, and returns `HeadStageNamesOtherManifest` on disagreement. It requires head and manifest predecessor equality and returns `HeadPredecessorMismatch` on disagreement. The existing comparison against published predecessor history remains a separate requirement.

Accepting a shared digest as sufficient was rejected because other persisted head fields remain user-visible protocol claims. Deferring the check to post-publication observation was rejected because recovery would already have committed ambiguous evidence. Re-encoding or repairing the head was rejected under the Core Law. No durable bytes, logical identities, or public error variants change.

The filesystem regressions require preservation of the absent or previous published head, the exact typed planning refusal, and all retained bytes. A deterministic public-planner sweep exercises every canonical manifest length derived from the bounded entry domain, refusing every mismatch and accepting the exact length. This finite sweep reports the first counterexample directly; it uses no ambient randomness or arbitrary case total as its oracle.

These checks establish staged head-to-manifest binding. They do not establish complete successor entry-set preservation, predecessor root reopening, closure verification, or physical power-loss durability. Those remain independent review obligations.
