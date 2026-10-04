# Retention recovery namespace admission

This decision owns namespace admission before filesystem retention recovery effects in PR #99. It preserves the existing rule that unknown or noncanonical protocol state is ambiguity and must refuse without destroying evidence.

Previously, publication invoked recovery before its forward namespace census. An unknown retention entry could therefore receive the correct publication refusal after recovery had deleted an otherwise canonical incomplete stage. Direct recovery omitted that census entirely and could return `Clean` for the same invalid namespace. Test commit `282c105` records both failures on unfixed production `a038b1c`.

Recovery first revalidates the pinned protocol directories and invalidates any pending in-memory publication attempt. It then admits retention entry names and root and manifest pool names and kinds, before observing stage bytes, planning, reopening stage handles, or executing filesystem effects. The existing typed namespace refusal remains the source of its `Observe` error. Publication invokes this same recovery entry point and retains its strict forward census afterward.

The forward and recovery censuses share one admission implementation. Forward publication permits only `HEAD`, `roots`, and `manifests`; recovery additionally permits `root.next`, `manifest.next`, and `head.next`. Accepting these stage names is not accepting their contents: the existing bounded, no-follow stage observation and canonical assessment still decide their kinds, bytes, and semantics.

Using the forward-only name list for restart was rejected because lawful crash stages would refuse. Keeping admission after recovery was rejected because it destroys unadmitted evidence. Silently deleting unknown entries or treating pool filenames as content proof was rejected under the Core Law. No new durable encoding or public error variant is introduced.

The real-filesystem laws exercise explicit and publication-triggered recovery, check the exact typed namespace source, and compare every retained file's bytes before and after refusal. This establishes names-and-kinds admission ordering; it does not prove complete pool-content validation, capacity enforcement in every recovery transition, or physical power-loss survival. Those remain independent review obligations.
