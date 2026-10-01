# Migration recovery admission

Recovery must return the exact named bytes or refuse. A residue snapshot is
untrusted evidence, not writer authority. Planning precedes record adoption,
truncated-stage removal, and all forward writes. Nested directory membership
is checked before adoption so a planner-visible prefix cannot hide foreign
entries that would otherwise fail only after creating the next stage.

The phase-resumption helper is private to recovery. Exposing it would let
callers manufacture a completion receipt without current-state verification,
residue planning, or adoption. The public recovery entry point owns that order.

A retained intent stage and a later namespace effect cannot arise from the
ordered protocol: stage removal precedes namespace admission. The analogous
marker-stage/receipt combination also refuses. Treating either as a harmless
cleanup would erase evidence of an ambiguous write history.

Resume the earliest unproven synchronization and use persisted intent bytes.
Device/inode coordinates survive restart; mount identity remains a fence for
one live authority and is not a persisted restart identity. No format bytes
change. Sealed version-1 objects remain untouched. A completed observation
performs no migration writes and does not grant retention publication authority.

The 68-case subprocess matrix tests process death, not physical power loss.
Exhaustive strict-stage truncations and forward-prefix laws supplement it;
broader hostile restart combinations remain independently tracked in #111.
