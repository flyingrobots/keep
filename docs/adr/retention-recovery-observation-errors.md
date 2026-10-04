# Publication preserves recovery observation failures

Status: accepted.

Publication performs restart recovery before admitting a forward attempt, so callers must distinguish a failure while observing recovery evidence from a later forward current-state refusal.

`RetentionCurrentStateRefusal::RecoveryObservationRefused` wraps the original observation `io::Error`, exposes it through the standard error source chain, and is itself returned through publication's existing `CurrentVerification` boundary.

The outer storage I/O kind is `InvalidData`, consistent with the other typed current-state refusals; the original I/O kind, raw OS code and any typed source remain in the nested observation error. This is an intentional diagnostic API change for the unreleased publication API, including failures that previously escaped as raw I/O errors.

Explicit recovery continues to return `FilesystemRetentionRecoveryError::Observe`. Planning and execution failures retain their existing `RecoveryRefused` and `RecoveryStepRefused` wrappers during publication.

Returning observation failures directly was rejected because it erased which recovery phase had failed. Stringifying them was rejected because callers need their original typed or operating-system cause. This changes diagnostic structure without changing effects, publication ordering, durable encodings or the observation refusal itself.
