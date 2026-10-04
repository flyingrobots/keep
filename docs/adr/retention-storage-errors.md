# Typed retention storage failures

Status: accepted.

Recovery callers must distinguish an operational failure from a disagreement between a retained record and its admitted evidence.

The shared stage adapter previously merged distinct exact-record refusals into message strings, losing the variant before recovery execution could report it.

`RetentionStorageError` now separates source-preserving I/O failures from `RetentionRecordRefusal`, a public semantic vocabulary that maps the filesystem exact-record refusals without importing private filesystem machinery into the recovery port.

Every recovery capability returns this error and `RetentionRecoveryError` retains it with a typed accessor and its standard error source chain.

Stage operations share the typed boundary across recovery and forward publication; forward publication keeps its existing I/O signature by preserving operational errors directly and boxing typed record refusals as the source of `InvalidData`.

This is an intentional source-level change to the unreleased recovery-storage trait; downstream implementations must return the new error type, while durable encodings and operation ordering are unaffected.

An I/O-only recovery port with downcasting at every consumer was rejected because it obscures the distinction the recovery API needs to expose directly.

The existing messages for invalid recovery invocation state are retained as operational errors; this decision addresses exact-record refusal preservation and does not claim every internal protocol-state diagnostic is now a dedicated variant.
