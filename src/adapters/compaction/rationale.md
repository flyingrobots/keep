# Compaction recovery preflight

Status: accepted for PR #107 integration.

The recovery driver may observe more than one retained stage. Discarding an admissible earlier stage before examining a later contradictory stage would destroy evidence during an invocation whose result is refusal. The driver therefore constructs an effect-free action queue for both staging artifacts and `head.next`, then executes that queue under the retained writer authority. The lower protocol capabilities still revalidate their requests when executing them.

An error loading the current catalog is not evidence that the store has never published. Only a missing `HEAD` at the `OpenHead` boundary supplies the uninitialized expectation; decoder, policy, missing selected artifact and other I/O failures retain their original typed causes. A valid generation-two candidate beside a corrupt current head demonstrates why deferring this distinction to the finalizer is insufficient: the planner otherwise emits an unrelated initial-generation error first.

The alternative of checking only `head.next` before the old sequential loop is rejected because every stage's planning can refuse. All requests must be admitted before any stage executes. Execution failure remains distinct from preflight refusal and does not imply rollback. This decision supplies no isolation guarantee against arbitrary concurrent out-of-band mutation; Keep's cooperating-writer contract and existing identity/evidence checks still apply.

This change preserves successful recovery of the existing interrupted publication phases. It introduces neither an on-disk record nor a cleanup namespace, and it does not change the incomplete-retention-stage refusal contract. Compaction's separate execution-effect and derivable-stage identity obligations remain open until their own runtime evidence establishes them.

Stage lengths are admitted before reading their contents. Reusing the shared no-follow stage observation boundary preserves format ceilings, exact-byte fingerprinting and namespace checks instead of allocating an arbitrary file before deciding it is oversized. The queue and its cleanup rereads still materialize bounded stage bytes, and catalog verification has separate allocation limits; this decision makes no constant-memory claim.
