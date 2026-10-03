# Recovery counterexamples

`unsupported-partial-seal-version.hex` is a version-one empty-segment header followed by the seal magic and version 2. It derives from `conformance/segment-store/v1/empty-segment.hex` by removing records and the seal suffix and changing the version's low byte. It retains only the 82 bytes needed to present this particular unsupported version after a valid header and seal discriminator. This is a corrupt regression input, not a canonical format vector.

The public `recovery_partial_seal` tests replay it on every ordinary test pass. `cargo xtask prepare-fuzz-corpus` additionally prefixes selector 5 and materializes it for the registered `segment_format` target. The fixed-framing oracle must reject a discardable classification for these bytes; the replay was observed RED against main and GREEN with #171. Preserve this input unless a stronger, cheaper runtime law demonstrably subsumes its corruption/refusal contract.
