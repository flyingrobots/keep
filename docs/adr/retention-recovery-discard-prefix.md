# Retention recovery discard prefix

Status: Accepted

The forward protocol begins a manifest write only after the complete root stage is linked, and begins a head write only after complete root and manifest stages are linked. Retained earlier stages are removed only after head replacement. An incomplete later stage with absent earlier evidence therefore cannot be treated as a routine interrupted write.

Pure recovery planning admits the complete linked earlier prefix before scheduling an incomplete manifest or head for deletion. Missing, incomplete, or differently linked earlier evidence returns `TruncatedStageWithoutEarlierEvidence`, naming the incomplete stage and the earlier stage that failed admission. Subsequent existing planning still checks root history, manifest history, and cross-record relationships before executing any scheduled discard.

The filesystem observer supplies exact record admission and retained-stage inode binding. This guard does not infer completeness or identity from pathname existence, and it does not change on-disk encoding. The refusal enum is nonexhaustive; the new typed variant makes an impossible prefix distinguishable from a later-effect conflict.

Runtime regressions remove one earlier stage or link from an otherwise valid forward prefix and shorten the later stage to an admissible framing prefix. Refusal must preserve every retained byte. Legitimate incomplete prefixes remain discardable under the existing forward-prefix laws; concurrent replacement during unlink and closure verification remain separate obligations.
