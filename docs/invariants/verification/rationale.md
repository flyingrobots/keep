# Verification boundary rationale

## Typed namespace observations

Canonical membership and no-follow entry-kind observations belong to filesystem namespace admission, including the earlier platform directory traversal. Their typed source records a demonstrated contradiction; an I/O error alone does not. Verification consumes that evidence rather than parsing messages or broadly equating InvalidData, ELOOP or NotADirectory with corruption. Failed directory iteration remains operational before any membership inference. The shared guard leaves existing no-follow opens and opened-file checks intact and does not make later pathname operations conditional on inode identity. Catalog-selected segment I/O similarly retains its known digest and original cause, so absence identifies the missing evidence without renaming the present catalog. These additive diagnostic types enrich the public error surface; downstream exhaustive matches may need new arms, without changing durable formats or successful behavior.

## Subject-specific supported sets

Verification depth has equality and no total ordering. Catalog reachability, layout identity and retention closure describe different subjects; ordering enum discriminants would falsely imply proofs an operation did not establish. Each operation declares its exact supported set and returns the requested depth or a typed unsupported refusal. The reference verifier's three supported depths retain their local validation prerequisites without imposing an ordering on unrelated durable subjects.

## Reference refusal precedence

The reference verifier authenticates chunks and, at complete-blob depth, replays the profile and computes complete identity in one pass. A profile contradiction is retained until chunk authentication finishes so a chunk-identity refusal is not obscured. Repeating the chunk hash pass was rejected because the reference view already owns immutable bytes.

## Frozen receipt projection

The v1 receipt codec owns closed wire subjects and depths separately from the expanding runtime vocabulary. Checked projection rejects unsupported coordinates and reference-to-durable provenance substitution. Accepting an arbitrary caller-supplied view would turn serialization into fabricated evidence. Historical durable-view records remain decodable without treating their bytes as a newly established live report.
