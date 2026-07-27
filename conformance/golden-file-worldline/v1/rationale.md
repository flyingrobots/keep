# Golden File Worldline v1 Path Rationale

Status: accepted for
`keep.golden-file-worldline.path/v1`.

## Decision

Golden File Worldline source parameters use nonempty UTF-8 relative paths with
literal `/` separators. Every segment must be nonempty and must differ from
`.` and `..`. Backslash, colon, and NUL are invalid anywhere in the field.

Reject rather than normalize a spelling that violates this grammar. The
protocol never rewrites separators, removes components, case-folds text,
decodes escapes, or performs Unicode normalization.

Filesystem admission is a separate boundary. After lexical admission, the
checker resolves the path relative to its retained corpus-directory
capability, opens one handle, verifies that handle is a regular file, and
consumes the same handle. A path can therefore be canonical protocol text but
still be unavailable on a particular checkout.

## Why

Host path parsers disagree about backslash, drive prefixes, alternate data
streams, NUL, repeated separators, and parent components. Letting a host parser
interpret those spellings before the protocol does would make one corpus name
different resources on different systems.

The colon ban is intentionally broader than rejecting a leading drive prefix.
It also prevents a path segment from becoming an alternate data-stream
coordinate on hosts that implement that syntax.

Exact refusal preserves auditability. Normalization would create aliases
between distinct TSV fields, hide malformed producer output, and make future
format evolution depend on an implicit host or library algorithm.

## Rejected alternatives

- Native separators were rejected because the same bytes would not have one
  cross-host interpretation.
- Canonicalizing with the host filesystem was rejected because it is
  state-dependent, follows filesystem objects, and occurs too late to define
  protocol text.
- Percent escaping was rejected because version 1 does not need arbitrary host
  filenames and an escape codec would enlarge the canonicalization surface.
- Allowing colon outside a drive prefix was rejected because its meaning is
  host-dependent.

## Consequences

The grammar names a canonical protocol spelling; it does not promise that
every admitted spelling can be materialized on every filesystem. Corpus
authors must choose paths available to supported checkouts. Any future need
for a broader or escaped namespace requires a new named path-profile version,
new fixtures, and migration guidance rather than silent relaxation.
