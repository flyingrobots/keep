# Retention recovery selected root admission

This decision owns reopening the published root selection before filesystem retention recovery effects. A manifest entry supplies coordinates; it does not prove that the corresponding predecessor pool record remains available or authentic after a crash.

Regression commit `3cd7bde` records real-filesystem RED on production `48187bd`, whose documentation-only successor is `8686931`. Missing, checksum-corrupt, and canonical substituted predecessor records each allowed recovery to link the successor root, link its manifest, or replace the head. Every tested transition failed the named retained-byte preservation assertion.

After pure planning and before reopening executable stages, recovery now reuses forward publication's selected-root admission. An uncommitted successor reopens the manifest-selected predecessor through the pinned roots capability, bounds the read, decodes the record, and compares its generation and digest to that selection. An already-selected staged root instead uses committed-root admission to require its exact bytes; a newly inserted namespace has no published predecessor to reopen.

Both direct restart recovery and publication's automatic recovery enter this guard. Missing predecessors retain `PredecessorRootAbsent`; corrupted or substituted predecessors retain `PredecessorRootChanged`, carried by the observation error. The guard performs no writes, links, cleanup, or directory creation.

Trusting manifest coordinates without reopening their record was rejected because it permits publication from unavailable evidence. Deferring the read until cleanup was rejected because head replacement would already have occurred. Repairing or substituting a predecessor was rejected under the Core Law. Durable formats and identities remain unchanged.

The fault matrix crosses missing, corrupted, and substituted records with root linking, manifest linking, and head replacement. The tests observe all retained file bytes and the exact typed refusal. These checks do not establish full staged closure verification, every predecessor-coordinate rule in the pure planner, safety against arbitrary later out-of-band replacement, or physical power-loss durability.
