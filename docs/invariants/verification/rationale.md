# Verification Rationale

This note records the governed decisions behind the verification vocabulary.

## An ordered enumeration, not flags

A policy of boolean flags (`check_chunks`, `check_blob`, ...) lets a caller
ask for a combination nothing establishes and lets a report be read as more
than it says. One ordered depth makes both impossible: the request is one
value, the report is one value, and the deeper value implies the shallower
ones the view supports.

Rejected: a bit set of independent checks. Rejected: a boolean `deep`
parameter, which collapses five distinct propositions into two.

## Refuse an unsupported depth instead of degrading

A view that silently verified to the deepest depth it could would return a
report whose depth the caller did not ask for, and a caller comparing
`report.depth() >= requested` would be the only defense. Refusing with the
supported range keeps the report's depth equal to the request by law.

## Report the lowest stage first

The reference store folds complete-blob hashing and profile replay into the
single chunk pass so that no chunk is hashed twice. A profile-boundary
contradiction can therefore surface before the last chunk has been
authenticated. It is held until the pass completes, so a chunk-identity
contradiction (a lower stage) is the one reported when both exist. Callers
can rely on the stage order without knowing the pass structure.

Rejected: two passes, one per stage, which doubles the work the read path
already avoids.

## Missing, corrupt, and ambiguous stay distinct

The authenticated reconstruction contract separates evidenced refusal from
operational failure. Verification refines the refusal into absence,
contradiction, and conflict because each authorizes a different next step:
absence can be resolved by ingestion, contradiction by restoring from another
copy, and conflict only by a human reading both sides. Collapsing them into
one `Invalid` would push that distinction into prose.

`Ambiguous` is defined now with no reference-store producer so that durable
views, which can hold conflicting catalog and retention evidence, do not
introduce a fourth vocabulary later.
