# Shared authenticated read policy

The domain read core owns chunk authentication, complete-blob and storage-profile verification, selected-range verification, exact synchronous emission, and success receipts. It accepts an admitted layout and a crate-private immutable chunk source implemented independently by the reference map and durable catalog.

Lookup policy, canonical codec ingress, retained-root selection, filesystem admission and fences belong to the adapters. The core imports neither adapter implementations nor their codec-bearing public errors. A facade that re-exported reference-adapter code would retain the wrong ownership and was rejected.

Core failures carry semantic identities, coordinates and original causes. The shared outward read-error boundary maps them to the existing public variants without stringification, additional wrapping of I/O sources, or changed accepted-prefix accounting. Codec and missing-layout/blob variants remain at that outward boundary because the core receives an already admitted layout.

Nested internal failure enums retain those coordinates on the stack rather than allocating a box to satisfy a size lint. The scoped lint expectation records this deliberate choice; it introduces no heap allocation or new public error variant.

Chunk bytes must remain immutable for the entire verification and emission operation. The reference map and pinned catalog supply that guarantee through borrowed immutable storage; an arbitrary mutable callback source is not exposed publicly. A successful whole-blob receipt proves identity and profile verification, while a range receipt proves only the requested bytes from authenticated overlapping chunks.

The reference adapter retains its public lookup and codec entry points and its private corruption fixtures. Existing runtime tests and generated source-slice oracles retain their expectations; relocating static inspection inputs is not evidence of runtime correctness. This change does not alter formats, synchronization, recovery or supported filesystem concurrency.
