# Retention manifest read-bound ownership

Change kind: refactoring. Subject: static ownership of the recovery read bound, with no intended runtime change.

At parent `be71fa5`, recovery independently fixed the manifest bound at 295,136 bytes while the decoder computed canonical length from a 160-byte header, 72-byte entries and a 64-byte trailer, with the semantic limit at 4,096 entries.

Recovery now calls the decoder's checked canonical-length calculation at that semantic limit: 160 + 72 × 4,096 + 64 = 295,136, preserving the existing bound exactly and retaining the existing one-extra-byte oversize classification.

A checked calculation is used instead of an independently evaluated constant so conversion to the platform's `usize` and arithmetic overflow follow the decoder's existing typed error path; recovery preserves that source in its I/O boundary.

This before/after evidence establishes ownership and numeric equivalence, not new runtime fault coverage; no assertion on a constant or source text is added to the product test suite.

Existing manifest and recovery behavior tests provide regression checks; their expectations are unchanged. No durable format, parser acceptance rule, or publication order changes.
