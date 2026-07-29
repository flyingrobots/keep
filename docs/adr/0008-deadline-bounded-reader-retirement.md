# ADR-0008: Deadline-Bounded Reader Retirement

- Status: Accepted
- Date: 2026-07-28
- Owners: Keep repository verification
- Related issue: #59
- Depends on: ADR-0007

## Context

Repository verification tasks run external tools synchronously and drain
standard output and standard error on dedicated reader threads. Concurrent
draining prevents a child from blocking when either pipe fills, and per-stream
capture limits prevent unbounded retained output.

A failed child does not guarantee that a reader thread can be joined. A
descendant may retain an inherited pipe, an injected reader may stop making
progress, or an operating-system read may remain blocked after the primary
operation has failed. An unbounded join would turn a typed timeout,
interruption, or collection refusal into an unbounded caller hang.

Stable Rust does not provide a safe operation that cancels an arbitrary blocked
thread. Keep forbids unsafe code, and asynchronous process I/O would introduce a
runtime and a second process-lifecycle model without a demonstrated consumer
need.

## Decision

Failed captured-process operations perform cleanup in this order:

1. Drop the parent's child-standard-input handle.
2. Terminate the child's dedicated process group.
3. Kill the direct child as an idempotent fallback.
4. Poll direct-child reaping for one fixed per-step cleanup grace.
5. Give each reader worker one fixed per-step cleanup grace to publish its
   bounded result.

Reader retirement is successful when the caller already received the worker's
only result or the result arrives within the cleanup grace. If a worker remains
blocked, its join handle is detached. This bounded retirement is permitted only
on an already-failed operation; it can never convert a failure into success.

The returned error retains the primary failure as its source. A reader
retirement timeout is attached as an additional typed failure, so bounded
retirement does not hide either the cause of the operation failure or the
incomplete cleanup observation.

Process-group termination and direct-child reaping precede reader retirement.
For real child pipes, terminating every process that could retain the pipe is
the operation that makes reader completion reachable. The retirement grace is
not a durability claim and does not prove that an arbitrary injected reader has
stopped; it bounds how long repository verification waits for that evidence.

## Alternatives considered

- Joining every reader without a deadline was rejected because one inherited or
  stalled pipe could hang repository verification forever.
- Detaching every reader immediately was rejected because it would discard
  available read failures and make ordinary cleanup nondeterministic.
- Cancelling blocked threads through platform-specific or unsafe APIs was
  rejected because Rust cannot make arbitrary cancellation memory-safe and
  Keep forbids unsafe code.
- Moving subprocess capture to an asynchronous runtime was rejected because the
  synchronous core already has explicit deadlines and no consumer requires an
  async process boundary.
- Reading the streams sequentially was rejected because either child pipe can
  fill while the other stream is being drained.

## Consequences

Every failed captured-process operation returns after bounded cleanup steps.
The caller receives the primary typed failure plus any observed cleanup
failure. Successful operations still require both bounded reader results and
the direct child's exit status.

A detached reader thread may remain alive until its underlying read completes
or the process exits. It owns only its reader, bounded output accumulator, and
single-result sender. It owns no Keep data, repository lock, mutation
authority, or caller reference. This residual lifetime is explicit uncertainty,
not a claim that cleanup completed.

The two-second cleanup grace is a per-step bound. It is separate from the
operation deadline and may be consumed once for child reaping and once for each
reader. Tests use zero-duration injected retirement and reap boundaries; they
do not classify behavior from scheduler timing.

This decision changes no Keep library API, content identity, durable format, or
recovery protocol. It governs only private repository-process orchestration in
`xtask`.
