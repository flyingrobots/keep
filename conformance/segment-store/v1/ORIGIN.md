# Durable Segment Store Corpus v1 Origin

The corpus was created for issue
[#14](https://github.com/flyingrobots/keep/issues/14) from the accepted
`keep.segment-store/v1` field tables and checksum preimages.

Construction used these independent inputs:

1. fixed-width headers were encoded directly from the published offset tables;
2. the one-zero chunk length and digest were copied from
   `conformance/chunk-id/v1/identities.tsv`;
3. the one-zero layout bytes and canonical binary `LayoutId` were copied from
   the independently frozen `conformance/layout/v1` corpus;
4. the test-only Rust fixture oracle reconstructed every artifact without
   calling a production segment, catalog, or publication codec;
5. each record checksum, segment digest, seal checksum, catalog checksum,
   catalog digest, and head checksum was recomputed from its named
   domain-separated preimage; and
6. every embedded length, count, offset, digest, and checksum was compared
   with the enclosing artifact before the fixtures were admitted.

The fixture oracle is executable evidence, not a production implementation.
Issue #15 must test its encoder and decoder against the frozen bytes and must
retain mutation and corruption cases discovered during implementation. Issue
Issue #16 must do the same for catalog and head publication. Issue #17 must
implement
the transition ledger as a crash-injection oracle.

The source byte is synthetic (`00`), license-safe, and contains no private
workspace material, timestamps, host paths, random values, or application
policy.
