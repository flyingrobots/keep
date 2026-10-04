# Retention reader catalog pinning

Status: accepted.

The retention reader must load catalog bytes from the same opened store directory that supplies its collected publication coordinates.

Reopening the ambient path during collection can select a replacement directory while coordinate reads continue through the original directory capability, producing a refusal or a mixed view unrelated to the admitted root.

The filesystem reader therefore uses the existing directory-based catalog restart loader with its pinned root capability and unchanged restart policy.

This changes neither the durable format nor the public API and adds no catalog allocation or retry beyond the existing restart loader.

The regression drives the production collection port with a real migrated store, renames that store after pinning, places an empty replacement at the ambient path, and requires the original catalog generation and digest.

Full public-loader scheduling during admission, head-coordinate completeness, and catalog error classification remain separate obligations.
