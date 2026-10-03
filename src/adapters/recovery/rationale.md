# Partial seal contradiction admission

KEEP-RECOVERY-010 distinguishes demonstrably corrupt framing from an incomplete segment. A recognized seal magic is insufficient to authorize truncation: already-observed version, flags, length, algorithm and reserved bytes must agree with the version-one format before assessment can issue a discardable state (#171).

The seal-prefix validator borrows format constants and returns existing precise seal diagnostics through an explicit incomplete-seal error boundary. Missing bytes are filled only in temporary fixed-field comparisons; the result never admits those bytes or promises that variable coordinates, digests or the entire seal have a canonical future completion. Complete-seal admission remains the full decoder's responsibility. This correction adds no on-disk format or recovery mutation capability and does not change the separate version-two incomplete-retention-stage disposition policy.
